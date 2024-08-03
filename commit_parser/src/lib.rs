use pom::parser::*;
use utils::{into_string, space};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub section: String,
    pub scope: Option<String>,
    pub message: String,
}

pub fn commit<'a>() -> Parser<'a, char, Commit> {
    let scope = sym('(') * none_of("()").repeat(1..) - sym(')');

    // let section = (!scope + none_of(" :")) * any().repeat(1..) + scope.opt();

    let parser = none_of(" :(").repeat(1..) + scope.opt() - sym(':') * space() + any().repeat(1..);

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

    #[test]
    fn test() {
        let message = "fix(hello): hihi";

        let input = message.chars().collect::<Vec<_>>();

        let parser = commit();

        let res = parser.parse(&input).unwrap();

        dbg!(&res);
    }
}
