use crate::{
    config::{Cli, Generate},
    run_generic,
};

use super::FsTest;

#[test]
fn test_repo() {
    let cli = Cli {
        command: crate::config::Commands::Generate(Generate {
            file: todo!(),
            map: todo!(),
            parsing: todo!(),
            exclude_unidentified: todo!(),
            exclude_not_pr: todo!(),
            provider: todo!(),
            repo: todo!(),
            omit_pr_link: todo!(),
            omit_thanks: todo!(),
            stdout: todo!(),
            specific: todo!(),
            milestone: todo!(),
            since: todo!(),
            until: todo!(),
        }),
    };

    let r = FsTest {
        commits: vec![],
        tags: vec![],
    };

    run_generic(&r, cli).unwrap();
}
