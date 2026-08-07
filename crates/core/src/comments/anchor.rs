//! Attaching a comment to a span of markdown, and finding that span again after
//! the markdown around it has changed. Pure text handling: no filesystem access.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::Error;

/// How much text on either side of the selection is kept as context, in bytes.
const CONTEXT_LENGTH: usize = 80;

/// Where a comment is attached within an artifact, captured richly enough to be
/// re-found once the surrounding text has moved or changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    /// The commented artifact, relative to the project root.
    pub artifact_path: String,
    /// The commented text itself, trimmed.
    pub selected_text: String,
    /// The markdown headings enclosing the selection, outermost first.
    pub heading_path: Vec<String>,
    /// The text immediately before the selection, and immediately after it.
    pub before_text: String,
    pub after_text: String,
    /// Byte offsets of the selection in the markdown it was created against.
    pub start_offset: usize,
    pub end_offset: usize,
}

/// How confidently an anchor was located in the markdown as it stands now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnchorState {
    /// The recorded offset still holds the selected text.
    Exact,
    /// The selected text, or its context, was found somewhere else.
    Fuzzy,
    /// The artifact is there, but nothing in it matches the anchor.
    Orphaned,
    /// The artifact itself is gone.
    Missing,
    /// There was never an anchor: the comment is scoped to the session or change
    /// as a whole. Reported apart from `Orphaned`, which means an anchor was
    /// lost and is a defect the reviewer should see.
    Unanchored,
}

/// Where an anchor lands in the current markdown, and how sure that is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub state: AnchorState,
    /// The byte offset the anchor resolved to; `None` when it did not resolve.
    pub offset: Option<usize>,
}

impl fmt::Display for AnchorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Exact => "exact",
            Self::Fuzzy => "fuzzy",
            Self::Orphaned => "orphaned",
            Self::Missing => "missing",
            Self::Unanchored => "unanchored",
        };
        f.write_str(label)
    }
}

/// Anchor `selected_text` within `markdown`, recording its heading path,
/// surrounding context, and offsets.
///
/// Selected text that is absent from the markdown is an error rather than a
/// guessed-at anchor: the caller is describing a selection that does not exist.
/// Search begins at `search_from`, allowing a caller with a parser range to name
/// the intended occurrence of repeated text. Passing zero keeps first-occurrence
/// behaviour for callers that do not know the selection's source position.
pub fn create(
    artifact_path: &str,
    markdown: &str,
    selected_text: &str,
    search_from: usize,
) -> Result<Anchor, Error> {
    let selected_text = selected_text.trim();
    if selected_text.is_empty() {
        return Err(Error::EmptySelection);
    }

    let Some(start_offset) = markdown
        .get(search_from..)
        .and_then(|remaining| remaining.find(selected_text))
        .map(|offset| search_from + offset)
    else {
        return Err(Error::SelectionNotFound {
            artifact_path: artifact_path.to_owned(),
            selected_text: selected_text.to_owned(),
        });
    };
    let end_offset = start_offset + selected_text.len();

    Ok(Anchor {
        artifact_path: artifact_path.to_owned(),
        selected_text: selected_text.to_owned(),
        heading_path: heading_path_before(markdown, start_offset),
        before_text: slice(
            markdown,
            start_offset.saturating_sub(CONTEXT_LENGTH),
            start_offset,
        ),
        after_text: slice(markdown, end_offset, end_offset + CONTEXT_LENGTH),
        start_offset,
        end_offset,
    })
}

/// Locate `anchor` in `markdown` as it stands now: the recorded offset first,
/// then the anchor's heading and surrounding text, and otherwise not at all.
///
/// `markdown` is `None` when the anchored artifact no longer exists, which is
/// reported apart from a failed search so a caller can tell the two apart.
pub fn resolve(anchor: &Anchor, markdown: Option<&str>) -> Resolution {
    let Some(markdown) = markdown else {
        return Resolution {
            state: AnchorState::Missing,
            offset: None,
        };
    };

    if let Some(offset) = find_exact(anchor, markdown) {
        return Resolution {
            state: AnchorState::Exact,
            offset: Some(offset),
        };
    }

    match find_fuzzy(anchor, markdown) {
        Some(offset) => Resolution {
            state: AnchorState::Fuzzy,
            offset: Some(offset),
        },
        None => Resolution {
            state: AnchorState::Orphaned,
            offset: None,
        },
    }
}

/// The recorded offset, when the selected text is still sitting there.
fn find_exact(anchor: &Anchor, markdown: &str) -> Option<usize> {
    let end = anchor.start_offset + anchor.selected_text.len();
    (markdown.get(anchor.start_offset..end)? == anchor.selected_text).then_some(anchor.start_offset)
}

/// The anchor's most plausible offset once its recorded one has gone stale: the
/// selected text searched from its heading, then the text that bracketed it, and
/// finally the heading alone.
fn find_fuzzy(anchor: &Anchor, markdown: &str) -> Option<usize> {
    let heading_offset = find_heading_offset(&anchor.heading_path, markdown);

    let search_start = heading_offset.unwrap_or(0);
    if let Some(offset) = markdown[search_start..]
        .find(&anchor.selected_text)
        .map(|found| search_start + found)
    {
        return Some(offset);
    }

    let before = anchor.before_text.trim();
    if let Some(offset) = (!before.is_empty())
        .then(|| markdown.find(before))
        .flatten()
    {
        return Some(offset + before.len());
    }

    let after = anchor.after_text.trim();
    if let Some(offset) = (!after.is_empty()).then(|| markdown.find(after)).flatten() {
        return Some(offset);
    }

    heading_offset
}

/// The heading path enclosing `offset`: each heading level's most recent title
/// before that point, outermost first. Levels that were skipped are dropped.
fn heading_path_before(markdown: &str, offset: usize) -> Vec<String> {
    let mut path: Vec<Option<String>> = Vec::new();

    for line in markdown[..offset].lines() {
        let Some((level, title)) = parse_heading(line.trim()) else {
            continue;
        };

        // Deeper levels are dropped and skipped ones left blank, so that a
        // heading only ever displaces its own level and the levels below it.
        path.resize(level - 1, None);
        path.push(Some(title.to_owned()));
    }

    path.into_iter().flatten().collect()
}

/// Where the anchor's innermost heading is written in `markdown`.
fn find_heading_offset(heading_path: &[String], markdown: &str) -> Option<usize> {
    let heading = heading_path.last()?;

    let mut offset = 0;
    for line in markdown.split('\n') {
        if parse_heading(line).is_some_and(|(_, title)| title == heading) {
            return Some(offset);
        }
        offset += line.len() + 1;
    }

    None
}

/// The level and title of an ATX heading line, or `None` when the line is not
/// one. The title is trimmed; a heading of seven or more `#` is not a heading.
pub(crate) fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }

    let rest = &line[level..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }

    let title = rest.trim();
    (!title.is_empty()).then_some((level, title))
}

/// `text[from..to]`, with both ends pulled back to a character boundary and
/// clamped to the text, so a byte window can never split a character.
fn slice(text: &str, from: usize, to: usize) -> String {
    text[floor_boundary(text, from)..floor_boundary(text, to)].to_owned()
}

fn floor_boundary(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The markdown the ported TypeScript tests anchor against.
    const ORIGINAL: &str =
        "# Proposal\n\n## Why\n\nAlpha before. Selected sentence. Omega after.\n";
    const ARTIFACT: &str = "openspec/changes/add-core/proposal.md";
    const SELECTED: &str = "Selected sentence.";

    fn anchor() -> Anchor {
        create(ARTIFACT, ORIGINAL, SELECTED, 0).expect("create anchor")
    }

    #[test]
    fn an_anchor_records_its_heading_path_context_and_offsets() {
        let anchor = anchor();

        assert_eq!(anchor.heading_path, ["Proposal", "Why"]);
        assert!(
            anchor.before_text.contains("Alpha before."),
            "before context: {:?}",
            anchor.before_text
        );
        assert!(
            anchor.after_text.contains("Omega after."),
            "after context: {:?}",
            anchor.after_text
        );
        assert_eq!(
            anchor.start_offset,
            ORIGINAL.find(SELECTED).expect("offset")
        );
        assert_eq!(anchor.end_offset, anchor.start_offset + SELECTED.len());
    }

    #[test]
    fn unchanged_markdown_resolves_exactly_at_the_recorded_offset() {
        let anchor = anchor();

        assert_eq!(
            resolve(&anchor, Some(ORIGINAL)),
            Resolution {
                state: AnchorState::Exact,
                offset: Some(ORIGINAL.find(SELECTED).expect("offset")),
            }
        );
    }

    #[test]
    fn text_pushed_to_a_new_offset_resolves_fuzzily_at_its_new_one() {
        let moved = "# Proposal\n\n## Why\n\nNew introduction.\n\nAlpha before. Selected sentence. Omega after.\n";

        let resolution = resolve(&anchor(), Some(moved));

        assert_eq!(resolution.state, AnchorState::Fuzzy);
        assert_eq!(
            resolution.offset,
            Some(moved.find(SELECTED).expect("offset"))
        );
    }

    #[test]
    fn rewritten_text_resolves_fuzzily_from_its_surrounding_context() {
        let changed = "# Proposal\n\n## Why\n\nAlpha before. Replacement text. Omega after.\n";

        let resolution = resolve(&anchor(), Some(changed));

        assert_eq!(resolution.state, AnchorState::Fuzzy);
        assert_eq!(
            resolution.offset,
            Some(changed.find(" Replacement text.").expect("offset")),
            "the anchor lands where the preceding context ends"
        );
    }

    #[test]
    fn markdown_with_nothing_recognizable_left_resolves_as_orphaned() {
        let rewritten = "Completely different proposal without useful context.";

        assert_eq!(
            resolve(&anchor(), Some(rewritten)),
            Resolution {
                state: AnchorState::Orphaned,
                offset: None,
            }
        );
    }

    #[test]
    fn an_absent_artifact_resolves_as_missing() {
        assert_eq!(
            resolve(&anchor(), None),
            Resolution {
                state: AnchorState::Missing,
                offset: None,
            }
        );
    }

    #[test]
    fn a_surviving_heading_alone_still_resolves_fuzzily() {
        let gutted = "# Proposal\n\n## Why\n\nEntirely new prose.\n";

        let resolution = resolve(&anchor(), Some(gutted));

        assert_eq!(resolution.state, AnchorState::Fuzzy);
        assert_eq!(
            resolution.offset,
            Some(gutted.find("## Why").expect("offset")),
            "the innermost heading is the last usable landmark"
        );
    }

    #[test]
    fn selected_text_absent_from_the_markdown_is_an_error() {
        let error = create(ARTIFACT, ORIGINAL, "never written", 0).expect_err("absent selection");

        assert!(matches!(error, Error::SelectionNotFound { .. }), "{error}");
    }

    #[test]
    fn an_empty_selection_is_an_error() {
        for selection in ["", "   \n\t"] {
            assert!(
                matches!(
                    create(ARTIFACT, ORIGINAL, selection, 0),
                    Err(Error::EmptySelection)
                ),
                "accepted {selection:?}"
            );
        }
    }

    #[test]
    fn a_skipped_heading_level_leaves_no_gap_in_the_path() {
        let markdown = "# Top\n\n### Deep\n\nSelected sentence.\n";

        let anchor = create(ARTIFACT, markdown, SELECTED, 0).expect("create anchor");

        assert_eq!(anchor.heading_path, ["Top", "Deep"]);
    }

    #[test]
    fn a_sibling_heading_replaces_its_predecessor_in_the_path() {
        let markdown = "# Top\n\n## First\n\n## Second\n\nSelected sentence.\n";

        let anchor = create(ARTIFACT, markdown, SELECTED, 0).expect("create anchor");

        assert_eq!(anchor.heading_path, ["Top", "Second"]);
    }

    #[test]
    fn seven_hashes_are_not_a_heading() {
        let markdown = "####### Not a heading\n\nSelected sentence.\n";

        let anchor = create(ARTIFACT, markdown, SELECTED, 0).expect("create anchor");

        assert!(anchor.heading_path.is_empty());
    }

    #[test]
    fn a_search_offset_selects_the_repeated_occurrence_and_its_heading() {
        let markdown = "# First\n\nRepeated item.\n\n# Second\n\nRepeated item.\n";
        let second_offset = markdown.rfind("Repeated item.").expect("second occurrence");

        let anchor = create(ARTIFACT, markdown, "Repeated item.", second_offset)
            .expect("create second anchor");

        assert_eq!(anchor.start_offset, second_offset);
        assert_eq!(anchor.heading_path, ["Second"]);
    }

    #[test]
    fn search_from_zero_keeps_first_occurrence_behaviour() {
        let markdown = "First occurrence.\n\nFirst occurrence.\n";
        let anchor =
            create(ARTIFACT, markdown, "First occurrence.", 0).expect("create first anchor");

        assert_eq!(
            anchor.start_offset,
            markdown.find("First occurrence.").expect("first")
        );
    }

    #[test]
    fn multibyte_context_is_captured_without_splitting_a_character() {
        // The em dashes sit exactly where an 80-byte context window would cut.
        let padding = "—".repeat(40);
        let markdown = format!("# Top\n\n{padding}Selected sentence.{padding}\n");

        let anchor = create(ARTIFACT, &markdown, SELECTED, 0).expect("create anchor");

        assert!(
            anchor.before_text.ends_with('—'),
            "{:?}",
            anchor.before_text
        );
        assert!(
            anchor.after_text.starts_with('—'),
            "{:?}",
            anchor.after_text
        );
        assert_eq!(resolve(&anchor, Some(&markdown)).state, AnchorState::Exact);
    }
}
