use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
    time::SystemTime,
    time::Duration,
};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_rust::Notification;

use tokio;

use crate::reminder::reminder;

use serde_json;

// #[tokio::main]
pub fn start_daemon() -> notify::Result<()> {
    println!("Watching reminder_data.json");
    let (tx, rx) = std::sync::mpsc::channel();

    let mut content: Vec<reminder::Event> = vec![];

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(
        std::path::Path::new("remind_data.json").as_ref(),
        RecursiveMode::NonRecursive,
    )?;

    // tokio::spawn(async move {
    //     loop {
    //         reload_file(&mut content);
    //         tokio::time::sleep(Duration::from_secs(30)).await;
    //     }
    // });

    for res in rx {
        match res {
            Ok(event) => {
                println!("Change: {event:?}");
                reload_file(&mut content)
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
        .open("remind_data.json");

    match file {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or_default();
            println!("File content read: {:?}", content);

            let mut changed: bool = false;

            if let Ok(new_events) = serde_json::from_str::<Vec<reminder::Event>>(&content) {
                events.clear();
                events.extend(new_events);

                for event in events.iter_mut() {
                    if SystemTime::now() > event.time {
                        if event.already_dispatched {
                            continue;
                        }

                        changed = true;
                        event.already_dispatched = true;

                        let _ = Notification::new()
                            .summary("RUST <3 Reme")
                            .body(&event.subject)
                            .show();
                    }
                }
            } else {
                println!("Error parsing JSON");
            }

            if changed {
                let json_content = serde_json::to_string_pretty(&events).unwrap();

                file.set_len(0).unwrap();
                file.seek(std::io::SeekFrom::Start(0)).unwrap();
                file.write_all(json_content.as_bytes()).unwrap();
            }
        }
        Err(e) => {
            println!("Error opening file: {}", e);
        }
    }
}
// fn reload_file(events: &mut Vec<reminder::Event>) {
//     let file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .truncate(true)
//         .create(true)
//         .open("remind_data.json");

//     match file {
//         Ok(mut file) => {
//             let mut content = String::new();
//             file.read_to_string(&mut content).unwrap_or_default();
//             println!("FIle of content read: {:?}", content);

//             if let Ok(new_events) = serde_json::from_str::<Vec<reminder::Event>>(&content) {
//                 events.clear();
//                 events.extend(new_events);
//             } else {
//                 println!("Error parsing JSON");
//             }

//             let mut changed: bool = false;
//             for event in events.iter_mut() {
//                 if SystemTime::now() > event.time {
//                     if event.already_dispatched == true {
//                         continue;
//                     }

//                     changed = true;
//                     event.already_dispatched = true;

//                     let _ = Notification::new()
//                         .summary("RUST <3 Reme")
//                         .body(&event.subject)
//                         .show();
//                 }
//             }

//             if changed {

//                 let json_content = serde_json::to_string_pretty(&events).unwrap();
//                 file.set_len(0).unwrap();
//                 file.seek(std::io::SeekFrom::Start(0)).unwrap();
//                 file.write_all(json_content.as_bytes()).unwrap();
//             }
//         }
//         Err(e) => {
//             println!("Error opening file: {}", e);
//         }
//     }
// }
