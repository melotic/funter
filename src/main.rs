use futures::future;
use itertools::Itertools;
use regex::bytes::Regex;
use std::sync::Arc;
use tokio::{fs::File, io::AsyncReadExt};
use walkdir::{DirEntry, WalkDir};

async fn worker_thread(entry: DirEntry, regex: Arc<Regex>) {
    let mut file = match File::open(&entry.path()).await {
        Ok(file) => file,
        Err(_) => return,
    };

    // Read file in 32kb chunks
    let mut buf = [0; 32 * 1024];

    match file.read(&mut buf).await {
        Ok(n) => {
            if n == 0 {
                return;
            }

            // Check if the file matches the regex
            for m in regex.find_iter(&buf[..n]) {
                println!("{}", String::from_utf8_lossy(&buf[m.start()..m.end()]));
            }
        }
        Err(e) => eprintln!("{}", e),
    }
}

#[tokio::main]
async fn main() {
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

    let root_path = match std::env::args().nth(2) {
        Some(path) => path,
        None => "/".to_string(),
    };

    for file_chunks in &WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|x| x.file_type().is_file() && !x.path().starts_with("/proc"))
        .chunks(32)
    {
        // map the chunks into a futures
        let futures = file_chunks.map(|entry| {
            let regex = Arc::clone(&regex);
            tokio::spawn(async move {
                worker_thread(entry, regex).await;
            })
        });

        // join all the futures
        future::join_all(futures).await;
    }
}
