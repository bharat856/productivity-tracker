mod core;
mod db;
mod tracker;

use chrono::{Duration, Local};
use db::Database;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, State,
};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: Arc<Mutex<Database>>,
    pub(crate) paused: Arc<Mutex<bool>>,
}

#[derive(Serialize)]
struct UsageRow {
    key: String,
    label: String,
    seconds: i64,
    category: String,
    domain: Option<String>,
}
#[derive(Serialize)]
struct Dashboard {
    total_seconds: i64,
    productive_seconds: i64,
    distracting_seconds: i64,
    neutral_seconds: i64,
    apps: Vec<UsageRow>,
    browsers: Vec<UsageRow>,
    websites: Vec<UsageRow>,
    daily: Vec<DayRow>,
    focus_sessions: i64,
}
#[derive(Serialize)]
struct DayRow {
    day: String,
    seconds: i64,
}

fn db_path(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("daymark.sqlite")
}

fn category_for(conn: &rusqlite::Connection, source: &str, key: &str) -> String {
    conn.query_row(
        "SELECT category FROM rules WHERE source=?1 AND key=?2",
        [source, key],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| "neutral".to_string())
}

#[tauri::command]
fn get_dashboard(state: State<AppState>) -> Result<Dashboard, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = Local::now().timestamp();
    let from = (Local::now() - Duration::days(1)).timestamp();
    let mut query = db.conn.prepare("SELECT source,key,label,COALESCE(domain,''),started_at,ended_at FROM intervals WHERE ended_at>?1 AND started_at<?2")
        .map_err(|e| e.to_string())?;
    let rows = query
        .query_map([from, now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut intervals = Vec::new();
    let mut labels = HashMap::new();
    let mut domains = HashMap::new();
    for row in rows {
        let (source, key, label, domain, start, end) = row.map_err(|e| e.to_string())?;
        intervals.push(core::Interval {
            key: key.clone(),
            source: source.clone(),
            start,
            end,
        });
        labels.insert((source.clone(), key.clone()), label);
        if !domain.is_empty() {
            domains.insert((source, key), domain);
        }
    }

    let totals = core::aggregate(&intervals, from, now);
    let mut total = 0;
    let mut productive = 0;
    let mut distracting = 0;
    let mut neutral = 0;
    let mut apps = Vec::new();
    let mut websites = Vec::new();
    for ((source, key), seconds) in totals {
        let category = category_for(&db.conn, &source, &key);
        let label = labels
            .get(&(source.clone(), key.clone()))
            .cloned()
            .unwrap_or_else(|| key.clone());
        let item = UsageRow {
            key: key.clone(),
            label,
            seconds,
            category: category.clone(),
            domain: domains.get(&(source.clone(), key.clone())).cloned(),
        };
        if source == "website" {
            websites.push(item);
            continue;
        }
        total += seconds;
        match category.as_str() {
            "productive" => productive += seconds,
            "distracting" => distracting += seconds,
            _ => neutral += seconds,
        }
        apps.push(item);
    }
    apps.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    websites.sort_by(|a, b| b.seconds.cmp(&a.seconds));
    let daily = (0..7)
        .map(|i| DayRow {
            day: (Local::now() - Duration::days(6 - i))
                .format("%a")
                .to_string(),
            seconds: if i == 6 { total } else { 0 },
        })
        .collect();
    let browsers = apps
        .iter()
        .filter(|item| item.key == "Brave Browser")
        .map(|item| UsageRow {
            key: item.key.clone(),
            label: item.label.clone(),
            seconds: item.seconds,
            category: item.category.clone(),
            domain: None,
        })
        .collect();
    Ok(Dashboard {
        total_seconds: total,
        productive_seconds: productive,
        distracting_seconds: distracting,
        neutral_seconds: neutral,
        apps,
        browsers,
        websites,
        daily,
        focus_sessions: 0,
    })
}

#[tauri::command]
fn set_category(
    state: State<AppState>,
    key: String,
    category: String,
    source: String,
) -> Result<(), String> {
    if !matches!(category.as_str(), "productive" | "neutral" | "distracting") {
        return Err("invalid category".into());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.conn.execute("INSERT INTO rules(source,key,category) VALUES(?1,?2,?3) ON CONFLICT(source,key) DO UPDATE SET category=excluded.category", [&source, &key, &category]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_tracking_paused(state: State<AppState>, paused: bool) -> Result<(), String> {
    *state.paused.lock().map_err(|e| e.to_string())? = paused;
    Ok(())
}

#[tauri::command]
fn show_pet(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("pet")
        .ok_or_else(|| "pet window missing".to_string())?
        .show()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_pet(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("pet")
        .ok_or_else(|| "pet window missing".to_string())?
        .hide()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_data(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.conn.prepare("SELECT source,key,label,domain,started_at,ended_at,idle FROM intervals ORDER BY started_at").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok(serde_json::json!({ "source": row.get::<_, String>(0)?, "key": row.get::<_, String>(1)?, "label": row.get::<_, String>(2)?, "domain": row.get::<_, Option<String>>(3)?, "startedAt": row.get::<_, i64>(4)?, "endedAt": row.get::<_, i64>(5)?, "idle": row.get::<_, i64>(6)? != 0 }))).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    for row in rows {
        data.push(row.map_err(|e| e.to_string())?);
    }
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_all_data(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.conn
        .execute_batch("DELETE FROM intervals; DELETE FROM rules;")
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = db_path(&app.handle());
            std::fs::create_dir_all(path.parent().unwrap())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let db =
                Database::open(&path).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let db = Arc::new(Mutex::new(db));
            let paused = Arc::new(Mutex::new(false));
            tracker::start(db.clone(), paused.clone());
            tracker::start_website_receiver(db.clone());
            app.manage(AppState { db, paused });
            let open = MenuItem::with_id(app, "open", "Open Daymark", true, None::<&str>)?;
            let pause =
                MenuItem::with_id(app, "pause", "Pause / resume tracking", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Daymark", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &pause, &quit])?;
            let mut tray = TrayIconBuilder::new().menu(&menu).tooltip("Daymark");
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.on_menu_event(|app, event| match event.id().as_ref() {
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause" => {
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Ok(mut paused) = state.paused.lock() {
                            *paused = !*paused;
                        }
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            })
            .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            set_category,
            set_tracking_paused,
            show_pet,
            hide_pet,
            export_data,
            delete_all_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running Daymark");
}
