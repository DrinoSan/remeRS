use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::new_debouncer;

use notify_rust::Notification;

use daemonize::Daemonize;
use tokio;

use crate::reminder::reminder;

use serde_json;

const FILE_PATH: &str = "remind_data.json";
const PID_FILE_PATH: &str = "test.pid";
const LOG_FILE_PATH: &str = "daemon.out";

pub fn setup_daemon() {
    let stdout = File::create(LOG_FILE_PATH).unwrap();
    let daemonize = Daemonize::new()
        .pid_file(PID_FILE_PATH) // Every method except `new` and `start`
        .chown_pid_file(true) // is optional, see `Daemonize` documentation
        .working_directory(".") // for default behaviour.
        .stdout(stdout)
        .umask(0o777); // Set umask, `0o027` by default.

    // match daemonize.start() {
    //     Ok(_) => {
    //         let _ = start_daemon();
    //     }
    //     Err(e) => eprintln!("Error, {}", e),
    // }
    let _ = start_daemon();
}

#[tokio::main]
pub async fn start_daemon() -> notify::Result<()> {
    println!("Watching reminder_data.json");
    // let (tx, rx) = std::sync::mpsc::channel();
    // let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // watcher.watch(
    //     std::path::Path::new(FILE_PATH).as_ref(),
    //     RecursiveMode::NonRecursive,
    // )?;

    let events_arc = Arc::new(Mutex::new(Vec::new()));

    let events_for_check_due_date = events_arc.clone();
    let events_for_reload_file = events_arc.clone();
    let events_for_reload = events_arc.clone();
    // let (tx_events, rx_events) = mpsc::channel::<reminder::Event>();

    // setup debouncer
    let (tx, rx) = std::sync::mpsc::channel();

    // No specific tickrate, max debounce time 1 seconds
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    debouncer
        .watcher()
        .watch(Path::new(FILE_PATH), RecursiveMode::NonRecursive)
        .unwrap();

    tokio::spawn(async move {
        loop {
            println!("tokio thread");
            check_due_date(&mut events_for_check_due_date.lock().unwrap());
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(FILE_PATH);

            match file {
                Ok(mut file) => {
                    let mut events = events_for_reload.lock().unwrap();
                    let mut content = String::new();
                    file.read_to_string(&mut content).unwrap_or_default();
                    println!("File content read: {:?}", content);

                    if let Ok(new_events) = serde_json::from_str::<Vec<reminder::Event>>(&content) {
                        events.clear();
                        events.extend(new_events);
                    } else {
                        println!("Error parsing JSON");
                        return;
                    }

                    let json_content = serde_json::to_string_pretty(&*events).unwrap();
                    fs::write("remind_data.json", json_content.as_bytes()).unwrap();
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
        }
    });

    // Is a filewatcher even needed?
    for res in rx {
        match res {
            Ok(event) => {
                println!("Change: {event:?}");
                reload_file(&mut events_for_reload_file.lock().unwrap());
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn reload_file(events: &mut Vec<reminder::Event>) {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(FILE_PATH);

    match file {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or_default();
            println!("File content read: {:?}", content);

            if let Ok(new_events) = serde_json::from_str::<Vec<reminder::Event>>(&content) {
                if let Some(last_element) = new_events.last().cloned() {
                    events.push(last_element);
                };
            } else {
                println!("Error parsing JSON");
                return;
            }
        }
        Err(e) => {
            println!("Error opening file: {}", e);
        }
    }
}

fn check_due_date(events: &mut Vec<reminder::Event>) {
    for event in events.iter_mut() {
        if SystemTime::now() < event.time {
            continue;
        }

        if event.already_dispatched {
            continue;
        }

        event.already_dispatched = true;

        let _ = Notification::new()
            .summary("RUST <3 Reme")
            .body(&event.subject)
            .show();
    }
}

