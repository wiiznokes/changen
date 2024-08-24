use changen::{config::Cli, run};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    run(cli)
}

#[cfg(test)]
mod gen {
    use std::{fs::OpenOptions, io::Write};

    use changen::config::Cli;

    #[ignore = ""]
    #[test]
    fn gen_doc() -> anyhow::Result<()> {
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
}
