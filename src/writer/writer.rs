use std::fs::{OpenOptions, self};
use std::io::{stdin, Read, Seek, Write};
use std::time::{Duration, SystemTime};

use serde_json;

use crate::reminder::reminder;

pub fn check_type() {
    println!("Hey you. Press 't' to set a timer or 'p' to set an appointment.");

    let mut option = String::new();
    // Could use read_exact???
    match stdin().read_line(&mut option) {
        Ok(_) => {}
        Err(e) => println!("We got an error: {e}"),
    }

    if option.trim() == 't'.to_string() {
        write_event();
        return;
    }

    return;
}

// Change my name to write_reminder
fn write_event() {
    println!("Hours");
    let mut hours = String::new();
    stdin().read_line(&mut hours).unwrap();
    let hours: i64 = hours.trim().parse().expect("Input is not an integer");

    println!("Minutes");
    let mut minutes = String::new();
    stdin().read_line(&mut minutes).unwrap();
    let minutes: i64 = minutes.trim().parse().expect("Input is not an integer");

    println!("Subject");
    let mut subject = String::new();
    stdin().read_line(&mut subject).unwrap();

    // Getting into seconds
    let additional_time_seconds = hours * 3600 + minutes * 60;
    let additional_time_duration = Duration::from_secs(additional_time_seconds as u64);

    let current_time = SystemTime::now();
    let new_time = current_time
        .checked_add(additional_time_duration)
        .unwrap_or(current_time);

    let remind_event = reminder::Event {
        time: new_time,
        subject,
        already_dispatched: false,
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("remind_data.json");

    match file {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or_default();

            // Parsing existing object
            let mut json_content: serde_json::Value =
                serde_json::from_str(&content).unwrap_or(serde_json::Value::Array(Vec::new()));

            // if already array, append object
            if let serde_json::Value::Array(ref mut arr) = json_content {
                let new_json_value = serde_json::to_value(&remind_event).unwrap();
                arr.push(new_json_value);
            } else {
                // Create new array and add element
                let new_array =
                    serde_json::Value::Array(vec![serde_json::to_value(&remind_event).unwrap()]);
                json_content = new_array;
            }

            // Write back to file
            let serialized = serde_json::to_string_pretty(&json_content).unwrap();
            fs::write("remind_data.json", serialized.as_bytes()).unwrap();
        }
        Err(e) => {
            println!("Error opening file: {}", e);
        }
    }
}
