use crate::generate::generate;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_repo() {
    env_logger::init();

    let mut options = DEFAULT_GENERATE.clone();

    let r = FsTest {
        commits: vec![
            raw_commit("fix: 1", "000"),
            raw_commit("fix: 2", "001"),
            raw_commit("fix: 3", "002"),
            raw_commit("doc: 1", "003"),
            raw_commit("doc: 2", "004"),
            raw_commit("doc: 3", "005"),
        ],
        tags: vec![
            tag("0.1.0", "002"),
            tag("0.1.1", "004"),
            tag("0.2.1", "005"),
        ],
    };

    options.until = Some("004".into());

    let changelog = read_changelog("src/integration_test/test1/test1.init").unwrap();

    let output = generate(&r, changelog, &options).unwrap();

    let expected = read_file("src/integration_test/test1/test1.expect").unwrap();

    // println!("{}", output);

    assert_eq!(output, expected);
}
