use crate::db::Database;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use uuid::Uuid;

#[cfg(target_os = "macos")]
pub mod macos {
    use std::process::Command;
    pub fn focused_app() -> Option<(String, String)> {
        let script = "tell application \"System Events\" to tell (first process whose frontmost is true) to return name";
        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .ok()?;
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some((name.clone(), name))
        }
    }
    pub fn idle_seconds() -> i64 {
        let output = Command::new("ioreg")
            .args(["-c", "IOHIDSystem"])
            .output()
            .ok();
        let text = output
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();
        text.lines()
            .find_map(|line| {
                line.split("HIDIdleTime")
                    .nth(1)?
                    .split('=')
                    .nth(1)?
                    .trim()
                    .parse::<i64>()
                    .ok()
            })
            .map(|n| n / 1_000_000_000)
            .unwrap_or(0)
    }
}

#[cfg(not(target_os = "macos"))]
pub mod macos {
    pub fn focused_app() -> Option<(String, String)> {
        None
    }
    pub fn idle_seconds() -> i64 {
        0
    }
}

pub fn start(state: Arc<Mutex<Database>>, paused: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        let mut open: Option<(String, String, i64)> = None;
        loop {
            thread::sleep(Duration::from_secs(5));
            let now = chrono::Local::now().timestamp();
            let is_paused = paused.lock().map(|p| *p).unwrap_or(true);
            let current = if !is_paused && !crate::core::is_idle(0, macos::idle_seconds(), 300) {
                macos::focused_app()
            } else {
                None
            };
            if current.as_ref().map(|x| &x.0) != open.as_ref().map(|x| &x.0) {
                if let Some((key, label, start)) = open.take() {
                    if let Some(interval) = crate::core::close_interval(
                        Some(crate::core::Interval {
                            key,
                            source: "app".into(),
                            start,
                            end: now,
                        }),
                        now,
                    ) {
                        if let Ok(db) = state.lock() {
                            let _ = db.insert(
                                &Uuid::new_v4().to_string(),
                                &interval,
                                &label,
                                None,
                                false,
                            );
                        }
                    }
                }
                if let Some((key, label)) = current {
                    open = Some((key, label, now));
                }
            }
        }
    });
}

pub fn start_website_receiver(state: Arc<Mutex<Database>>) {
    #[cfg(unix)]
    thread::spawn(move || {
        use std::{fs, io::Read, os::unix::net::UnixListener};
        let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join("Library/Application Support/Daymark/events.sock");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::remove_file(&path);
        let listener = match UnixListener::bind(path) {
            Ok(x) => x,
            Err(_) => return,
        };
        for mut stream in listener.incoming().flatten() {
            let mut header = [0u8; 4];
            if stream.read_exact(&mut header).is_err() {
                continue;
            }
            let len = u32::from_ne_bytes(header) as usize;
            let mut body = vec![0u8; len];
            if stream.read_exact(&mut body).is_err() {
                continue;
            }
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body) {
                let start = v["startedAt"].as_i64().unwrap_or(0) / 1000;
                let end = v["endedAt"].as_i64().unwrap_or(0) / 1000;
                let domain = v["domain"].as_str().unwrap_or("unknown").to_string();
                if end > start {
                    if let Ok(db) = state.lock() {
                        let _ = db.insert(
                            &Uuid::new_v4().to_string(),
                            &crate::core::Interval {
                                key: domain.clone(),
                                source: "website".into(),
                                start,
                                end,
                            },
                            &domain,
                            Some(&domain),
                            false,
                        );
                    }
                }
            }
        }
    });
}
