use std::{fs::File, io::Read};

use crate::*;

#[test]
fn changelog2() {
    let mut file = File::open("../tests/changelogs/CHANGELOG2.md").unwrap();

    let mut input = String::new();

    file.read_to_string(&mut input).unwrap();

    let now = std::time::Instant::now();

    let input = input.chars().collect::<Vec<_>>();

    let parser = changelog();

    let _res = parser.parse(&input).unwrap();

    println!("{:?}", now.elapsed())

    // dbg!(&res);
}
