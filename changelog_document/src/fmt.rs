use std::mem;

use indexmap::{IndexMap, IndexSet};

use crate::{ChangeLog, Release, ReleaseSectionNote};

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub section_order: Vec<String>,
}

impl ChangeLog {
    pub fn sanitize(&mut self, options: Options) {
        for (_, release) in &mut self.releases {
            release.deduplicate();
            release.remove_empty();
            release.sort_notes(&options.section_order);
        }
    }

    pub fn deduplicate(&mut self) {
        for (_, release) in &mut self.releases {
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

    /// This will sort the section using section_order, and also group notes by scope.
    pub fn sort_notes<S, I>(&mut self, section_order: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.note_sections = {
            let mut sorted = IndexMap::new();

            let mut section_cloned = self.note_sections.clone();

            for section in section_order {
                if let Some((key, section)) = section_cloned.shift_remove_entry(section.as_ref()) {
                    sorted.insert(key, section);
                }
            }

            sorted.extend(section_cloned);
            sorted
        };

        for (_, section) in &mut self.note_sections {
            let mut scoped: IndexMap<String, Vec<ReleaseSectionNote>> = IndexMap::new();

            let len = section.notes.len();

            let prev = mem::replace(&mut section.notes, Vec::with_capacity(len));

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
                    None => section.notes.push(note),
                }
            }

            section
                .notes
                .extend(scoped.into_values().flat_map(|notes| notes.into_iter()));
        }
    }
}
