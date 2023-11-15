use std::env;

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
        daemon::daemon::setup_daemon();
    }
    else {
        // Add new reminder
        writer::writer::check_type();
        println!("Finished writing to file");
    }
}
