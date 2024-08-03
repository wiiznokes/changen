use std::collections::HashSet;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub map: MapMessageToSection,
    pub provider: Provider,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Provider {
    #[default]
    Github,
    Other,
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

        let map = vec![
            map("Fixed", vec!["fix"]),
            map("Added", vec!["feat"]),
            map("Changed", vec!["improve", "impr"]),
        ];

        Self(IndexMap::from_iter(map.into_iter()))
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
