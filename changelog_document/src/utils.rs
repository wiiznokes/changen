use std::{borrow::Cow, collections::btree_map, iter::Rev, sync::LazyLock};

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
    pub fn nth_release(&self, n: usize) -> Option<NthRelease<'_>> {
        if self.unreleased.is_some() {
            if n == 0 {
                self.unreleased
                    .as_ref()
                    .map(|e| NthRelease::Unreleased(Cow::Borrowed(e)))
            } else {
                self.releases_full()
                    .nth(n - 1)
                    .map(|e| NthRelease::Released(Cow::Borrowed(e.0), Cow::Borrowed(e.1)))
            }
        } else {
            self.releases_full()
                .nth(n)
                .map(|e| NthRelease::Released(Cow::Borrowed(e.0), Cow::Borrowed(e.1)))
        }
    }
}

impl Release {
    pub fn version(&self) -> &str {
        &self.title.version
    }
}
