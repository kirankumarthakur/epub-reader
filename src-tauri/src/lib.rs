use std::{collections::HashMap, sync::Mutex};

use epub::doc::EpubDoc;
use tauri::State;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BookState{
            epub: std::sync::Mutex::new(None),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_epub, get_page, get_metadata])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Clone)]
struct EpubBook {
    title: Option<String>,
    author: Option<String>,
    pages: HashMap<PageRef, String>,
    total_pages: usize,
    current_page: usize,
    toc: Vec<PageRef>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EpubMetadata {
    title: String,
    author: String,
    total_pages: usize,
    current_page: usize
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PageRef(String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Page {
    title: String,
    content: String,
}

struct BookState {
    epub: Mutex<Option<EpubBook>>
}

#[tauri::command]
fn load_epub(path: &str, state: tauri::State<BookState>) -> EpubMetadata {
    dbg!("loading book from path");
    dbg!(path);
    let mut book = EpubDoc::new(&path).expect("failed to open epub");
    let title = book.mdata("title");
    let author = book.mdata("creator");

    let pagerefs = book
        .spine
        .iter().map(|item| PageRef(item.idref.clone()))
        .collect::<Vec<_>>();

    let toc = pagerefs.clone();

    let pages: HashMap<PageRef, String> = pagerefs
        .into_iter()
        .filter_map(|id| {
            book.get_resource(&id.0).and_then(|(data, _)| {
                Some((id, unsafe { String::from_utf8_unchecked(data) }))
            })
        })
        .collect();

    let total_pages = pages.len();

    let metadata = EpubBook {
        title: title.clone(),
        author: author.clone(),
        pages,
        total_pages,
        current_page: 0,
        toc
    };

    let mut lock = state.epub.lock().expect("failed to acquire lock");
    *lock = Some(metadata);

    EpubMetadata { 
        title: title.unwrap_or("No Title".to_string()), 
        author: author.unwrap_or("No Author".to_string()), 
        total_pages, 
        current_page: 0 
    }
}

#[tauri::command]
fn get_page(page: usize, state: State<BookState>) -> Page {
    dbg!("loading chapter from page = ");
    dbg!(page);
    let mut lock = state.epub.lock().unwrap();
    let metadata = lock.as_mut().expect("epub not loaded");
    let pageref = &metadata.toc[page];
    if let Some(res) = metadata.pages.get(pageref) {
        metadata.current_page = page;
        return Page {
            title: pageref.0.clone(),
            content: res.clone()
        };
    } 

    Page {
        title: "No Title".to_string(),
        content: "No Content".to_string(),
    }
}

#[tauri::command]
fn get_metadata(state: State<BookState>) -> EpubMetadata {
    dbg!("loading metadata");
    let lock = state.epub.lock().unwrap();
    let metadata = lock.as_ref().expect("epub not loaded");

    EpubMetadata { 
        title: metadata.title.clone().unwrap_or("No Title".to_string()), 
        author: metadata.author.clone().unwrap_or("No Author".to_string()), 
        total_pages: metadata.total_pages, 
        current_page: metadata.current_page, 
    }
}