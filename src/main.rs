use std::env;
use std::thread;

mod daemon;
mod reminder;
mod writer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Command: ./reme --daemon or ./reme");
        return;
    }

    if args.len() == 2 && args[1].trim() == "--daemon" {
        // Start daemon

        let handle_daemon = thread::spawn(|| daemon::daemon::start_daemon());
        let _ = handle_daemon.join().unwrap();
        return;
    }

    writer::writer::check_type();
    println!("Finished writing to file");
    return;

    // Add new reminder
}
