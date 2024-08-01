pub fn parse_change_log(changelog: &str) {
    let changelog = parse_changelog::parse(changelog).unwrap();

    dbg!(&changelog);
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::parse_change_log;

    #[test]
    fn test() {
        let mut file = File::open("tests/changelogs/CHANGELOG1.md").unwrap();

        let mut changelog = String::new();

        file.read_to_string(&mut changelog).unwrap();

        parse_change_log(&changelog);
    }
}
