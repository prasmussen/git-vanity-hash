use std::string;
use std::io;
use std::process;
use std::process::Command;
use std::io::Write;
use std::process::Stdio;



pub enum Error {
    FailedToExecute(io::Error),
    FailedToReadStdout(string::FromUtf8Error),
    FailedToReadStderr(string::FromUtf8Error),
    ExitFailure(String, Option<i32>),
    FailedToCaptureStdin(),
    FailedToWriteStdin(io::Error),
    FailedToWaitForChild(io::Error),
}


pub fn run(cmd: &str, args: &[&str]) -> Result<process::Output, Error> {
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(Error::FailedToExecute)
}


pub fn run_with_stdin(cmd: &str, args: &[&str], stdin: &str) -> Result<process::Output, Error> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::FailedToExecute)?;

    child.stdin
        .as_mut()
        .ok_or(Error::FailedToCaptureStdin())?
        .write_all(stdin.as_bytes())
        .map_err(Error::FailedToWriteStdin)?;

    child.wait_with_output()
        .map_err(Error::FailedToWaitForChild)
}


pub fn output_to_string(output: process::Output) -> Result<String, Error> {
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(Error::FailedToReadStdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(Error::FailedToReadStderr)?;

        Err(Error::ExitFailure(stderr, output.status.code()))
    }
}
