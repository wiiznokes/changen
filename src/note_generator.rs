use std::process::Command;

use crate::config::Config;


fn last_commit_title() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap()
}

fn last_commit() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap()
}



#[cfg(test)]
mod test {
    use super::last_commit_title;



    #[test]
    fn test() {

        let res = last_commit_title();

        dbg!(&res);
    }
}