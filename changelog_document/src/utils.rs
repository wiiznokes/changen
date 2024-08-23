use std::sync::LazyLock;

use semver::Version;

use crate::{ChangeLog, Release, ReleaseTitle};

pub const UNRELEASED: &str = "Unreleased";

pub static DEFAULT_UNRELEASED: LazyLock<Release> = LazyLock::new(|| Release {
    title: ReleaseTitle {
        version: UNRELEASED.into(),
        release_link: None,
        title: None,
    },
    header: Default::default(),
    note_sections: Default::default(),
    footer: Default::default(),
});

impl ChangeLog {
    pub fn last_version(&self) -> Option<Version> {
        let mut keys = self.releases.keys();

        keys.next().cloned()
    }

    pub fn unreleased_or_default(&mut self) -> &mut Release {
        if self.unreleased.is_none() {
            self.unreleased = Some(DEFAULT_UNRELEASED.clone())
        };

        self.unreleased.as_mut().unwrap()
    }
}