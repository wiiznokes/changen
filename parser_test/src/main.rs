use core::str;

use pom::parser::*;
use utils::{into_string, passive_tag, space};

fn main() {
    // let input = b"abcde";
    // let parser = sym(b'a') * none_of(b"AB") - sym(b'c') + seq(b"de");
    // let output = parser.parse(input);
    // assert_eq!(output, Ok((b'b', vec![b'd', b'e'].as_slice())));
}

// fn string() -> Parser<'static, u8, String> {
//     let special_char = sym(b'\\')
//         | sym(b'/')
//         | sym(b'"')
//         | sym(b'b').map(|_| b'\x08')
//         | sym(b'f').map(|_| b'\x0C')
//         | sym(b'n').map(|_| b'\n')
//         | sym(b'r').map(|_| b'\r')
//         | sym(b't').map(|_| b'\t');
//     let escape_sequence = sym(b'\\') * special_char;
//     let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
//     string.convert(String::from_utf8)
// }

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseTitle {
    pub version: String,
    pub title: Option<String>,
}

mod utils {
    use pom::{parser::*};

    pub fn into_string(v: Vec<char>) -> String {
        let str = v.into_iter().collect::<String>();
        let str = str.trim();
        str.to_owned()
    }

 

    pub fn space<'a>() -> Parser<'a, char, ()> {
        one_of(" \t\r\n").repeat(0..).discard()
    }
}

fn header<'a>() -> Parser<'a, char, Option<String>> {
    (any() - (!tag("\n## ["))).repeat(0..).convert(|header| {
        let header = into_string(header);

        if header.is_empty() {
            Ok::<_, ()>(None)
        } else {
            Ok(Some(header))
        }
    })
}



fn release_title<'a>() -> Parser<'a, char, ReleaseTitle> {
    let version =
        space() * sym('#').repeat(2) * sym(' ') * sym('[') * none_of("\n]").repeat(1..) - sym(']');

    let title = sym(' ') * sym('-') * sym(' ') * none_of("\n]").repeat(1..);

    let parser = version + title.opt();

    parser.convert(|(version, title)| {
        let res = ReleaseTitle {
            version: into_string(version),
            title: title.map(|title| into_string(title)),
        };

        Ok::<ReleaseTitle, ()>(res)
    })
}

#[test]
fn t() {
    let input = r#"
hello
la miff

## [2024.7] - 2024-07-24

"#;

    let input = input.chars();

    let input = input.collect::<Vec<_>>();

    let res = header();

    let res = res.parse(&input).unwrap();

    dbg!(&res);
}

#[test]
fn t2() {
    let input = r#"
hello
la miff


## [2024.7] - 2024-07-24

"#;

    let input = input.chars();

    let input = input.collect::<Vec<_>>();

    let res = header() + release_title();

    let res = res.parse(&input).unwrap();

    dbg!(&res);
}
