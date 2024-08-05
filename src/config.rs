use std::{collections::HashSet, fmt::Display};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub map: MapMessageToSection,
}

#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitProvider {
    #[default]
    Github,
    Other,
}

impl Display for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitProvider::Github => write!(f, "github"),
            GitProvider::Other => write!(f, "other "),
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitMessageParsing {
    #[default]
    Smart,
    Strict,
}

impl Display for CommitMessageParsing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitMessageParsing::Smart => write!(f, "smart"),
            CommitMessageParsing::Strict => write!(f, "strict"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMessageToSection(IndexMap<String, HashSet<String>>);

impl Default for MapMessageToSection {
    fn default() -> Self {
        fn map(section: &'static str, message: Vec<&'static str>) -> (String, HashSet<String>) {
            (
                section.to_string(),
                HashSet::from_iter(message.into_iter().map(ToString::to_string)),
            )
        }

        // todo:
        let map = vec![
            map("Fixed", vec!["fix"]),
            map("Added", vec!["feat"]),
            map("Changed", vec!["improve", "impr"]),
        ];

        Self(IndexMap::from_iter(map))
    }
}

impl MapMessageToSection {
    pub fn into_changelog_ser_options(self) -> changelog::ser::Options {
        changelog::ser::Options {
            section_order: self.0.into_iter().map(|(section, _)| section).collect(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::Config;

    #[test]
    fn a() {
        let e = Config::default();

        let json = serde_json::ser::to_string_pretty(&e).unwrap();

        println!("{}", json);
    }
}
