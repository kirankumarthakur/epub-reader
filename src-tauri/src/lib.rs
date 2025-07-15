use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use epub::doc::EpubDoc;
use sqlx::{
    sqlite::SqlitePoolOptions,
    Pool, Row, Sqlite,
};
use tauri::{App, Manager, State};
use tauri_plugin_sql::{Migration, MigrationKind};

type Db = Pool<Sqlite>;

struct AppState {
    db: Db,
    epub: Mutex<Option<EpubBook>>,
}

struct EpubBook {
    pages: Vec<String>,
    // 0 -> idref, 1 -> Chapter Title, 2 -> mime, 3 -> content
    resources: HashMap<String, (String, String, String)>,
    // paths are not trimmed here, some epubs they are some the aren't it's confusing
    paths: HashMap<PathBuf, String>,
    crc: i64,
}

#[derive(serde::Serialize)]
struct Book {
    cover_url: String,
    book_url: String,
    crc: i64,
    title: String,
    author: String,
    current_page: i64,
    total_pages: i64,
}

#[derive(serde::Serialize)]
struct PageContent {
    title: String,
    mime: String,
    content: String,
}

static APPDATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: "create table if not exists books (
                id integer primary key autoincrement,
                name text not null,
                author text not null,
                crc integer not null,
                current_page integer default 0,
                total_pages integer default 0,
                created_at text default current_timestamp,
                book_url text not null,
                cover_url text
            );
            ",
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .setup(|app| {
            let datadir = app.path().app_data_dir().unwrap();
            let db_path = app.path().app_data_dir().unwrap().join("epub-reader.db");

            let _ = APPDATA_DIR.set(datadir);
            let _ = DB_PATH.set(db_path);

            init_data().unwrap();
            tauri::async_runtime::block_on(async move {
                let db = setup_db().await;

                app.manage(AppState {
                    db,
                    epub: Mutex::new(None),
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:epub-reader.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            import_epub, 
            get_page,
            get_library,
            get_toc,
            get_last_page,
            set_last_page,
            get_page_from_idref,
            get_idref_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, PartialEq)]
enum DataFolder {
    NEW,
    LOADED,
}

fn init_data() -> io::Result<DataFolder> {
    let base_path = APPDATA_DIR.get().expect("appdata directory not set");
    let paths = [
        base_path.to_path_buf(),
        base_path.join("library"),
        // base_path.join("fonts"),
        // base_path.join("settings.json"),
        // base_path.join("themes.json"),
        // base_path.join("fonts").join("fonts.json"),
    ];
    let mut created_any = false;
    for path in paths.iter() {
        if !path.exists() {
            created_any = true;

            if path.extension().is_some() {
                fs::write(path, "{}")?;
            } else {
                fs::create_dir_all(path)?;
            }
        }
    }
    Ok(if created_any {
        DataFolder::NEW
    } else {
        DataFolder::LOADED
    })
}

async fn setup_db() -> Db {
    let path = DB_PATH.get().unwrap();
    let db = SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .expect(&format!("failed to connect to db at {}", path.display()));
    db
}

#[tauri::command]
async fn import_epub(path: &str, state: State<'_, AppState>) -> Result<Book, String> {
    let filepath = Path::new(&path);
    let mut f = File::open(&path).map_err(|_| format!("cannot access: {}", filepath.display()))?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    let checksum = crc32fast::hash(&buffer) as i64;
    let db = &state.db;
    let rows = sqlx::query("select * from books where crc = ?1 limit 1")
        .bind(checksum)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;
    let stem = filepath.file_stem().unwrap().to_str().unwrap();
    let file_name = filepath.file_name().unwrap().to_str().unwrap();
    let bookfolder = APPDATA_DIR
        .get()
        .unwrap()
        .clone()
        .join("library")
        .join(stem);
    let doc = bookfolder.join(file_name);
    let cover_path = bookfolder.join("cover.jpg");
    if let Some(row) = rows {
        let title: String = row.get("name");
        let author: String = row.get("author");
        let current_page: i64 = row.get("current_page");
        let total_pages: i64 = row.get("total_pages");
        let mut book =
            EpubDoc::new(&doc).map_err(|e| format!("Error: failed to open EPUB: {}", e))?;
        let path_to_idref: HashMap<PathBuf, String> = book
            .resources
            .iter()
            .map(|(idref, (res_path, _mime))| (res_path.clone(), idref.clone()))
            .collect();
        let mut idref_to_title: HashMap<String, String> = HashMap::new();
        for item in book.toc.iter() {
            if let Some(idref) = path_to_idref.get(&item.content) {
                idref_to_title.insert(idref.clone(), item.label.clone());
            }
        }
        let pages: Vec<String> = book.spine.iter().map(|s| s.idref.clone()).collect();
        let mut resources: HashMap<String, (String, String, String)> = HashMap::new();
        for idref in &pages {
            let mime_option = book
                .resources
                .get(idref)
                .map(|(_res_path, mime)| mime.clone())
                .unwrap();
            let res_title = idref_to_title
                .get(idref)
                .cloned()
                .unwrap_or_else(|| idref.clone());
            if let Some((data, _)) = book.get_resource_str(idref) {
                resources.insert(idref.clone(), (res_title, mime_option, data));
            } else {
                resources.insert(idref.clone(), ("".to_string(), "".to_string(), "".to_string()));
            }
        }
        let epubbook = EpubBook {
            pages,
            resources,
            paths: path_to_idref,
            crc: checksum,
        };
        let mut lock = state.epub.lock().expect("failed to acquire lock");
        *lock = Some(epubbook);
        Ok(Book {
            cover_url: if cover_path.exists() {
                cover_path.to_str().unwrap_or("").to_string()
            } else {
                "".to_string()
            },
            book_url: doc.to_str().unwrap_or("").to_string(),
            crc: checksum,
            title,
            author,
            current_page,
            total_pages,
        })
    } else {
        let extension = filepath.extension().unwrap().to_str().unwrap();
        let isepub = file_name.contains(".epub") && extension == "epub";
        if !isepub {
            return Err("Error: Book extension does not contain epub".to_string());
        }
        std::fs::create_dir_all(&bookfolder).map_err(|e| e.to_string())?;
        std::fs::write(&doc, &buffer).map_err(|e| e.to_string())?;
        std::fs::write(bookfolder.join("checksum.txt"), checksum.to_string())
            .map_err(|e| e.to_string())?;
        let mut book =
            EpubDoc::new(&doc).map_err(|e| format!("Error: failed to parse {}, {}", &stem, e))?;
        let title = book.mdata("title").unwrap_or(stem.to_string());
        let author = book.mdata("creator").unwrap_or("<unknown>".to_string());
        let mut cover_exists = true;
        match book.get_cover() {
            Some((cover_data, _)) => {
                let mut f = File::create(bookfolder.join("cover.jpg")).unwrap();
                let _ = f.write_all(&cover_data);
            }
            None => {
                cover_exists = false;
                println!("Info: Book does not have a cover");
            }
        }
        let path_to_idref: HashMap<PathBuf, String> = book
            .resources
            .iter()
            .map(|(idref, (res_path, _mime))| (res_path.clone(), idref.clone()))
            .collect();
        let mut idref_to_title: HashMap<String, String> = HashMap::new();
        for item in book.toc.iter() {
            if let Some(idref) = path_to_idref.get(&item.content) {
                idref_to_title.insert(idref.clone(), item.label.clone());
            }
        }
        let pages: Vec<String> = book.spine.iter().map(|s| s.idref.clone()).collect();
        let mut resources: HashMap<String, (String, String, String)> = HashMap::new();
        for idref in &pages {
            let mime_option = book
                .resources
                .get(idref)
                .map(|(_res_path, mime)| mime.clone())
                .unwrap();
            let res_title = idref_to_title
                .get(idref)
                .cloned()
                .unwrap_or_else(|| idref.clone());
            if let Some((data, _)) = book.get_resource_str(idref) {
                resources.insert(idref.clone(), (res_title, mime_option, data));
            } else {
                eprintln!(
                    "Warning: Could not get resource data for idref: {} (new book)",
                    idref
                );
            }
        }
        let total_pages = pages.len() as i64;
        let epubbook = EpubBook {
            pages,
            resources,
            paths: path_to_idref,
            crc: checksum,
        };

        let cover_url = if cover_exists {
            bookfolder.join("cover.jpg").to_str().unwrap().to_string()
        } else {
            "".to_string()
        };

        sqlx::query("insert into books(name, author, crc, current_page, total_pages, book_url, cover_url) values (?1, ?2, ?3, ?4, ?5, ?6, ?7);")
            .bind(title.clone())
            .bind(author.clone())
            .bind(checksum)
            .bind(1)
            .bind(total_pages)
            .bind(doc.to_str().unwrap().to_string())
            .bind(cover_url.clone())
            .execute(db)
            .await .map_err(|x| x.to_string())?;

        let mut lock = state.epub.lock().expect("failed to acquire lock");
        *lock = Some(epubbook);
        Ok(Book {
            cover_url,
            book_url: doc.to_str().unwrap().to_string(),
            crc: checksum,
            title,
            author,
            current_page: 1,
            total_pages,
        })
    }
}

#[tauri::command]
async fn get_page(page: usize, state: State<'_, AppState>) -> Result<PageContent, String> {
    let (title, content, mime, crc) = {
        let lock = state
            .epub
            .lock()
            .map_err(|_| "Failed to lock EPUB".to_string())?;
        let epub = lock.as_ref().ok_or("No book is currently loaded")?;

        if page > epub.pages.len() || page < 1 {
            return Err("Page out of bounds".to_string());
        }

        let idref = &epub.pages[page - 1];
        let (title, mime, content) = epub
            .resources
            .get(idref)
            .ok_or_else(|| format!("Failed to get content for idref: {}", idref))?;

        (title.clone(), content.clone(), mime.clone(), epub.crc)
    };

    sqlx::query("UPDATE books SET current_page = ?1 WHERE crc = ?2")
        .bind(page as i64)
        .bind(crc)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(PageContent {
        title,
        mime,
        content,
    })
}

#[tauri::command]
async fn get_toc(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let mut res = Vec::new();
    let lock = state.epub.lock().map_err(|e| e.to_string())?;
    let epub = lock.as_ref().ok_or("No book is currently loaded")?;

    for page in &epub.pages {
        let (title, _, _) = epub.resources.get(page).ok_or_else(|| format!("Failed to get content for idref {}", page))?;
        res.push((title.clone(), page.clone()));
    }
    Ok(res)
}

#[tauri::command]
async fn get_library(state: State<'_, AppState>) -> Result<Vec<Book>, String> {
    let mut res = Vec::new();
    let rows = sqlx::query("select * from books")
        .fetch_all(&state.db)
        .await;

    if let Ok(rows) = rows {
        for row in rows.iter() {
            let title: String = row.get("name");
            let author: String = row.get("author");
            let current_page: i64 = row.get("current_page");
            let total_pages: i64 = row.get("total_pages");
            let book_url: String = row.get("book_url");
            let cover_url: String = row.get("cover_url");
            let crc: i64 = row.get("crc");

            res.push(Book {
                title,
                cover_url,
                book_url,
                crc,
                author,
                current_page,
                total_pages
            });
        }
    }

    Ok(res)
}

#[tauri::command]
async fn get_last_page(crc: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let row = sqlx::query("select current_page from books where crc = ?1 limit 1")
        .bind(crc)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = row  {
        let page: i64 = row.get("current_page");
        return Ok(page);
    }
    Ok(1)
}

#[tauri::command]
async fn set_last_page(crc: i64, page: i64, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query("UPDATE books SET current_page = ?1 WHERE crc = ?2")
        .bind(page)
        .bind(crc)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_page_from_idref(idref: &str, state: State<'_, AppState>) -> Result<i64, String> {
    let lock = state.epub.lock().map_err(|e| e.to_string())?;
    let epub = lock.as_ref().ok_or("No book is currently loaded")?;

    Ok(1 + epub.pages.iter().position(|x| x == idref).unwrap_or(1) as i64)
}

#[tauri::command]
async fn get_idref_from_path(path: &str, state: State<'_, AppState>) -> Result<String, String> {
    let lock = state.epub.lock().map_err(|e| e.to_string())?;
    let epub = lock.as_ref().ok_or("No book is currently loaded")?;

    if let Some(idref) = epub.paths.get(&PathBuf::from(path)) {
        return Ok(idref.to_string());
    }
    Err("Unknown Idref".to_string())
}