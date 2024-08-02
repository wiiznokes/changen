use core::str;

use pom::parser::*;

fn main() {
    let input = b"abcde";
    let parser = sym(b'a') * none_of(b"AB") - sym(b'c') + seq(b"de");
    let output = parser.parse(input);
    assert_eq!(output, Ok((b'b', vec![b'd', b'e'].as_slice())));
}

fn string() -> Parser<'static, u8, String> {
    let special_char = sym(b'\\')
        | sym(b'/')
        | sym(b'"')
        | sym(b'b').map(|_| b'\x08')
        | sym(b'f').map(|_| b'\x0C')
        | sym(b'n').map(|_| b'\n')
        | sym(b'r').map(|_| b'\r')
        | sym(b't').map(|_| b'\t');
    let escape_sequence = sym(b'\\') * special_char;
    let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseTitle {
    pub version: String,
    pub title: Option<String>,
}

fn release_title(src: &str) -> anyhow::Result<ReleaseTitle> {
    let version =
        sym(b'#').repeat(2) * sym(b' ') * sym(b'[') * none_of(b"\n]").repeat(1..) - sym(b']');

    let title = sym(b' ') * sym(b'-') * sym(b' ') * none_of(b"\n]").repeat(1..);

    let parser = version + title.opt();

    let parser = parser.convert(|(version, title)| {
        let res = unsafe {
            ReleaseTitle {
                version: String::from_utf8_unchecked(version),
                title: title.map(|title| String::from_utf8_unchecked(title)),
            }
        };

        Ok::<ReleaseTitle, ()>(res)
    });

    let res = parser.parse(src.as_bytes())?;

    Ok(res)
}

#[test]
fn t() {
    let input = "## [2024.7] - 2024-07-24";

    let res = release_title(input);

    dbg!(&res);
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct ReleaseTitle {
//     pub version: String,
// }

// fn release_title(src: &str) -> Option<ReleaseTitle> {
//     let parser =
//         sym(b'#').repeat(2) * sym(b' ') * sym(b'[') * none_of(b"\n]").repeat(1..) - sym(b']');

//     let parser =
//         parser.convert(|bytes| unsafe { Ok::<String, ()>(String::from_utf8_unchecked(bytes)) });

//     // called `Result::unwrap()` on an `Err` value: Incomplete
//     let version = parser.parse(src.as_bytes()).unwrap();

//     Some(ReleaseTitle {
//         version,
//     })
// }

// #[test]
// fn test() {
//     let input = "## [2024]";

//     let res = release_title(input);

//     dbg!(&res);
// }
