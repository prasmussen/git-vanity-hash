use std::io;
use std::io::Write;
use std::string;
use std::fmt;
use std::env;
use std::thread;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use num_cpus;


static VANITY_HEADER: &str = "vanity";


enum Error {
    FailedToParseArgs(),
    FailedToParseCommitInfo(),
    PrefixNotFound(),
    GitCatFile(CommandError),
    GitHashObject(CommandError),
    GitUpdateRef(CommandError),
}


fn main() {
    match run(env::args()) {
        Ok(_) =>
            (),

        Err(err) =>
            println!("{}", format_error(err)),
    };
}


fn run(args: std::env::Args) -> Result<(), Error> {
    let options = Options::from_args(args)
        .ok_or(Error::FailedToParseArgs())?;

    let commit_info_str = git_cat_file()
        .and_then(process_command_output)
        .map_err(Error::GitCatFile)?;

    let commit_info = CommitInfo::from_str(&commit_info_str)
        .map(|info| info.remove_header(VANITY_HEADER))
        .ok_or(Error::FailedToParseCommitInfo())?;

    let vanity_commit_info = find_vanity_commit_info(&commit_info, &options.wanted_prefix)?;
    let hash = vanity_commit_info.hash();

    println!("Found hash: {}", hash);

    match options.mode {
        Mode::Simulate() =>
            (),

        Mode::Write() => {
            git_hash_object(&vanity_commit_info.to_string())
                .and_then(process_command_output)
                .map_err(Error::GitHashObject)?;

            git_update_ref(&hash)
                .and_then(process_command_output)
                .map_err(Error::GitUpdateRef)?;

            println!("Commit updated")
        },
    }

    Ok(())
}


fn find_vanity_commit_info(commit_info: &CommitInfo, wanted_prefix: &str) -> Result<CommitInfo, Error> {
    let mut cancel_senders = vec![];
    let (found_sender, found_receiver) = mpsc::channel();

    for i in 0..num_cpus::get() {
        let (cancel_sender, cancel_receiver) = mpsc::channel();
        cancel_senders.push(cancel_sender);

        let options = FindOptions{
            commit_info: commit_info.clone(),
            wanted_prefix: wanted_prefix.to_string().clone(),
            vanity_prefix: i,
            found_channel: found_sender.clone(),
            cancel_channel: cancel_receiver,
        };

        thread::spawn(move || find_vanity_commit_info_worker(options));
    }

    // Important! The receiver can get stuck forever if not dropped
    drop(found_sender);

    // Wait for a found value
    match found_receiver.recv() {
        Ok(vanity_commit_info) => {
            // Stop all threads
            for chan in cancel_senders {
                let _ = chan.send(());
            }

            Ok(vanity_commit_info)
        }

        Err(_) =>
            Err(Error::PrefixNotFound()),
    }
}


struct FindOptions {
    commit_info: CommitInfo,
    wanted_prefix: String,
    vanity_prefix: usize,
    found_channel: mpsc::Sender<CommitInfo>,
    cancel_channel: mpsc::Receiver<()>,
}


fn find_vanity_commit_info_worker(options: FindOptions) {
    for n in 0..std::u32::MAX {
        let vanity_value = format!("{}-{:x}", options.vanity_prefix, n);
        let vanity_commit_info = options.commit_info.add_header(VANITY_HEADER, &vanity_value);
        let hash = vanity_commit_info.hash();

        if hash.starts_with(&options.wanted_prefix) {
            let _ = options.found_channel.send(vanity_commit_info);
            break
        }

        if let Ok(()) = options.cancel_channel.try_recv() {
            break
        }
    }

    drop(options.found_channel);
}


enum CommandError {
    FailedToExecute(io::Error),
    FailedToReadStdout(string::FromUtf8Error),
    FailedToReadStderr(string::FromUtf8Error),
    ExitFailure(String, Option<i32>),
    FailedToCaptureStdin(),
    FailedToWriteStdin(io::Error),
    FailedToWaitForChild(io::Error),
}

fn git_cat_file() -> Result<process::Output, CommandError> {
    Command::new("git").args(&["cat-file", "commit", "HEAD"])
        .output()
        .map_err(CommandError::FailedToExecute)
}

fn git_update_ref(hash: &str) -> Result<process::Output, CommandError> {
    Command::new("git")
        .args(&["update-ref", "HEAD", hash])
        .output()
        .map_err(CommandError::FailedToExecute)
}

fn git_hash_object(commit_info_str: &str) -> Result<process::Output, CommandError> {
    let mut child = Command::new("git")
        .args(&["hash-object", "-t", "commit", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(CommandError::FailedToExecute)?;

    child.stdin
        .as_mut()
        .ok_or(CommandError::FailedToCaptureStdin())?
        .write_all(commit_info_str.as_bytes())
        .map_err(CommandError::FailedToWriteStdin)?;

    child.wait_with_output()
        .map_err(CommandError::FailedToWaitForChild)
}


fn process_command_output(output: process::Output) -> Result<String, CommandError> {
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(CommandError::FailedToReadStdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(CommandError::FailedToReadStderr)?;

        Err(CommandError::ExitFailure(stderr, output.status.code()))
    }
}


fn format_error(err: Error) -> String {
    match err {
        Error::FailedToParseArgs() =>
            "Usage: <simulate|write> <wanted_prefix>".to_string(),

        Error::FailedToParseCommitInfo() =>
            "Failed to parse commit info".to_string(),

        Error::PrefixNotFound() =>
            "Prefix not found".to_string(),

        Error::GitCatFile(err) =>
            format!("git cat-file failed: {}", format_command_error(err)),

        Error::GitUpdateRef(err) =>
            format!("git update-ref failed: {}", format_command_error(err)),

        Error::GitHashObject(err) =>
            format!("git hash-object failed: {}", format_command_error(err)),
    }
}


fn format_command_error(err: CommandError) -> String {
    match err {
        CommandError::FailedToExecute(err) =>
            format!("Failed to execute command: {}", err),

        CommandError::FailedToReadStdout(err) =>
            format!("Failed decode stdout as utf-8: {}", err),

        CommandError::FailedToReadStderr(err) =>
            format!("Failed decode stderr as utf-8: {}", err),

        CommandError::ExitFailure(stderr, exit_status) => {
            match exit_status {
                Some(code) =>
                    format!("Exited with status code: {}\n{}", code, stderr),

                None =>
                    format!("Process terminated by signal\n{}", stderr),
            }
        },

        CommandError::FailedToCaptureStdin() =>
            "Failed to capture stdin".to_string(),

        CommandError::FailedToWriteStdin(err) =>
            format!("Failed to write to stdin: {}", err),

        CommandError::FailedToWaitForChild(err) =>
            format!("Failed to wait for child process: {}", err),
    }
}



struct Options {
    mode: Mode,
    wanted_prefix: String,
}


impl Options {
    fn from_args(mut args: std::env::Args) -> Option<Options> {
        args.next();

        let mode = args.next()
            .and_then(|str| Mode::from_str(&str))?;

        let wanted_prefix = args.next()?;

        Some(Options {
            mode,
            wanted_prefix,
        })
    }
}



enum Mode {
    Simulate(),
    Write(),
}


impl Mode {
    fn from_str(str: &str) -> Option<Mode> {
        match str {
            "simulate" =>
                Some(Mode::Simulate()),

            "write" =>
                Some(Mode::Write()),

            _ =>
                None,
        }
    }
}




#[derive(Clone)]
struct CommitInfo {
    headers: String,
    body: String,
}

impl fmt::Display for CommitInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n\n{}", self.headers, self.body)
    }
}

impl CommitInfo {
    fn from_str(str: &str) -> Option<CommitInfo> {
        let parts: Vec<&str> = str.splitn(2, "\n\n").collect();

        match *parts.as_slice() {
            [headers, body] =>
                Some(CommitInfo{
                    headers: headers.to_string(),
                    body: body.to_string(),
                }),

            _ =>
                None,
        }
    }

    fn add_header(&self, name: &str, value: &str) -> CommitInfo {
        let new_headers = format!("{}\n{} {}", self.headers, name, value);

        CommitInfo{
            headers: new_headers,
            body: self.body.clone(),
        }
    }

    fn remove_header(&self, name: &str) -> CommitInfo {
        let new_headers = self.headers
            .split('\n')
            .filter(|header| !header.starts_with(name))
            .collect::<Vec<&str>>()
            .join("\n");

        CommitInfo{
            headers: new_headers,
            body: self.body.clone(),
        }
    }


    fn hash(&self) -> String {
        let commit_info_str = self.to_string();
        let commit_info_with_prefix = CommitInfo::add_length_prefix(&commit_info_str);

        sha1(&commit_info_with_prefix)
    }


    fn add_length_prefix(commit_info_str: &str) -> String {
        format!("commit {}{}{}", commit_info_str.len(), '\0', commit_info_str)
    }
}




fn sha1(str: &str) -> String {
    let mut hasher = Sha1::new();

    hasher.input_str(str);
    hasher.result_str()
}
