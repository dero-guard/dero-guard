use std::io::{Error as IoError, Write};
use std::process::{Command, Stdio};

use failure::Fail;

pub fn execute(line: Vec<&str>) -> Result<String, ExecutionError> {
    execute_with(line, "")
}

pub fn execute_with(line: Vec<&str>, input: &str) -> Result<String, ExecutionError> {
    use ExecutionError::*;

    println!("   > {}", line.join(" "));

    let mut process = Command::new(line.get(0).ok_or(EmptyLine)?)
        .args(&line[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| ExecError { inner: err })?;

    let stdin = process.stdin.as_mut().ok_or(NoStdin)?;
    stdin
        .write_all(input.as_bytes())
        .map_err(|err| WriteError { inner: err })?;

    let std::process::Output { stdout, stderr, .. } = process
        .wait_with_output()
        .map_err(|err| ExecError { inner: err })?;

    let stderr = read_output(stderr)?;
    if !stderr.is_empty() {
        eprintln!("     /!\\ {}", stderr.trim());
    }

    Ok(read_output(stdout)?.trim().to_string())
}

fn read_output(output: Vec<u8>) -> Result<String, ExecutionError> {
    String::from_utf8(output).map_err(|err| ExecutionError::UTF8Error { inner: err })
}

#[derive(Debug, Fail)]
pub enum ExecutionError {
    #[fail(display = "Tried to execute an empty command")]
    EmptyLine,

    #[fail(display = "Missing standard input")]
    NoStdin,

    #[fail(display = "I/O error while writing to standard input: {}", inner)]
    WriteError { inner: IoError },

    #[fail(display = "Error while parsing command output as UTF-8: {}", inner)]
    UTF8Error { inner: std::string::FromUtf8Error },

    #[fail(display = "Error during command execution: {}", inner)]
    ExecError { inner: IoError },
}
