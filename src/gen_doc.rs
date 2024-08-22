use std::{fs::OpenOptions, io::Write};

use changelog_gen::config::Cli;

fn main() -> anyhow::Result<()> {
    let doc = clap_markdown::help_markdown::<Cli>();

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("res/API_REFERENCE.md")?;

    file.write_all(doc.as_bytes())?;

    println!("Documentation successfully written");

    Ok(())
}
