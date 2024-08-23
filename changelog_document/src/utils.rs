use std::{borrow::Cow, collections::btree_map, iter::Rev, sync::LazyLock};

use anyhow::bail;
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
        self.releases_keys().next().cloned()
    }

    pub fn unreleased_or_default(&mut self) -> &mut Release {
        if self.unreleased.is_none() {
            self.unreleased = Some(DEFAULT_UNRELEASED.clone())
        };

        self.unreleased.as_mut().unwrap()
    }

    pub fn releases(&self) -> Rev<btree_map::Values<Version, Release>> {
        self.releases.values().rev()
    }

    pub fn releases_keys(&self) -> Rev<btree_map::Keys<Version, Release>> {
        self.releases.keys().rev()
    }

    pub fn releases_full(&self) -> Rev<btree_map::Iter<Version, Release>> {
        self.releases.iter().rev()
    }
}

pub enum NthRelease<'a> {
    Unreleased(Cow<'a, Release>),
    Released(Cow<'a, Version>, Cow<'a, Release>),
}

impl<'a> NthRelease<'a> {
    pub fn release(self) -> Cow<'a, Release> {
        match self {
            NthRelease::Unreleased(e) => e,
            NthRelease::Released(_, e) => e,
        }
    }

    pub fn owned(&'a self) -> NthRelease<'static> {
        match self {
            NthRelease::Unreleased(e) => NthRelease::Unreleased(Cow::Owned(e.clone().into_owned())),
            NthRelease::Released(v, r) => NthRelease::Released(
                Cow::Owned(v.clone().into_owned()),
                Cow::Owned(r.clone().into_owned()),
            ),
        }
    }
}

impl ChangeLog {
    /// -1 being unreleased, 0 the last release, ...
    pub fn nth_release(&self, n: i32) -> anyhow::Result<NthRelease<'_>> {
        match n {
            ..-1 => {
                bail!("invalid n: {n}")
            }
            -1 => self
                .unreleased
                .as_ref()
                .map(|e| NthRelease::Unreleased(Cow::Borrowed(e)))
                .ok_or(anyhow::format_err!("No unreleased section")),

            0.. => self
                .releases_full()
                .nth(n as usize)
                .map(|e| NthRelease::Released(Cow::Borrowed(e.0), Cow::Borrowed(e.1)))
                .ok_or(anyhow::format_err!("the {n}th release does not exist")),
        }
    }
}

impl Release {
    pub fn version(&self) -> &str {
        &self.title.version
    }
}
