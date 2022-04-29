use std::{
    fs::{self},
    io::Read,
    sync::Arc,
    thread,
};

use crossbeam_channel::{bounded, Receiver};
use regex::bytes::Regex;
use walkdir::{DirEntry, WalkDir};

fn worker_thread(rx: Receiver<DirEntry>, regex: Arc<Regex>) {
    for entry in rx {
        let mut file = match fs::File::open(&entry.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };

        // Read file in 32kb chunks
        let mut buf = [0; 32 * 1024];

        match file.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }

                // Check if the file matches the regex
                for m in regex.find_iter(&buf[..n]) {
                    println!("{}", String::from_utf8_lossy(&buf[m.start()..m.end()]));
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}

fn main() {
    // Parse the regex from the first argument
    let regex_str = match std::env::args().nth(1) {
        Some(regex_str) => regex_str,
        None => {
            eprintln!("Usage: ./funter <regex> [path]");
            std::process::exit(1);
        }
    };

    // Build our regex
    let regex = match Regex::new(&regex_str) {
        Ok(regex) => Arc::new(regex),
        Err(err) => {
            eprintln!("error parsing regex: {}", err);
            std::process::exit(1);
        }
    };

    // Create the channel
    let (tx, rx) = bounded(1024);

    // Spawn workers
    for _ in 0..num_cpus::get() {
        let rx = rx.clone();
        let regex = regex.clone();
        thread::spawn(move || worker_thread(rx, regex));
    }

    let root_path = match std::env::args().nth(2) {
        Some(path) => path,
        None => "/".to_string(),
    };

    for path in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|x| x.file_type().is_file())
    {
        // Ignore /proc
        if path.path().starts_with("/proc") {
            continue;
        }

        // Send the path to the channel
        tx.send(path).unwrap();
    }
}
