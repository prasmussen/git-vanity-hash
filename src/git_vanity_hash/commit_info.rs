use std::fmt;
use crypto::digest::Digest;
use crypto::sha1::Sha1;


#[derive(Clone)]
pub struct CommitInfo {
    headers: String,
    body: String,
}

impl fmt::Display for CommitInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n\n{}", self.headers, self.body)
    }
}

impl CommitInfo {
    pub fn from_str(str: &str) -> Option<CommitInfo> {
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

    pub fn add_header(&self, name: &str, value: &str) -> CommitInfo {
        let new_headers = format!("{}\n{} {}", self.headers, name, value);

        CommitInfo{
            headers: new_headers,
            body: self.body.clone(),
        }
    }

    pub fn remove_header(&self, name: &str) -> CommitInfo {
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


    pub fn hash(&self) -> String {
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

