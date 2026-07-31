//! Opening the platform's default browser at the served URL.

use std::process::{Command, Stdio};

/// Launch a browser at `url`. A launch failure is reported and the server keeps
/// running: the URL has already been printed, so it stays reachable by hand.
pub fn open(url: &str) {
    let (program, arguments): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("open", &[])
    } else if cfg!(target_os = "windows") {
        ("cmd", &["/c", "start", ""])
    } else {
        ("xdg-open", &[])
    };

    let launched = Command::new(program)
        .args(arguments)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if let Err(source) = launched {
        eprintln!("warning: could not open a browser with `{program}`: {source}");
    }
}
