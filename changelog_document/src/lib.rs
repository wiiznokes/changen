use std::collections::HashMap;

use indexmap::IndexMap;

mod parser;
pub use parser::changelog;

mod serializer;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTitle {
    pub version: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSection {
    pub title: String,
    pub notes: Vec<ReleaseSectionNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSectionNote {
    pub component: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub title: ReleaseTitle,
    pub header: Option<String>,
    pub notes: HashMap<String, ReleaseSection>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterLink {
    pub text: String,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterLinks {
    pub links: Vec<FooterLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLog {
    pub header: Option<String>,
    pub releases: IndexMap<String, Release>,
    pub footer_links: FooterLinks,
}
