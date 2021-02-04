/*
 * Taken from: https://github.com/illumos/confomat
 */

use slog::*;
use std::io::{BufRead, BufReader, Read};

pub(crate) fn spawn_reader<T>(
    log: &Logger,
    name: &str,
    stream: Option<T>,
) -> Option<std::thread::JoinHandle<()>>
where
    T: Read + Send + 'static,
{
    let name = name.to_string();
    let stream = match stream {
        Some(stream) => stream,
        None => return None,
    };

    let log = log.clone();

    Some(std::thread::spawn(move || {
        let mut r = BufReader::new(stream);

        loop {
            let mut buf = String::new();

            match r.read_line(&mut buf) {
                Ok(0) => {
                    /*
                     * EOF.
                     */
                    return;
                }
                Ok(_) => {
                    let s = buf.trim();

                    if !s.is_empty() {
                        info!(log, "{}| {}", name, s);
                    }
                }
                Err(e) => {
                    error!(log, "failed to read {}: {}", name, e);
                    std::process::exit(100);
                }
            }
        }
    }))
}
