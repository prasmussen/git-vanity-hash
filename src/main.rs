mod git_vanity_hash;

use std::env;
use std::thread;
use std::sync::mpsc;
use num_cpus;
use git_vanity_hash::config::{Config, Mode};
use git_vanity_hash::commit_info::CommitInfo;
use git_vanity_hash::cmd;


static VANITY_HEADER: &str = "vanity";


enum Error {
    FailedToParseArgs(),
    FailedToParseCommitInfo(),
    PrefixNotFound(),
    GitCatFile(cmd::Error),
    GitHashObject(cmd::Error),
    GitUpdateRef(cmd::Error),
}


fn main() {
    match run(env::args()) {
        Ok(_) =>
            (),

        Err(err) => {
            println!("{}", format_error(err));
            std::process::exit(1)
        },
    };
}


fn run(args: std::env::Args) -> Result<(), Error> {
    let config = Config::from_args(args)
        .ok_or(Error::FailedToParseArgs())?;

    let commit_info_str = git_cat_file()
        .map_err(Error::GitCatFile)?;

    let commit_info = CommitInfo::from_str(&commit_info_str)
        .map(|info| info.remove_header(VANITY_HEADER))
        .ok_or(Error::FailedToParseCommitInfo())?;

    let vanity_commit_info = find_vanity_commit_info(&commit_info, &config.wanted_prefix)?;
    let hash = vanity_commit_info.hash();

    println!("Found hash: {}", hash);

    match config.mode {
        Mode::Simulate() =>
            (),

        Mode::Write() => {
            git_hash_object(&vanity_commit_info.to_string())
                .map_err(Error::GitHashObject)?;

            git_update_ref(&hash)
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

        let options = WorkerOptions{
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


struct WorkerOptions {
    commit_info: CommitInfo,
    wanted_prefix: String,
    vanity_prefix: usize,
    found_channel: mpsc::Sender<CommitInfo>,
    cancel_channel: mpsc::Receiver<()>,
}

fn find_vanity_commit_info_worker(options: WorkerOptions) {
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
}


fn git_cat_file() -> Result<String, cmd::Error> {
    cmd::run("git", &["cat-file", "commit", "HEAD"])
        .and_then(cmd::output_to_string)
}


fn git_update_ref(hash: &str) -> Result<String, cmd::Error> {
    cmd::run("git", &["update-ref", "HEAD", hash])
        .and_then(cmd::output_to_string)
}


fn git_hash_object(commit_info_str: &str) -> Result<String, cmd::Error> {
    cmd::run_with_stdin("git", &["hash-object", "-t", "commit", "-w", "--stdin"], commit_info_str)
        .and_then(cmd::output_to_string)
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


fn format_command_error(err: cmd::Error) -> String {
    match err {
        cmd::Error::FailedToExecute(err) =>
            format!("Failed to execute command: {}", err),

        cmd::Error::FailedToReadStdout(err) =>
            format!("Failed decode stdout as utf-8: {}", err),

        cmd::Error::FailedToReadStderr(err) =>
            format!("Failed decode stderr as utf-8: {}", err),

        cmd::Error::ExitFailure(stderr, exit_status) => {
            match exit_status {
                Some(code) =>
                    format!("Exited with status code: {}\n{}", code, stderr),

                None =>
                    format!("Process terminated by signal\n{}", stderr),
            }
        },

        cmd::Error::FailedToCaptureStdin() =>
            "Failed to capture stdin".to_string(),

        cmd::Error::FailedToWriteStdin(err) =>
            format!("Failed to write to stdin: {}", err),

        cmd::Error::FailedToWaitForChild(err) =>
            format!("Failed to wait for child process: {}", err),
    }
}
