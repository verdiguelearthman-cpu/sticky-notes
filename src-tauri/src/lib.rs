use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconEvent,
    AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder,
};

// ─── Data Model ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub pinned: bool,
    pub collapsed: bool,
    pub opacity: f64,
    pub reminder: Option<Reminder>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub time: String,
    pub repeat: String, // "none" | "daily" | "weekly" | "weekday"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub notes: Vec<Note>,
}

impl Default for AppData {
    fn default() -> Self {
        Self { notes: Vec::new() }
    }
}

pub struct AppState {
    pub data: Mutex<AppData>,
    pub data_path: PathBuf,
}

// ─── Persistence ───

fn get_data_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&dir).ok();
    dir.join("notes.json")
}

fn load_data(path: &PathBuf) -> AppData {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppData::default(),
    }
}

fn save_data(state: &AppState) {
    let data = state.data.lock().unwrap();
    if let Ok(json) = serde_json::to_string_pretty(&*data) {
        fs::write(&state.data_path, json).ok();
    }
}

// ─── Window Management ───

fn create_note_window(app: &AppHandle, note: &Note) -> Result<(), String> {
    let label = format!("note-{}", note.id);

    // If window already exists, focus it
    if let Some(win) = app.get_webview_window(&label) {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let url = WebviewUrl::App(format!("index.html?noteId={}", note.id).into());

    let builder = WebviewWindowBuilder::new(app, &label, url)
        .title("")
        .inner_size(note.width, note.height)
        .position(note.x, note.y)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .visible(true);

    builder.build().map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Tauri Commands ───

#[tauri::command]
fn get_notes(state: State<AppState>) -> Vec<Note> {
    state.data.lock().unwrap().notes.clone()
}

#[tauri::command]
fn get_note(state: State<AppState>, id: String) -> Option<Note> {
    state
        .data
        .lock()
        .unwrap()
        .notes
        .iter()
        .find(|n| n.id == id)
        .cloned()
}

#[tauri::command]
fn create_note(app: AppHandle, state: State<AppState>, color: Option<String>) -> Result<Note, String> {
    let now = chrono::Local::now().to_rfc3339();
    let note = Note {
        id: uuid::Uuid::new_v4().to_string(),
        title: String::from("New Note"),
        content: String::new(),
        color: color.unwrap_or_else(|| "yellow".into()),
        x: 200.0,
        y: 150.0,
        width: 280.0,
        height: 320.0,
        pinned: true,
        collapsed: false,
        opacity: 1.0,
        reminder: None,
        created_at: now.clone(),
        updated_at: now,
    };

    {
        let mut data = state.data.lock().unwrap();
        data.notes.push(note.clone());
    }
    save_data(&state);
    create_note_window(&app, &note)?;
    Ok(note)
}

#[tauri::command]
fn update_note(state: State<AppState>, note: Note) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if let Some(existing) = data.notes.iter_mut().find(|n| n.id == note.id) {
        *existing = Note {
            updated_at: chrono::Local::now().to_rfc3339(),
            ..note
        };
    }
    drop(data);
    save_data(&state);
    Ok(())
}

#[tauri::command]
fn delete_note(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    // Close the window
    let label = format!("note-{}", id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }

    // Remove from data
    let mut data = state.data.lock().unwrap();
    data.notes.retain(|n| n.id != id);
    drop(data);
    save_data(&state);
    Ok(())
}

#[tauri::command]
fn update_note_position(state: State<AppState>, id: String, x: f64, y: f64) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if let Some(note) = data.notes.iter_mut().find(|n| n.id == id) {
        note.x = x;
        note.y = y;
        note.updated_at = chrono::Local::now().to_rfc3339();
    }
    drop(data);
    save_data(&state);
    Ok(())
}

#[tauri::command]
fn update_note_size(state: State<AppState>, id: String, width: f64, height: f64) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if let Some(note) = data.notes.iter_mut().find(|n| n.id == id) {
        note.width = width;
        note.height = height;
        note.updated_at = chrono::Local::now().to_rfc3339();
    }
    drop(data);
    save_data(&state);
    Ok(())
}

#[tauri::command]
fn show_all_notes(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let data = state.data.lock().unwrap();
    for note in &data.notes {
        create_note_window(&app, note).ok();
    }
    Ok(())
}

// ─── App Entry ───

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Init logging in debug
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Init notification plugin
            app.handle().plugin(tauri_plugin_notification::init())?;

            // Load data
            let data_path = get_data_path(app.handle());
            let data = load_data(&data_path);
            let has_notes = !data.notes.is_empty();

            let state = AppState {
                data: Mutex::new(data),
                data_path,
            };
            app.manage(state);

            // Build tray menu
            let new_note = MenuItemBuilder::with_id("new_note", "New Note")
                .build(app)?;
            let show_all = MenuItemBuilder::with_id("show_all", "Show All Notes")
                .build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&new_note)
                .item(&show_all)
                .separator()
                .item(&quit)
                .build()?;

            // Build tray icon
            let _tray = tauri::tray::TrayIconBuilder::new(app)
                .menu(&menu)
                .tooltip("StickyNotes")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "new_note" => {
                            let state: State<AppState> = app.state();
                            let _ = create_note(app.clone(), state, None);
                        }
                        "show_all" => {
                            let state: State<AppState> = app.state();
                            let _ = show_all_notes(app.clone(), state);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        // Double-click tray to create new note
                        let app = _tray.app_handle();
                        let state: State<AppState> = app.state();
                        let _ = create_note(app.clone(), state, None);
                    }
                })
                .build()?;

            // Restore existing notes on startup
            if has_notes {
                let state: State<AppState> = app.state();
                let notes = state.data.lock().unwrap().notes.clone();
                for note in &notes {
                    create_note_window(app.handle(), note).ok();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_notes,
            get_note,
            create_note,
            update_note,
            delete_note,
            update_note_position,
            update_note_size,
            show_all_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
