//! Decomposing markdown artifacts into rendered, anchorable blocks.
//!
//! Every block keeps two representations of the same source range: HTML for the
//! dashboard and the exact markdown bytes that produced it. The latter is sliced
//! from the input rather than reconstructed from parser events so an anchor can
//! search for it verbatim.

use std::ops::Range;

use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

/// A rendered text segment and the source bytes that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    /// Text as it appears in the rendered block.
    pub text: String,
    /// Absolute byte range in the artifact source.
    pub source_start: usize,
    pub source_end: usize,
}

/// One rendered, commentable region of an artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Stable for the same source range in the same artifact.
    pub id: String,
    /// Sanitised HTML rendered from the block's parser events.
    pub html: String,
    /// Exact source bytes occupied by [`range`].
    pub source: String,
    /// Rendered text segments mapped back to source ranges.
    pub source_spans: Vec<SourceSpan>,
    /// Byte range of [`source`] in the artifact.
    pub range: Range<usize>,
}

/// Decompose `markdown` into rendered blocks that can be sent back to the
/// anchor writer without losing their source position.
pub fn blocks(markdown: &str) -> Vec<Block> {
    let events: Vec<_> = parser(markdown).into_offset_iter().collect();
    let mut candidates = Vec::new();
    let mut stack = Vec::new();

    for (event_index, (event, range)) in events.iter().enumerate() {
        match event {
            Event::Start(tag) => {
                if let Some(kind) = block_kind(tag, &stack) {
                    stack.push(Frame {
                        kind,
                        event_start: event_index,
                        source_start: range.start,
                    });
                }
            }
            Event::End(end) if stack.last().is_some_and(|frame| frame.kind.closes(end)) => {
                let frame = stack.pop().expect("checked block frame");
                candidates.push(Candidate {
                    event_range: frame.event_start..event_index + 1,
                    source_range: frame.source_start..range.end,
                });
            }
            _ => {}
        }
    }

    candidates.sort_by_key(|candidate| {
        (
            candidate.source_range.start,
            candidate.source_range.end,
            candidate.event_range.start,
        )
    });

    candidates
        .into_iter()
        .map(|candidate| {
            let block_events = &events[candidate.event_range];
            let source = source(markdown, &candidate.source_range);
            let html = render(block_events);
            let source_spans = source_spans(block_events);

            Block {
                id: format!(
                    "block-{}-{}",
                    candidate.source_range.start, candidate.source_range.end
                ),
                html,
                source,
                source_spans,
                range: candidate.source_range,
            }
        })
        .collect()
}

fn parser(markdown: &str) -> Parser<'_> {
    Parser::new_ext(markdown, Options::all())
}

#[derive(Debug)]
struct Frame {
    kind: FrameKind,
    event_start: usize,
    source_start: usize,
}

/// A table and a code fence are single blocks rather than rows and lines: a
/// reviewer reads an aligned table and an ASCII diagram as one thing, and
/// selecting text inside a block already anchors a comment more finely than the
/// block itself.
#[derive(Debug)]
enum FrameKind {
    Paragraph,
    Heading,
    ListItem,
    Table,
    Html,
    CodeFence,
}

impl FrameKind {
    fn closes(&self, end: &TagEnd) -> bool {
        matches!(
            (self, end),
            (Self::Paragraph, TagEnd::Paragraph)
                | (Self::Heading, TagEnd::Heading(_))
                | (Self::ListItem, TagEnd::Item)
                | (Self::Table, TagEnd::Table)
                | (Self::Html, TagEnd::HtmlBlock)
                | (Self::CodeFence, TagEnd::CodeBlock)
        )
    }
}

#[derive(Debug)]
struct Candidate {
    event_range: Range<usize>,
    source_range: Range<usize>,
}

fn block_kind(tag: &Tag<'_>, stack: &[Frame]) -> Option<FrameKind> {
    match tag {
        Tag::Paragraph if !inside_list_item(stack) && !inside_table(stack) => {
            Some(FrameKind::Paragraph)
        }
        Tag::Heading { .. } => Some(FrameKind::Heading),
        Tag::Item => Some(FrameKind::ListItem),
        Tag::Table(_) => Some(FrameKind::Table),
        Tag::HtmlBlock => Some(FrameKind::Html),
        Tag::CodeBlock(CodeBlockKind::Fenced(_)) => Some(FrameKind::CodeFence),
        _ => None,
    }
}

fn inside_list_item(stack: &[Frame]) -> bool {
    stack
        .iter()
        .any(|frame| matches!(&frame.kind, FrameKind::ListItem))
}

fn inside_table(stack: &[Frame]) -> bool {
    stack
        .iter()
        .any(|frame| matches!(&frame.kind, FrameKind::Table))
}

fn source(markdown: &str, range: &Range<usize>) -> String {
    markdown
        .get(range.clone())
        .expect("pulldown-cmark reported an invalid UTF-8 source range")
        .to_owned()
}

fn render(events: &[(Event<'_>, Range<usize>)]) -> String {
    let mut html = String::new();
    push_html(&mut html, events.iter().map(|(event, _)| safe_event(event)));
    html
}

/// Keep the text a reviewer can select tied to the source bytes that produced it.
///
/// Markdown syntax disappears from rendered text, so the browser cannot send its
/// selection straight back as a source substring. These spans let the client
/// recover the source range while keeping HTML rendering unchanged.
fn source_spans(events: &[(Event<'_>, Range<usize>)]) -> Vec<SourceSpan> {
    let mut footnote_number = 0;

    events
        .iter()
        .filter_map(|(event, range)| {
            let text = match event {
                Event::Text(text)
                | Event::Code(text)
                | Event::InlineMath(text)
                | Event::DisplayMath(text)
                | Event::Html(text)
                | Event::InlineHtml(text) => text.to_string(),
                Event::FootnoteReference(_) => {
                    footnote_number += 1;
                    footnote_number.to_string()
                }
                // These events produce a newline in the HTML renderer. The
                // source range still gives the client a valid anchor boundary.
                Event::SoftBreak | Event::HardBreak | Event::Rule | Event::TaskListMarker(_) => {
                    "\n".to_owned()
                }
                Event::Start(_) | Event::End(_) => return None,
            };

            (!text.is_empty()).then_some(SourceSpan {
                text,
                source_start: range.start,
                source_end: range.end,
            })
        })
        .collect()
}

/// Convert raw HTML into text before pulldown-cmark's HTML renderer sees it.
/// Passing it through as `Event::Html` or `Event::InlineHtml` would emit agent-
/// written markup into the reviewer's browser.
fn safe_event<'a>(event: &Event<'a>) -> Event<'a> {
    match event {
        Event::Html(value) | Event::InlineHtml(value) => Event::Text(value.clone()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn assert_sources_are_slices(markdown: &str, blocks: &[Block]) {
        for block in blocks {
            assert_eq!(
                markdown.get(block.range.clone()),
                Some(block.source.as_str()),
                "{} does not carry its reported source slice",
                block.id
            );
            assert!(
                markdown.find(&block.source).is_some(),
                "{} source {:?} was not found",
                block.id,
                block.source
            );
        }
    }

    #[test]
    fn nested_lists_are_split_into_list_item_blocks() {
        let markdown = "- outer\n  - inner\n\n- sibling\n";
        let blocks = blocks(markdown);

        let sources: Vec<_> = blocks.iter().map(|block| block.source.as_str()).collect();
        assert!(sources.iter().any(|source| source.contains("outer")));
        assert!(sources.iter().any(|source| source.contains("inner")));
        assert!(sources.iter().any(|source| source.contains("sibling")));
        assert_sources_are_slices(markdown, &blocks);
    }

    #[test]
    fn a_table_is_one_block_holding_the_whole_table() {
        let markdown = "| A | B |\n| --- | --- |\n| one | two |\n";
        let blocks = blocks(markdown);

        assert_eq!(blocks.len(), 1, "{blocks:?}");
        let table = &blocks[0];
        assert_eq!(table.source, markdown);
        assert!(table.html.starts_with("<table>"), "{}", table.html);
        assert_eq!(table.html.matches("<tr>").count(), 2, "{}", table.html);
        assert!(table.html.contains("<th>A</th>"), "{}", table.html);
        assert!(table.html.contains("<td>one</td>"), "{}", table.html);
        assert_sources_are_slices(markdown, &blocks);
    }

    #[test]
    fn a_fenced_code_block_is_one_block_keeping_every_line() {
        let markdown = "```rust\nlet one = 1;\nlet two = 2;\n```\n";
        let blocks = blocks(markdown);

        assert_eq!(blocks.len(), 1, "{blocks:?}");
        let fence = &blocks[0];
        assert_eq!(fence.source, "```rust\nlet one = 1;\nlet two = 2;\n```");
        assert_eq!(
            fence.html.matches("<pre><code").count(),
            1,
            "{}",
            fence.html
        );
        assert!(
            fence
                .html
                .contains("let one = 1;\nlet two = 2;\n</code></pre>"),
            "{}",
            fence.html
        );
        assert_sources_are_slices(markdown, &blocks);
    }

    /// The diagrams in this project's own design documents are the reason the
    /// fence is one block: a per-line split renders each line as its own code
    /// element and the drawing stops being a drawing.
    #[test]
    fn an_ascii_diagram_keeps_its_interior_blank_lines() {
        let markdown = "```\n  a\n\n  b\n```\n";
        let blocks = blocks(markdown);

        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(
            blocks[0].html.contains("  a\n\n  b\n"),
            "{}",
            blocks[0].html
        );
    }

    #[test]
    fn setext_headings_keep_their_full_source_range() {
        let markdown = "A heading\n=========\n\nA paragraph.\n";
        let blocks = blocks(markdown);

        let heading = blocks.first().expect("heading block");
        assert_eq!(heading.source, "A heading\n=========\n");
        assert!(
            heading.html.starts_with("<h1>A heading</h1>"),
            "{heading:?}"
        );
        assert_sources_are_slices(markdown, &blocks);
    }

    #[test]
    fn rendered_text_spans_keep_inline_markup_mapped_to_source() {
        let markdown = "A **bold** and `code`.\n";
        let parsed = blocks(markdown);
        let block = parsed.first().expect("paragraph block");
        let visible: String = block
            .source_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();

        assert_eq!(visible, "A bold and code.");
        let code = block
            .source_spans
            .iter()
            .find(|span| span.text == "code")
            .expect("inline code span");
        assert_eq!(
            markdown.get(code.source_start..code.source_end),
            Some("`code`")
        );
    }

    #[test]
    fn an_html_block_is_rendered_as_text() {
        let markdown = "<div>not markup</div>\n\nAfter.\n";
        let blocks = blocks(markdown);

        let html = &blocks.first().expect("html block").html;
        assert!(!html.contains("<div>"), "{html}");
        assert!(html.contains("&lt;div&gt;not markup&lt;/div&gt;"), "{html}");
        assert_sources_are_slices(markdown, &blocks);
    }

    #[test]
    fn a_script_element_reaches_the_caller_as_escaped_text() {
        let markdown = "<script>alert(1)</script>\n";
        let parsed = blocks(markdown);
        let block = parsed.first().expect("script block");

        assert_eq!(block.source, markdown);
        assert!(
            block.html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
            "{}",
            block.html
        );
        assert!(!block.html.contains("<script>"), "{}", block.html);
    }

    #[test]
    fn every_real_openspec_artifact_has_findable_block_sources() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../openspec");
        let mut paths = Vec::new();
        markdown_files(&root, &mut paths);
        assert!(!paths.is_empty(), "no real OpenSpec artifacts found");

        for path in paths {
            let markdown = fs::read_to_string(&path).expect("read real artifact");
            let blocks = blocks(&markdown);
            assert_sources_are_slices(&markdown, &blocks);
        }
    }

    fn markdown_files(dir: &Path, paths: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read OpenSpec tree") {
            let path = entry.expect("read directory entry").path();
            if path.is_dir() {
                markdown_files(&path, paths);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                paths.push(path);
            }
        }
    }
}
