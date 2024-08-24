use std::mem;

use indexmap::{IndexMap, IndexSet};

use crate::{ChangeLog, Release, ReleaseSectionNote};

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub sort_options: SortOptions,
}

impl ChangeLog {
    pub fn sanitize(&mut self, options: &Options) {
        if let Some(unreleased) = &mut self.unreleased {
            unreleased.deduplicate();
            unreleased.remove_empty();
            unreleased.sort_notes(&options.sort_options);
        }

        for release in self.releases.values_mut() {
            release.deduplicate();
            release.remove_empty();
            release.sort_notes(&options.sort_options);
        }

        self.unreleased_or_default();
    }

    pub fn deduplicate(&mut self) {
        for release in self.releases.values_mut() {
            release.deduplicate();
        }
    }
}

impl Release {
    pub fn deduplicate(&mut self) {
        for (_, sec) in &mut self.note_sections {
            let mut deduplicator = IndexSet::new();

            for n in sec.notes.drain(..) {
                deduplicator.insert(n);
            }

            sec.notes.extend(deduplicator);
        }
    }

    pub fn remove_empty(&mut self) {
        self.note_sections.retain(|_, section| {
            section.notes.retain(|n| !n.message.is_empty());

            !section.notes.is_empty()
        });
    }
}

#[derive(Debug, Clone)]
pub struct SortOptions {
    pub section_order: Vec<String>,
    pub sort_scope: bool,
}

impl Default for SortOptions {
    fn default() -> Self {
        Self {
            section_order: Default::default(),
            sort_scope: true,
        }
    }
}

impl Release {
    /// This will sort the section using section_order, and also group notes by scope.
    pub fn sort_notes(&mut self, options: &SortOptions) {
        self.note_sections = {
            let mut sorted = IndexMap::new();

            let mut section_cloned = self.note_sections.clone();

            for section in &options.section_order {
                if let Some((key, section)) = section_cloned.shift_remove_entry(section) {
                    sorted.insert(key, section);
                }
            }

            sorted.extend(section_cloned);
            sorted
        };

        if options.sort_scope {
            for (_, section) in &mut self.note_sections {
                let mut scoped: IndexMap<String, Vec<ReleaseSectionNote>> = IndexMap::new();

                let len = section.notes.len();

                let prev = mem::replace(&mut section.notes, Vec::with_capacity(len));

                let mut note_without_scope = Vec::new();

                for note in prev {
                    match &note.scope {
                        Some(scope) => match scoped.get_mut(scope) {
                            Some(notes) => {
                                notes.push(note);
                            }
                            None => {
                                scoped.insert(scope.clone(), vec![note]);
                            }
                        },
                        None => note_without_scope.push(note),
                    }
                }

                scoped.sort_by(|_k1, v1, _k2, v2| v2.len().cmp(&v1.len()));

                section.notes.extend(
                    scoped
                        .into_values()
                        .flat_map(|notes| notes.into_iter())
                        .chain(note_without_scope),
                )
            }
        }
    }
}
