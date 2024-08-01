use light_enum::Values;

#[derive(Debug, Clone, Values, PartialEq, Eq)]
pub enum CommitKind {
    Fix,
    Improve,
    Feat,
}

impl CommitKind {
    fn text(&self) -> &'static str {
        match self {
            CommitKind::Fix => "fix",
            CommitKind::Improve => "improve",
            CommitKind::Feat => "feat",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub kind: CommitKind,
    pub scope: Option<String>,
    pub ignore: bool,
    pub message: String,
}

pub fn parse_commit_message(message: &str) -> Option<Commit> {
    let kind = match CommitKind::VALUES
        .iter()
        .find(|kind| message.starts_with(kind.text()))
    {
        Some(kind) => kind.clone(),
        None => {
            log::error!("commit type not recognized");
            return None;
        }
    };

    let mut commit = Commit {
        kind,
        scope: None,
        ignore: false,
        message: String::new(),
    };

    let mut sub = message[commit.kind.text().len()..].trim();

    if sub.starts_with("(") {
        match sub.find(|c| c == ')') {
            Some(pos) => {
                let scope = &sub[1..pos];

                if scope == "ignore" {
                    commit.ignore = true;
                } else {
                    commit.scope = Some(scope.to_string());
                }

                sub = &sub[pos + 1..].trim();
            }
            None => {
                log::error!("'(' found but no ')'");
                return None;
            }
        };
    }

    if !sub.starts_with(":") {
        log::error!("missing ':' after the commit type");
        return None;
    }

    sub = &sub[1..].trim();

    commit.message = sub.to_owned();

    Some(commit)
}

#[cfg(test)]
mod test {
    use crate::commit_parser::{Commit, CommitKind};

    use super::parse_commit_message;

    #[test]
    fn test() {
        assert_eq!(
            parse_commit_message("fix(hello): hihi"),
            Some(Commit {
                kind: CommitKind::Fix,
                scope: Some(String::from("hello")),
                ignore: false,
                message: String::from("hihi")
            })
        );
        assert_eq!(parse_commit_message("fix(hello: hihi"), None);
        assert_eq!(parse_commit_message("fixhello: hihi"), None);
        assert_eq!(
            parse_commit_message("feat:"),
            Some(Commit {
                kind: CommitKind::Feat,
                scope: None,
                ignore: false,
                message: String::from("")
            })
        );

        assert_eq!(
            parse_commit_message("improve(ignore): hihi"),
            Some(Commit {
                kind: CommitKind::Improve,
                scope: None,
                ignore: true,
                message: String::from("hihi")
            })
        );

        assert_eq!(
            parse_commit_message("improve (ignore) : hihi"),
            Some(Commit {
                kind: CommitKind::Improve,
                scope: None,
                ignore: true,
                message: String::from("hihi")
            })
        );
    }
}
