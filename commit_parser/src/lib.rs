use pom::parser::*;
use utils::{into_string, space};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub section: String,
    pub scope: Option<String>,
    pub message: String,
}

pub fn parse_commit(input: &str) -> anyhow::Result<Commit> {
    let input = input.chars().collect::<Vec<_>>();
    let parser = commit_parser();
    let commit = parser.parse(&input)?;

    Ok(commit)
}

fn commit_parser<'a>() -> Parser<'a, char, Commit> {
    let scope = space() * sym('(') * none_of("()").repeat(1..) - sym(')');

    let parser = none_of(" :()").repeat(1..) + scope.opt() - space() * sym(':') * space()
        + any().repeat(1..);

    parser.convert(|((section, scope), message)| {
        let res = Commit {
            section: into_string(section),
            scope: scope.map(into_string),
            message: into_string(message),
        };

        Ok::<Commit, ()>(res)
    })
}

mod utils {
    use pom::parser::*;

    pub fn into_string(v: Vec<char>) -> String {
        let str = v.into_iter().collect::<String>();
        let str = str.trim();
        str.to_owned()
    }

    pub fn space<'a>() -> Parser<'a, char, ()> {
        one_of(" \t\r").repeat(0..).discard()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn map(input: &str) -> Vec<char> {
        input.chars().collect::<Vec<_>>()
    }

    #[test]
    fn test() {
        let m = map("fix(hello): hihi");
        assert_eq!(
            commit_parser().parse(&m),
            Ok(Commit {
                section: String::from("fix"),
                scope: Some(String::from("hello")),
                message: String::from("hihi")
            })
        );

        let m = map("fix(hello: hihi");
        commit_parser().parse(&m).unwrap_err();

        let m = map("feat");
        commit_parser().parse(&m).unwrap_err();

        let m = map("improve (ignore) : hihi");
        assert_eq!(
            commit_parser().parse(&m),
            Ok(Commit {
                section: String::from("improve"),
                scope: Some(String::from("ignore")),
                message: String::from("hihi")
            })
        );
    }
}
