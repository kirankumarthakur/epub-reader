import { useState, useEffect, useRef } from "react"
import "./App.css"
import { invoke } from "@tauri-apps/api/core"
import { open } from "@tauri-apps/plugin-dialog"
import { Box, Button, Center, CloseButton, Dialog, Drawer, Flex, For, Heading, Input, Kbd, Link, Popover, Portal, Stack, Text} from "@chakra-ui/react"
import { Global } from "@emotion/react"
import { useSwipeable } from "react-swipeable"

type epubBook = {
  book_url: string
  cover_url: string
  crc: number
  title: string
  author: string
  current_page: number
  total_pages: number
}

type Toc = [string, string][]

type pageContent = {
  title: string
  mime: string
  content: string
}

const App = () => {
  const [book, setBook] = useState<epubBook | null>(null)
  const [toc, setToc] = useState<Toc | null>(null)
  const [pickerPath, setPickerPath] = useState<string | null>(null)
  const [currentContent, setCurrentContent] = useState<pageContent | null>(null)
  const [showError, setShowError] = useState(false)
  const [errorMessage, setErrorMessage] = useState("")
  const currentPageRef = useRef<number>(1);
  const [inputValue, setInputValue] = useState("")

  useEffect(() => {
    currentPageRef.current = book?.current_page ?? 1
  }, [book])

  useEffect(() => {
    const processNewBook = async () => {
      if (pickerPath) {
        try {
          const newBook = await invoke<epubBook>("import_epub", { path: pickerPath })

          const initialPage = newBook.current_page

          let content: pageContent | null = null
          try {
            content = await invoke<pageContent>("get_page", { page: initialPage })
          } catch (error) {
            console.error("Failed to load page content for new book:", error)
            setErrorMessage("Failed to load book content. The file might be corrupted.")
            setShowError(true)
            content = {
              title: "Error",
              mime: "text/plain",
              content: "<p>Could not load page content.</p>"
            }
          }

          setBook(newBook)
          setToc(await invoke<Toc>("get_toc", {}))
          setCurrentContent(content)
          setPickerPath(null)
        } catch (error) {
          console.error("Failed to import book:", error)
          setErrorMessage("Failed to import book. Please ensure it's a valid EPUB file and try again.")
          setShowError(true)
        }
      }
    }
    processNewBook()
  }, [pickerPath])

  const filePicker = async (): Promise<boolean> => {
    try {
      const new_path = await open({
        multiple: false,
        filters: [{ name: "Epub Files", extensions: ["epub"] }]
      })
      if (typeof new_path === "string") {
        setPickerPath(new_path)
        return true
      }
      return false
    } catch (error) {
      console.error("File picker error:", error)
      setErrorMessage("File selection was cancelled or failed.")
      setShowError(true)
      return false
    }
  }

  const goToPage = async (newPage: number) => {
    if (!book) return

    const page = Math.max(1, Math.min(newPage, book.total_pages))
    if (page === currentPageRef.current && currentContent != null) {
      return
    }

    let content: pageContent | null = null
    try {
      content = await invoke<pageContent>("get_page", { page })
    } catch (error) {
      console.error("Failed to load page content:", error)
      setErrorMessage("Failed to load page content. The book might be corrupted or the page does not exist.")
      setShowError(true)
      content = {
        title: "Error",
        mime: "text/plain",
        content: "<p>Could not load page content.</p>"
      }
    }

    currentPageRef.current = page
    setBook(prevBook =>
      prevBook
        ? {
            ...prevBook,
            current_page: page
          }
        : null
    )

    setCurrentContent(content)

    invoke("set_last_page", { crc: book.crc, page }).catch(error => {
      console.error("Failed to save last page:", error)
      setErrorMessage("Failed to save reading progress.")
      setShowError(true)
    })
  }

  useEffect(() => {
    const handleClick = async (event: MouseEvent) => {
      const target = event.target as HTMLElement
      if (target.tagName === 'A') {
        const href = (target as HTMLAnchorElement).href
        console.log(href)
        const parsed = new URL(href)
        const idref = parsed.pathname.substring(1)
        event.preventDefault()
        const page = await invoke<number>('get_page_from_idref', { idref })
        goToPage(page);
      }
    };

    const handleKey = (event: KeyboardEvent) => {
      const tag = (event.target as HTMLElement).tagName.toLowerCase();

      if (tag === 'input' || tag === 'textarea') return;
      const key = event.key.toLowerCase();

      const current_page = currentPageRef.current
      console.log(current_page)
      if (key === 'arrowleft' || key === 'p') {
        goToPage(current_page - 1)
      } else if (key === 'arrowright' || key === 'n') {
        goToPage(current_page + 1)
      } else if (key === 't') {
        const tocButton = document.getElementById('toc-button');
        if (tocButton) {
          tocButton.click(); 
        }
      } else if (key == 'i') {
        const importbutton = document.getElementById('import-button');
        if (importbutton) {
          importbutton.click(); 
        }
      } else if (key == 'j') {
        const jumpbutton = document.getElementById('jump-button');
        if (jumpbutton) {
          jumpbutton.click(); 
        }
      }
    }

    document.addEventListener('click', handleClick);
    document.addEventListener('keydown', handleKey);

    return () => {
      document.removeEventListener('click', handleClick);
      document.removeEventListener('keydown', handleKey);
    };
  }, []);

  const GlobalEPUBStyles = () => (
    <Global
      styles={{
        '.epub-content': {
          fontFamily: 'Roboto, Inter, Avenir, Helvetica, Arial, sans-serif',
          lineHeight: '1.6',
          color: '#dddddd', 
          fontSize: '16px',
          lineHeightStep: '24px',
        },

        '.epub-content h1': {
          fontSize: '2rem',
          fontWeight: 'bold',
          marginBottom: '1rem',
          textAlign: 'center',
        },

        '.epub-content h2': {
          fontSize: '1.5rem',
          fontWeight: 'bold',
          marginBottom: '0.75rem',
          marginTop: '1.5rem',
        },

        '.epub-content h3': {
          fontSize: '1.25rem',
          fontWeight: 'semibold',
          marginBottom: '0.5rem',
          marginTop: '1.25rem',
        },

        '.epub-content p': {
          fontSize: '1rem',
          marginBottom: '1rem',
        },

        '.epub-content ul': {
          paddingLeft: '1.25rem',
          marginBottom: '1rem',
        },

        '.epub-content li': {
          marginBottom: '0.5rem',
        },

        '.epub-content a': {
          color: '#3182CE', 
          textDecoration: 'underline',
          cursor: 'pointer',
        },

        '.epub-content blockquote': {
          borderLeft: '4px solid #CBD5E0', 
          paddingLeft: '1rem',
          color: '#4A5568', 
          fontStyle: 'italic',
          marginBottom: '1rem',
        },

        '.epub-content img': {
          maxWidth: '100%',
          height: 'auto',
          margin: '1rem auto',
          display: 'block',
        },

        '.epub-content strong': {
          fontWeight: 'bold',
        },

        '.epub-content em': {
          fontStyle: 'italic',
        },
      }}
    />
  );

  const handlers = useSwipeable({
    onSwipedLeft: () => goToPage((book?.current_page ?? 1) + 1),
    onSwipedRight: () => goToPage((book?.current_page ?? 1) - 1),
    trackTouch: true,
    trackMouse: true
  })

  return (
    <div>
      {showError && (
        <Dialog.Root open={showError} onOpenChange={(_) => setShowError(false)}>
          <Portal>
            <Dialog.Backdrop />
            <Dialog.Positioner>
              <Dialog.Content>
                <Dialog.Context>
                  {(store) => (
                    <Dialog.Body pt="6" spaceY="3">
                      <strong style={{ fontSize: '1.2rem' }}>Error!</strong>
                      <p>{errorMessage}</p>
                      <Button
                        size="sm"
                        variant="outline"
                        colorPalette="red"
                        onClick={() => store.setOpen(false)}
                      >
                        Dismiss
                      </Button>
                    </Dialog.Body>
                  )}
                </Dialog.Context>
                <Dialog.CloseTrigger asChild>
                  <CloseButton size="sm" />
                </Dialog.CloseTrigger>
              </Dialog.Content>
            </Dialog.Positioner>
          </Portal>
        </Dialog.Root>
      )}

      <GlobalEPUBStyles />

      <Flex as="header" justifyContent='center' gap={4} mb={2} px={2} py={2}>
        <Button colorPalette='teal' variant='outline' size='sm' id='import-button' onClick={filePicker} >
          <Kbd variant="outline">I</Kbd>
          Import Book 
        </Button>

        <Drawer.Root>
          <Drawer.Trigger asChild>
            <Button colorPalette="teal" variant="outline" size="sm" id="toc-button">
              <Kbd variant="outline">T</Kbd>
              Table of Contents
            </Button>
          </Drawer.Trigger>
          <Portal>
            <Drawer.Backdrop />
            <Drawer.Positioner padding="4">
              <Drawer.Content rounded="md">
                <Drawer.Header>
                  <Drawer.Title>Table of Contents</Drawer.Title>
                </Drawer.Header>
                <Drawer.Body>
                  <Stack>
                    <For each={toc ?? []} >
                      {(item, _) => (
                        <Link variant="plain" href={item[1]}> {item[0]} </Link>
                      )}
                    </For>
                  </Stack>
                </Drawer.Body>
              </Drawer.Content>
            </Drawer.Positioner>
          </Portal>
        </Drawer.Root>

        <Popover.Root>
          <Popover.Trigger asChild>
            <Button size="sm" variant="outline" colorPalette='teal' id='jump-button'>
              <Kbd variant="outline">J</Kbd>
              Jump To
            </Button>
          </Popover.Trigger>
          <Portal>
            <Popover.Positioner minW='unset'>
              <Popover.Content maxW='200px'>
                <Popover.Body>
                  <Input 
                  type='number'
                  min={1}
                  max={book?.total_pages}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  placeholder={`Jump to (1-${book?.total_pages})`} size="sm"
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      const page = Math.max(1, Math.min(Number(inputValue), book?.total_pages ?? 1));
                      goToPage(page);
                    }
                  }}/>
                </Popover.Body>
              </Popover.Content>
            </Popover.Positioner>
          </Portal>
        </Popover.Root>
      </Flex>

      <main>
          <Box>
            {book ? (
              <>
                <Flex direction="column" mb={4} alignItems='center'>
                  <Heading as="h2" size="md" mb={2}>
                    {book.title}
                  </Heading>
                  <Text fontSize="sm" color="gray.400">
                    <Kbd variant="outline">P</Kbd>
                    Page {book.current_page} of {book.total_pages}
                    <Kbd variant="outline">N</Kbd>
                  </Text>
                </Flex>

                <Box {...handlers} w="100%" h="100%">
                <Box
                  margin='5%'
                  className='epub-content'
                  dangerouslySetInnerHTML={{
                    __html: currentContent?.content || "<p>No Content</p>",
                  }}
                />
                </Box>
            </>
            ) : (
              <Text>No book loaded.</Text>
            )}
          </Box>
      </main>
    </div>
  )
}

export default App
