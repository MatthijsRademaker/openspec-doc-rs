//! A scope's human-readable title, read out of the scratch note the agent is
//! already keeping current.
//!
//! Nothing writes a title: it is the note's own opening heading. That costs no
//! new state and no new command, and it stays accurate for free — but it also
//! means a scope with nothing written down has no title, and reporting a
//! synthesised one would be a guess the reviewer had to learn to distrust.

use std::path::Path;

use crate::comments::parse_heading;
use crate::error::Error;

use super::note::{read_change, read_session};

/// What opens and closes a fenced code block. A `#` inside one is markdown the
/// note is quoting, not a heading it is writing.
const FENCES: [&str; 2] = ["```", "~~~"];

/// The session's title, or `None` when its note is absent, empty, or does not
/// open with a level-one heading.
pub fn session_title(root: &Path, session_id: &str) -> Result<Option<String>, Error> {
    Ok(read_session(root, session_id)?
        .as_deref()
        .and_then(opening_heading))
}

/// The title of change `name`, taken from the note its exploration was promoted
/// into. A change with no note falls back to being known by its name, which is
/// already readable in a way a session id is not.
pub fn change_title(root: &Path, name: &str) -> Result<Option<String>, Error> {
    Ok(read_change(root, name)?
        .as_deref()
        .and_then(opening_heading))
}

/// The text of `markdown`'s first heading when that heading is level one.
///
/// The *first* heading, rather than the first level-one heading anywhere: a note
/// that opens at level two is one whose author did not write it a title, and
/// promoting some `#` further down would title the note after one of its
/// sections.
fn opening_heading(markdown: &str) -> Option<String> {
    let mut fenced = false;

    for line in markdown.lines() {
        let line = line.trim();

        if FENCES.iter().any(|fence| line.starts_with(fence)) {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }

        if let Some((level, title)) = parse_heading(line) {
            return (level == 1).then(|| title.to_owned());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::super::note::prepare_session;
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn note(contents: &str) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            contents,
        )
        .expect("write note");
        temp
    }

    fn title_of(contents: &str) -> Option<String> {
        session_title(note(contents).path(), "session-a").expect("title")
    }

    #[test]
    fn a_notes_opening_heading_is_its_title() {
        assert_eq!(
            title_of("#   Exploring: the dashboard's look and feel  \n\nProse.\n"),
            Some("Exploring: the dashboard's look and feel".to_owned())
        );
    }

    /// A note that opens at level two was never given a title, and titling it
    /// after a section further down would misdescribe it.
    #[test]
    fn a_note_that_opens_below_level_one_has_no_title() {
        assert_eq!(title_of("## Context\n\nProse.\n\n# Late heading\n"), None);
    }

    #[test]
    fn a_heading_inside_a_fence_is_not_a_title() {
        assert_eq!(
            title_of("Prose first.\n\n```sh\n# Example\n```\n\n# The real title\n"),
            Some("The real title".to_owned())
        );
        assert_eq!(
            title_of("~~~\n# Example\n~~~\n"),
            None,
            "a note made only of a quoted example has no title of its own"
        );
    }

    #[test]
    fn a_note_with_no_heading_at_all_has_no_title() {
        assert_eq!(title_of("Just prose, never headed.\n"), None);
        assert_eq!(title_of(""), None);
    }

    /// An absent note is a session that has written nothing down yet, which is
    /// every session before its exploration starts — not a failure.
    #[test]
    fn a_session_with_no_note_has_no_title_and_no_error() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(
            session_title(temp.path(), "session-a").expect("title"),
            None
        );
    }

    #[test]
    fn a_promoted_note_titles_its_change() {
        let temp = TempDir::new().expect("temp dir");
        let path = super::super::change_path(temp.path(), "add-thing");
        fs::create_dir_all(path.parent().expect("parent")).expect("create scratch dir");
        fs::write(&path, "# Exploring: titles\n").expect("write note");

        assert_eq!(
            change_title(temp.path(), "add-thing").expect("title"),
            Some("Exploring: titles".to_owned())
        );
        assert_eq!(
            change_title(temp.path(), "add-other").expect("title"),
            None,
            "a change with no exploration behind it has no note to read"
        );
    }
}
