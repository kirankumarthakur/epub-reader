import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

type EpubMetadata = {
  title: string
  author: string
  total_pages: number
  current_page: number
}

type Chapter = {
  title: string
  content: string
}

const App = () => {
  const [path, setPath] = useState<string|null>(null)
  const [metadata, setMetadata] = useState<EpubMetadata|null>(null)
  const [page, setPage] = useState<number>(0)
  const [chapter, setChapter] = useState<Chapter|null>(null)
  const [single, setSingle] = useState(true)
  const contentRef = useRef<HTMLDivElement|null>(null)

  useEffect(() => {
    load_epub()
  }, [path])

  useEffect(() => {
    load_chapter()
  }, [page])

  useEffect(() => {
    const handleClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (target.tagName === 'A') {
        const href = (target as HTMLAnchorElement).href;
        console.log('Hyperlink clicked:', href);
        event.preventDefault();
      }
    }

    document.addEventListener('click', handleClick);
    return () => {
      document.removeEventListener('click', handleClick);
    };
  }, [])

  useEffect(() => {
    const checkHeight = () => {
      const height = contentRef.current?.offsetHeight || 0;
      const windowHeight = window.innerHeight;
      setSingle(height < windowHeight * 2)
    }

    checkHeight()
    window.addEventListener('resize', checkHeight)
    return () => window.removeEventListener('resize', checkHeight)
  }, [chapter])

  const picker = async(): Promise<boolean> => {
    const new_path = await open({
      multiple: false,
      filters: [{
        name: 'Epub Files',
        extensions: ['epub']
      }]
    })

    if (new_path != null && typeof new_path === 'string') {
      setPath(new_path)
      return true
    }
    return false
  }

  const load_epub = async() => {
    if (path != null) {
      setMetadata(await invoke('load_epub', {path}))
      if (metadata != null) {
        setPage(metadata.current_page)
      }
    }
  }

  const load_chapter = async() => {
    if (path != null && metadata != null && page >= 0 && page < metadata.total_pages) {
      setChapter(await invoke('get_page', {page}))
    }
  }

  const NavigationControls = () => (
    <form className="flex justify-center gap-4 my-4">
      <button
        className="px-4 py-2 font-medium bg-white rounded border hover:border-blue-600 dark:bg-black/60 dark:text-white"
        onClick={(e) => {
          e.preventDefault();
          if (page > 0) setPage(page - 1)
          window.scrollTo({top: 0, behavior: 'smooth'})
        }}
      >
        Decrease Page
      </button>

      <button
        className="px-4 py-2 font-medium bg-white rounded border hover:border-blue-600 dark:bg-black/60 dark:text-white"
        onClick={async (e) => {
          e.preventDefault();
          const pick = await picker();
          if (pick) window.scrollTo({top: 0, behavior: 'smooth'})
        }}
      >
        {metadata ? metadata.title : "Pick Epub File"}
      </button>

      <button
        className="px-4 py-2 font-medium bg-white rounded border hover:border-blue-600 dark:bg-black/60 dark:text-white"
        onClick={(e) => {
          e.preventDefault();
          if (metadata && page + 1 < metadata.total_pages) {
            setPage(page + 1);
            window.scrollTo({top: 0, behavior: 'smooth'})
          }
        }}
      >
        Increase Page
      </button>
    </form>
  );


  return (
    // <main className='mt-[10vh] flex flex-col justify-center text-justify'>
    <main className='container'>
      <NavigationControls />

      {metadata ? (
        <div className="space-y-6">
          <h1 className="text-center text-3xl font-bold">{metadata.title || 'No Title'}</h1>
          <h2 className="text-center text-xl font-semibold">Author - {metadata.author || 'Unknown'}</h2>

          {chapter ? (
            <section className="space-y-4 px-4" key={chapter.title} ref={contentRef}>
              <h3 className="text-center text-lg font-semibold">File: {chapter.title}</h3>
              <h3 className="text-sm text-gray-500">Page: {page + 1} / {metadata.total_pages}</h3>
              <div dangerouslySetInnerHTML={{ __html: chapter.content }}></div>
            </section>
          ) : (
            <p className="text-center text-gray-500">No chapter loaded</p>
          )}
        </div>
      ) : (
        <p className="text-center text-gray-500">No epub file loaded</p>
      )}

      {single != true && <NavigationControls />}
    </main>
  );
}

export default App;
