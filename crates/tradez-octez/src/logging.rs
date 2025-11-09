use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};

pub fn spawn_prefixed_logger<R>(reader: R, prefix: &str, is_err: bool) -> JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    let prefix = prefix.to_string();
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if is_err {
                        eprintln!("[{}] {}", prefix, line);
                    } else {
                        println!("[{}] {}", prefix, line);
                    }
                }
                Err(_) => break,
            }
        }
    })
}

pub fn print_prefixed_lines(data: &[u8], prefix: &str, is_err: bool) {
    let content = String::from_utf8_lossy(data);
    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        if is_err {
            eprintln!("[{}] {}", prefix, line);
        } else {
            println!("[{}] {}", prefix, line);
        }
    }
}

pub fn configure_stdio(command: &mut Command, verbose: bool) {
    if verbose {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
}

pub fn spawn_command(
    command: &mut Command,
    prefix: &str,
    verbose: bool,
    print_commands: bool,
    spawn_err: &str,
) -> (Child, Option<JoinHandle<()>>, Option<JoinHandle<()>>) {
    configure_stdio(command, verbose);
    if print_commands {
        println!("> {:?}", command);
    }
    let mut child = command.spawn().expect(spawn_err);
    let (stdout_handle, stderr_handle) = if verbose {
        (
            child
                .stdout
                .take()
                .map(|stdout| spawn_prefixed_logger(stdout, prefix, false)),
            child
                .stderr
                .take()
                .map(|stderr| spawn_prefixed_logger(stderr, prefix, true)),
        )
    } else {
        (None, None)
    };
    (child, stdout_handle, stderr_handle)
}

pub fn run_command(
    command: &mut Command,
    prefix: &str,
    verbose: bool,
    print_commands: bool,
    spawn_err: &str,
    wait_err: &str,
) {
    let (mut child, stdout_handle, stderr_handle) =
        spawn_command(command, prefix, verbose, print_commands, spawn_err);
    child.wait().expect(wait_err);
    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }
}
