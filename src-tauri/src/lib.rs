use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconEvent,
    AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

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
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to parse notes.json: {}. Backing up corrupted file.", e);
                // Backup corrupted file
                let backup = path.with_extension("json.bak");
                fs::copy(path, &backup).ok();
                AppData::default()
            }
        },
        Err(_) => AppData::default(),
    }
}

fn save_data(state: &AppState) {
    let data = state.data.lock().unwrap();
    if let Ok(json) = serde_json::to_string_pretty(&*data) {
        fs::write(&state.data_path, json).ok();
    }
}

// ─── Smart Positioning ───

/// Calculate a non-overlapping position for a new note using cascade with wrap-around.
fn find_next_position(app: &AppHandle, existing_notes: &[Note], note_width: f64, note_height: f64) -> (f64, f64) {
    const CASCADE_OFFSET_X: f64 = 30.0;
    const CASCADE_OFFSET_Y: f64 = 30.0;
    const BASE_X: f64 = 100.0;
    const BASE_Y: f64 = 100.0;
    const OVERLAP_THRESHOLD: f64 = 10.0;

    // Try to get actual screen size from primary monitor, fallback to conservative defaults
    let (max_x, max_y) = if let Ok(Some(monitor)) = app.primary_monitor() {
        let size = monitor.size();
        (size.width as f64 - 100.0, size.height as f64 - 100.0)
    } else {
        (1600.0, 900.0)
    };

    let mut candidate_x = BASE_X;
    let mut candidate_y = BASE_Y;
    let mut wrap_count = 0u32;

    // Try up to 50 cascade slots before giving up
    for _ in 0..50 {
        let overlaps = existing_notes.iter().any(|note| {
            (note.x - candidate_x).abs() < OVERLAP_THRESHOLD
                && (note.y - candidate_y).abs() < OVERLAP_THRESHOLD
        });

        if !overlaps {
            return (candidate_x, candidate_y);
        }

        candidate_x += CASCADE_OFFSET_X;
        candidate_y += CASCADE_OFFSET_Y;

        // Wrap around if off-screen, interleave with half-offset
        if candidate_x + note_width > max_x || candidate_y + note_height > max_y {
            wrap_count += 1;
            let offset = (wrap_count as f64) * CASCADE_OFFSET_X / 2.0;
            candidate_x = BASE_X + offset;
            candidate_y = BASE_Y + offset;
        }
    }

    (candidate_x, candidate_y)
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
        .min_inner_size(200.0, 120.0)
        .position(note.x, note.y)
        .decorations(false)
        .transparent(true)
        .always_on_top(note.pinned)
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
    let width = 280.0;
    let height = 320.0;

    // Smart position: avoid overlapping existing notes
    let (x, y) = {
        let data = state.data.lock().unwrap();
        find_next_position(&app, &data.notes, width, height)
    };

    let note = Note {
        id: uuid::Uuid::new_v4().to_string(),
        title: String::from("New Note"),
        content: String::new(),
        color: color.unwrap_or_else(|| "yellow".into()),
        x,
        y,
        width,
        height,
        pinned: true,
        collapsed: false,
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

// ─── Autostart Commands ───

#[tauri::command]
fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ─── Hide/Show All Notes ───

fn toggle_all_notes_visibility(app: &AppHandle) {
    let state: State<AppState> = app.state();
    let data = state.data.lock().unwrap();

    // Check if any note windows are currently visible
    let any_visible = data.notes.iter().any(|note| {
        let label = format!("note-{}", note.id);
        app.get_webview_window(&label)
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false)
    });

    if any_visible {
        // Hide all note windows
        for note in &data.notes {
            let label = format!("note-{}", note.id);
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.hide();
            }
        }
    } else {
        // Show all note windows
        for note in &data.notes {
            let label = format!("note-{}", note.id);
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.show();
                let _ = win.set_focus();
            } else {
                create_note_window(app, note).ok();
            }
        }
    }
}

// ─── Reminder Scheduler ───

fn start_reminder_scheduler(app: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(30));
            check_reminders(&app);
        }
    });
}

fn check_reminders(app: &AppHandle) {
    let state: State<AppState> = app.state();
    let now = chrono::Local::now();

    let mut data = state.data.lock().unwrap();
    let mut triggered_ids: Vec<(String, String)> = Vec::new(); // (note_id, title)

    for note in data.notes.iter() {
        if let Some(ref reminder) = note.reminder {
            if let Ok(reminder_time) = chrono::NaiveDateTime::parse_from_str(&reminder.time, "%Y-%m-%dT%H:%M") {
                let reminder_local = reminder_time.and_local_timezone(chrono::Local)
                    .single();

                if let Some(reminder_dt) = reminder_local {
                    let diff = now.signed_duration_since(reminder_dt);
                    // Fire if within the last 60 seconds (covers the 30s poll interval with margin)
                    if diff.num_seconds() >= 0 && diff.num_seconds() < 60 {
                        triggered_ids.push((note.id.clone(), note.title.clone()));
                    }
                }
            }
        }
    }

    // Update triggered reminders: advance repeat or clear
    for (ref id, _) in &triggered_ids {
        if let Some(note) = data.notes.iter_mut().find(|n| &n.id == id) {
            if let Some(ref reminder) = note.reminder.clone() {
                match reminder.repeat.as_str() {
                    "daily" => {
                        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(&reminder.time, "%Y-%m-%dT%H:%M") {
                            let next = t + chrono::Duration::days(1);
                            note.reminder = Some(Reminder {
                                time: next.format("%Y-%m-%dT%H:%M").to_string(),
                                repeat: reminder.repeat.clone(),
                            });
                        }
                    }
                    "weekly" => {
                        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(&reminder.time, "%Y-%m-%dT%H:%M") {
                            let next = t + chrono::Duration::weeks(1);
                            note.reminder = Some(Reminder {
                                time: next.format("%Y-%m-%dT%H:%M").to_string(),
                                repeat: reminder.repeat.clone(),
                            });
                        }
                    }
                    "weekday" => {
                        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(&reminder.time, "%Y-%m-%dT%H:%M") {
                            let mut next = t + chrono::Duration::days(1);
                            // Skip weekends
                            while next.weekday() == chrono::Weekday::Sat || next.weekday() == chrono::Weekday::Sun {
                                next += chrono::Duration::days(1);
                            }
                            note.reminder = Some(Reminder {
                                time: next.format("%Y-%m-%dT%H:%M").to_string(),
                                repeat: reminder.repeat.clone(),
                            });
                        }
                    }
                    _ => {
                        // "none" — clear the reminder after firing
                        note.reminder = None;
                    }
                }
            }
        }
    }
    drop(data);

    // Fire notifications and save
    if !triggered_ids.is_empty() {
        save_data(&state);

        for (id, title) in &triggered_ids {
            // System notification
            let _ = app.notification()
                .builder()
                .title("StickyNotes Reminder")
                .body(&format!("📋 {}", title))
                .show();

            // Also emit event to frontend so the note window can flash
            let _ = app.emit(&format!("reminder-fired-{}", id), ());
        }
    }
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

            // Init autostart plugin (default: disabled, user can toggle)
            app.handle().plugin(
                tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"]))
            )?;

            // Init global shortcut plugin
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new().build()
            )?;

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

            // Build tray icon with embedded icon
            let tray_icon_image = tauri::image::Image::from_path(
                app.path().resolve("icons/32x32.png", tauri::path::BaseDirectory::Resource)
                    .unwrap_or_else(|_| std::path::PathBuf::from("icons/32x32.png"))
            ).unwrap_or_else(|_| {
                tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                    .expect("failed to load tray icon")
            });

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(tray_icon_image)
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
                .build(app)?;

            // Restore existing notes on startup
            if has_notes {
                let state: State<AppState> = app.state();
                let notes = state.data.lock().unwrap().notes.clone();
                for note in &notes {
                    create_note_window(app.handle(), note).ok();
                }
            }

            // Start reminder scheduler (polls every 30s)
            start_reminder_scheduler(app.handle().clone());

            // Register global shortcuts
            let app_handle_new = app.handle().clone();
            let app_handle_toggle = app.handle().clone();

            // Ctrl+Shift+N — create new note
            let shortcut_new: Shortcut = "ctrl+shift+n".parse().expect("invalid shortcut");
            // Ctrl+Shift+H — hide/show all notes
            let shortcut_toggle: Shortcut = "ctrl+shift+h".parse().expect("invalid shortcut");

            app.global_shortcut().on_shortcut(shortcut_new, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let state: State<AppState> = app_handle_new.state();
                    let _ = create_note(app_handle_new.clone(), state, None);
                }
            })?;

            app.global_shortcut().on_shortcut(shortcut_toggle, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_all_notes_visibility(&app_handle_toggle);
                }
            })?;

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
            get_autostart_enabled,
            set_autostart_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
