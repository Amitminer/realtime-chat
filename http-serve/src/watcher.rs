use crate::colors::*;
use crate::websocket::{broadcast_reload_message, get_client_count};
use notify::{Event, EventKind, Result as NotifyResult, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

static LAST_CHANGE: AtomicU64 = AtomicU64::new(0);

pub fn get_last_change() -> u64 {
    LAST_CHANGE.load(Ordering::Relaxed)
}

fn set_last_change(timestamp: u64) {
    LAST_CHANGE.store(timestamp, Ordering::Relaxed);
}

pub fn start_file_watcher(root_dir: PathBuf) -> NotifyResult<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
        if let Ok(event) = res {
            if matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                let _ = tx.send(event);
            }
        }
    })?;

    watcher.watch(&root_dir, RecursiveMode::Recursive)?;

    for event in rx {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        set_last_change(now);

        if !event.paths.is_empty() {
            let file_name = event.paths[0]
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            println!("{CYAN}🔄 File changed: {BOLD}{file_name}{RESET}");
        }

        broadcast_reload_message();
        let client_count = get_client_count();
        if client_count > 0 {
            println!(
                "{BRIGHT_CYAN}📡 Notified {client_count} connected clients{RESET}"
            );
        }
    }

    Ok(())
}
