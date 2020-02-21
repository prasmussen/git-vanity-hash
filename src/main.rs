use std::io;
use std::io::Write;
use std::string;
use std::fmt;
use std::process::Command;
use std::process::Stdio;
use crypto::digest::Digest;
use crypto::sha1::Sha1;



enum Error {
    FailedToGetCommitInfo(io::Error),
    FailedToReadCommitInfoStdout(string::FromUtf8Error),
    FailedToReadCommitInfoStderr(string::FromUtf8Error),
    GetCommitInfoNonZeroExitCode(String),
    FailedToParseCommitInfo(),
    PrefixNotFound(),
    FailedToSpawnGitHashObject(io::Error),
    FailedToCaptureStdinOfGitHashObject(),
    GitHashObjectFailedToWriteStdin(io::Error),
    FailedToWaitForGitHashObject(io::Error),
    FailedToReadGitHashObjectStdout(string::FromUtf8Error),
    FailedToReadGitHashObjectStderr(string::FromUtf8Error),
    GitHashObjectNonZeroExitCode(String),
    FailedToUpdateRef(io::Error),
    FailedToReadUpdateRefStderr(string::FromUtf8Error),
    UpdateRefNonZeroExitCode(String),
}




fn main() {
    match run() {
        Ok(_) =>
            (),

        Err(err) =>
            print_err(err),
    };
}


fn run() -> Result<(), Error> {
    let commit_info_str = get_commit_info_str()?;
    let commit_info = CommitInfo::from_str(&commit_info_str)
        .ok_or(Error::FailedToParseCommitInfo())?;

    let wanted_hash_prefix = String::from("0000");
    let vanity_commit_info = find_vanity_commit_info(&commit_info, &wanted_hash_prefix)?;



    println!("{}", vanity_commit_info.to_string());
    println!("{}", vanity_commit_info.hash());

    let hash = git_hash_object(&vanity_commit_info.to_string())?;
    git_update_ref(&hash)
}


fn find_vanity_commit_info(commit_info: &CommitInfo, wanted_hash_prefix: &str) -> Result<CommitInfo, Error> {
    let mut result = Err(Error::PrefixNotFound());
    let vanity_header = "vanity";

    for n in 0..std::u32::MAX {
        let vanity_value = format!("{:x}", n);
        let vanity_commit_info = commit_info.add_header(&vanity_header, &vanity_value);
        let hash = vanity_commit_info.hash();

        if hash.starts_with(wanted_hash_prefix) {
            result = Ok(vanity_commit_info);
            break;
        }
    }

    result
}


fn get_commit_info_str() -> Result<String, Error> {
    let output = Command::new("git")
        .arg("cat-file")
        .arg("commit")
        .arg("HEAD")
        .output()
        .map_err(Error::FailedToGetCommitInfo)?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(Error::FailedToReadCommitInfoStdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(Error::FailedToReadCommitInfoStderr)?;

        Err(Error::GetCommitInfoNonZeroExitCode(stderr))
    }
}


fn git_hash_object(commit_info_str: &str) -> Result<String, Error> {
    let mut child = Command::new("git")
        .arg("hash-object")
        .arg("-t")
        .arg("commit")
        .arg("-w")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::FailedToSpawnGitHashObject)?;

    child.stdin
        .as_mut()
        .ok_or(Error::FailedToCaptureStdinOfGitHashObject())?
        .write_all(commit_info_str.as_bytes())
        .map_err(Error::GitHashObjectFailedToWriteStdin)?;

    let output = child.wait_with_output()
        .map_err(Error::FailedToWaitForGitHashObject)?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|str| str.trim_end().to_string())
            .map_err(Error::FailedToReadGitHashObjectStdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(Error::FailedToReadGitHashObjectStderr)?;

        Err(Error::GitHashObjectNonZeroExitCode(stderr))
    }
}


fn git_update_ref(hash: &str) -> Result<(), Error> {
    let output = Command::new("git")
        .arg("update-ref")
        .arg("HEAD")
        .arg(hash)
        .output()
        .map_err(Error::FailedToUpdateRef)?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(Error::FailedToReadUpdateRefStderr)?;

        Err(Error::UpdateRefNonZeroExitCode(stderr))
    }
}


fn print_err(err: Error) {
    match err {
        Error::FailedToGetCommitInfo(err) =>
            println!("Failed to get commit info: {}", err),

        Error::FailedToReadCommitInfoStdout(err) =>
            println!("Failed to read commit info stdin as utf-8: {}", err),

        Error::FailedToReadCommitInfoStderr(err) =>
            println!("Failed to read commit info stderr as utf-8: {}", err),

        Error::GetCommitInfoNonZeroExitCode(err) =>
            println!("Got non-zero exit code when getting commit info: {}", err),

        Error::FailedToParseCommitInfo() =>
            println!("Failed to parse commit info"),

        Error::PrefixNotFound() =>
            println!("Prefix not found"),

        Error::FailedToSpawnGitHashObject(err) =>
            println!("Failed to spawn git hash-object: {}", err),

        Error::FailedToCaptureStdinOfGitHashObject() =>
            println!("Failed to capture stdin of git hash-object"),

        Error::GitHashObjectFailedToWriteStdin(err) =>
            println!("Failed to write to stdin of git hash-object: {}", err),

        Error::FailedToWaitForGitHashObject(err) =>
            println!("Failed to wait for git hash-object: {}", err),

        Error::FailedToReadGitHashObjectStdout(err) =>
            println!("Failed to read stdout from git hash-object as utf-8: {}", err),

        Error::FailedToReadGitHashObjectStderr(err) =>
            println!("Failed to read stdout from git hash-object as utf-8: {}", err),

        Error::GitHashObjectNonZeroExitCode(err) =>
            println!("Got non-zero exit code when running git hash-object: {}", err),

        Error::FailedToUpdateRef(err) =>
            println!("Failed to update ref: {}", err),

        Error::FailedToReadUpdateRefStderr(err) =>
            println!("Failed to ref stderr from git update-ref as utf-8: {}", err),

        Error::UpdateRefNonZeroExitCode(err) =>
            println!("Got non-zero exit code when running git update-ref: {}", err),
    };
}



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
