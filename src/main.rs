mod git_vanity_hash;

use std::env;
use std::thread;
use num_cpus;
use git_vanity_hash::config::{Config, Mode};
use git_vanity_hash::commit_info::CommitInfo;
use git_vanity_hash::cmd;
use git_vanity_hash::search_manager::{SearchManager, Worker};


static VANITY_HEADER: &str = "vanity";


enum Error {
    FailedToParseArgs(),
    FailedToParseCommitInfo(),
    PrefixNotFound(),
    NothingToRevert(),
    CannotRevertToPrevious(),
    GitCatFile(cmd::Error),
    GitHashObject(cmd::Error),
    GitUpdateRef(cmd::Error),
    GitShowCommitHash(cmd::Error),
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


    match config.mode {
        Mode::Find(wanted_prefix) => {
            let commit_info = find(&wanted_prefix)?;
            println!("Found hash: {}", commit_info.hash())
        },

        Mode::Update(wanted_prefix) => {
            let change = update(&wanted_prefix)?;
            println!("Updated HEAD from {} to {}", change.head_before, change.head_now);
        },

        Mode::Revert() => {
            let change = revert()?;
            println!("Reverted HEAD from {} to {}", change.head_before, change.head_now);
        },
    }

    Ok(())
}


fn find(wanted_prefix: &str) -> Result<CommitInfo, Error> {
    let commit_info_str = git_cat_file("HEAD")
        .map_err(Error::GitCatFile)?;

    let commit_info = CommitInfo::from_str(&commit_info_str)
        .map(|info| info.remove_header(VANITY_HEADER))
        .ok_or(Error::FailedToParseCommitInfo())?;

    find_vanity_commit_info(&commit_info, wanted_prefix)
}


fn update(wanted_prefix: &str) -> Result<HeadChange, Error> {
    let commit_info = find(wanted_prefix)?;
    let old_hash = git_show_commit_hash("HEAD")
        .map_err(Error::GitShowCommitHash)?;
    let new_hash = commit_info.hash();

    git_hash_object(&commit_info.to_string())
        .map_err(Error::GitHashObject)?;

    git_update_ref(&new_hash, "git-vanity-hash: update")
        .map_err(Error::GitUpdateRef)?;


    Ok(HeadChange{
        head_now: new_hash,
        head_before: old_hash,
    })
}


struct HeadChange {
    head_now: String,
    head_before: String,
}

fn revert() -> Result<HeadChange, Error> {
    let current_str = git_cat_file("HEAD")
        .map_err(Error::GitCatFile)?;

    let current = CommitInfo::from_str(&current_str)
        .ok_or(Error::FailedToParseCommitInfo())?;

    err_if_false(
        current.has_header(VANITY_HEADER),
        Error::NothingToRevert()
    )?;

    let previous_str = git_cat_file("HEAD@{1}")
        .map_err(Error::GitCatFile)?;

    let previous = CommitInfo::from_str(&previous_str)
        .ok_or(Error::FailedToParseCommitInfo())?;

    err_if_false(
        current.remove_header(VANITY_HEADER).to_string() == previous.to_string(),
        Error::CannotRevertToPrevious()
    )?;

    let old_hash = git_show_commit_hash("HEAD")
        .map_err(Error::GitShowCommitHash)?;

    let new_hash = git_show_commit_hash("HEAD@{1}")
        .map_err(Error::GitShowCommitHash)?;

    git_update_ref(&new_hash, "git-vanity-hash: revert")
        .map_err(Error::GitUpdateRef)?;

    Ok(HeadChange{
        head_now: new_hash,
        head_before: old_hash,
    })
}


fn find_vanity_commit_info(commit_info: &CommitInfo, wanted_prefix: &str) -> Result<CommitInfo, Error> {
    let mut manager = SearchManager::new();

    for i in 0..num_cpus::get() {
        let worker = manager.new_worker();

        let options = SearchOptions{
            commit_info: commit_info.clone(),
            wanted_prefix: wanted_prefix.to_string(),
            vanity_prefix: i,
        };

        thread::spawn(move || find_vanity_commit_info_worker(options, worker));
    }

    manager.immutable()
        .race()
        .ok_or(Error::PrefixNotFound())
}


struct SearchOptions {
    commit_info: CommitInfo,
    wanted_prefix: String,
    vanity_prefix: usize,
}

fn find_vanity_commit_info_worker(options: SearchOptions, worker: Worker<CommitInfo>) {
    for n in 0..std::u128::MAX {
        let vanity_value = format!("{}-{:x}", options.vanity_prefix, n);
        let commit_info = options.commit_info.add_header(VANITY_HEADER, &vanity_value);

        if commit_info.hash().starts_with(&options.wanted_prefix) {
            worker.found(commit_info);
            break
        }

        if worker.should_stop() {
            break
        }
    }
}


fn git_cat_file(rev: &str) -> Result<String, cmd::Error> {
    cmd::run("git", &["cat-file", "commit", rev])
        .and_then(cmd::output_to_string)
}


fn git_update_ref(hash: &str, message: &str) -> Result<String, cmd::Error> {
    cmd::run("git", &["update-ref", "-m", message, "HEAD", hash])
        .and_then(cmd::output_to_string)
}


fn git_hash_object(commit_info_str: &str) -> Result<String, cmd::Error> {
    cmd::run_with_stdin("git", &["hash-object", "-t", "commit", "-w", "--stdin"], commit_info_str)
        .and_then(cmd::output_to_string)
}


fn git_show_commit_hash(rev: &str) -> Result<String, cmd::Error> {
    cmd::run("git", &["show", "-s", "--format=%H", rev])
        .and_then(cmd::output_to_string)
        .map(|str| str.trim_end().to_string())
}


fn format_error(err: Error) -> String {
    match err {
        Error::FailedToParseArgs() =>
            concat!(
                "Usage: git-vanity-hash <mode> <prefix>\n\n",
                "mode\n",
                "    find        Find and print hash (read-only)\n",
                "    update      Find and update HEAD with found hash\n",
                "    revert      Revert HEAD back to original commit\n\n",
                "prefix\n",
                "    A hexadecimal string the hash should start with",
            ).to_string(),

        Error::FailedToParseCommitInfo() =>
            "Failed to parse commit info".to_string(),

        Error::PrefixNotFound() =>
            "Prefix not found".to_string(),

        Error::NothingToRevert() =>
            "Nothing to revert. HEAD commit does not have a vanity header".to_string(),

        Error::CannotRevertToPrevious() =>
            "Can't revert. HEAD does not match HEAD@{1}".to_string(),

        Error::GitCatFile(err) =>
            format!("git cat-file failed: {}", format_command_error(err)),

        Error::GitUpdateRef(err) =>
            format!("git update-ref failed: {}", format_command_error(err)),

        Error::GitHashObject(err) =>
            format!("git hash-object failed: {}", format_command_error(err)),

        Error::GitShowCommitHash(err) =>
            format!("git show failed: {}", format_command_error(err)),
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


fn err_if_false<E>(value: bool, err: E) -> Result<(), E> {
    if value {
        Ok(())
    } else {
        Err(err)
    }
}
