use anyhow::{Context, Result};
use lazy_static::*;
use reqwest::blocking as _reqwest;
use slog::*;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

mod util;

const BASE_URL: &str = "https://raw.githubusercontent.com/notracking/hosts-blocklists/master";

lazy_static! {
    static ref LOGGER: Logger = create_logger();
}

#[derive(Error, Debug)]
enum NoTrackingError {
    #[error("invalid domain line: `{0}`")]
    InvalidDomain(String),
    #[error("invalid hostname line: `{0}`")]
    InvalidHostname(String),
}

#[derive(Copy, Clone, Debug)]
enum FileType {
    Domains,
    Hostnames,
}

impl FileType {
    fn as_str(&self) -> &str {
        match *self {
            FileType::Domains => "domains",
            FileType::Hostnames => "hostnames",
        }
    }
}

fn create_logger() -> Logger {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!())
}

fn valid_ip(ip: &str) -> bool {
    matches!(ip, "0.0.0.0" | "::")
}

fn validate_domain_line(line: &str) -> Result<()> {
    // ex: "address=/hostname.domain.com/0.0.0.0"
    let split: Vec<&str> = line.split('/').collect();
    if split.len() != 3 || split[0] != "address=" || !valid_ip(split[2]) {
        return Err(NoTrackingError::InvalidDomain(line.to_string()).into());
    }

    Ok(())
}

fn validate_hostname_line(line: &str) -> Result<()> {
    // ex: "0.0.0.0 hostname.domain.com"
    let split: Vec<&str> = line.split(' ').collect();
    if split.len() != 2 || !valid_ip(split[0]) {
        return Err(NoTrackingError::InvalidHostname(line.to_string()).into());
    }
    Ok(())
}

fn validate<S: AsRef<str>>(ftype: FileType, data: S) -> Result<()> {
    let lines = data.as_ref().lines();
    let f = match ftype {
        FileType::Domains => validate_domain_line,
        FileType::Hostnames => validate_hostname_line,
    };

    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        f(line)?;
    }

    Ok(())
}

fn do_list<P: AsRef<Path>>(ftype: FileType, path: P) -> Result<()> {
    let url = format!("{}/{}.txt", BASE_URL, ftype.as_str());
    let path = path.as_ref().join(ftype.as_str()).with_extension("txt");
    let path_tmp = &path.with_extension("tmp");

    info!(LOGGER, "getting {} at {}", ftype.as_str(), &url);
    let body = _reqwest::get(&url)?.text()?;
    validate(ftype, &body)?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path_tmp)
        .with_context(|| format!("failed to open {:?}", &path_tmp))?;
    file.write_all(body.as_bytes())?;
    file.flush()?;

    fs::rename(&path_tmp, &path)?;
    info!(LOGGER, "installed {} to {:?}", ftype.as_str(), &path);

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0].clone();
    let brief = format!("Usage: {} [options] [<command>...]", program);
    let mut dir = PathBuf::new();

    let mut opts = getopts::Options::new();
    opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
    opts.optflag("h", "help", "Print this help message");
    opts.optopt(
        "d",
        "",
        "Directory to store notracking files in",
        "DIRECTORY",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            anyhow::bail!("{}\n{}", f, opts.usage(&brief));
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(&brief));
        return Ok(());
    }

    if let Some(path) = matches.opt_str("d") {
        dir.push(path);
    }

    if dir.as_os_str().is_empty() {
        dir = std::env::current_dir()?;
    }

    do_list(FileType::Domains, &dir)?;
    do_list(FileType::Hostnames, &dir)?;

    if !matches.free.is_empty() {
        info!(LOGGER, "exec {:?}", &matches.free);

        let mut cmd = Command::new(&matches.free[0]);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if matches.free.len() > 1 {
            cmd.args(&matches.free[1..]);
        }

        let mut child = cmd.spawn()?;

        let readout = util::spawn_reader(&LOGGER, "O", child.stdout.take());
        let readerr = util::spawn_reader(&LOGGER, "E", child.stderr.take());

        if let Some(t) = readout {
            t.join().expect("join stdout thread");
        }
        if let Some(t) = readerr {
            t.join().expect("join stderr thread");
        }

        match child.wait() {
            Err(e) => return Err(e.into()),
            Ok(es) => {
                if !es.success() {
                    return Err(anyhow::anyhow!("exec {:?}: failed {:?}", &args, &es));
                }
            }
        };
    }

    Ok(())
}
