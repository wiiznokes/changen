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
pub struct MapMessageToSection(pub IndexMap<String, HashSet<String>>);

impl Default for MapMessageToSection {
    fn default() -> Self {
        fn map(section: &'static str, message: Vec<&'static str>) -> (String, HashSet<String>) {
            (
                section.to_string(),
                HashSet::from_iter(message.into_iter().map(ToString::to_string)),
            )
        }

        // todo: add more
        let map = vec![
            map("Fixed", vec!["fix"]),
            map("Added", vec!["feat"]),
            map("Changed", vec!["improve", "impr", "chore "]),
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

    pub fn map_section(&self, section: &str) -> Option<String> {
        let section_normalized = section.to_lowercase();

        for (section, needles) in &self.0 {
            for needle in needles {
                let needle_normalized = needle.to_lowercase();

                if section_normalized == needle_normalized {
                    return Some(section.to_owned());
                }
            }
        }

        None
    }

    /// Best effort recognition
    pub fn try_find_section(&self, (message, desc): (&str, &str)) -> Option<String> {
        let message_normalized = message.to_lowercase();
        let desc_normalized = desc.to_lowercase();

        for (section, needles) in &self.0 {
            for needle in needles {
                let needle_normalized = needle.to_lowercase();

                if message_normalized.contains(&needle_normalized) {
                    return Some(section.to_owned());
                }
                if desc_normalized.contains(&needle_normalized) {
                    return Some(section.to_owned());
                }
            }
        }

        None
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
