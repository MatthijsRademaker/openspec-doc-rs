//! Rendering one artifact for review.
//!
//! The markdown is shown as its own source rather than converted to HTML, so
//! that what the reviewer selects in the browser is byte-for-byte a substring of
//! the file the anchor is created against. Anchoring is the whole point of the
//! page, and this is what keeps a selection from having to be mapped back from
//! rendered HTML onto source offsets.

use crate::scope::Artifact;

use super::escape;

/// Every artifact of a scope, or a notice when the scope has none on disk yet.
pub(super) fn list(artifacts: &[Artifact]) -> String {
    if artifacts.is_empty() {
        return "<p class=\"empty\">No artifact on disk to review yet.</p>".to_owned();
    }

    artifacts.iter().map(render).collect()
}

fn render(artifact: &Artifact) -> String {
    format!(
        "<article class=\"artifact\" data-artifact-path=\"{path}\">\n\
         <h2>{path}</h2>\n\
         <pre class=\"source\">{markdown}</pre>\n\
         </article>\n",
        path = escape(&artifact.path),
        markdown = escape(&artifact.markdown)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(path: &str, markdown: &str) -> Artifact {
        Artifact {
            path: path.to_owned(),
            markdown: markdown.to_owned(),
        }
    }

    #[test]
    fn an_artifact_renders_its_path_and_its_source() {
        let html = list(&[artifact(
            "openspec/changes/add-a/proposal.md",
            "## Why\n\nBecause of X.\n",
        )]);

        assert!(html.contains("data-artifact-path=\"openspec/changes/add-a/proposal.md\""));
        assert!(
            html.contains("## Why\n\nBecause of X.\n"),
            "the source is rendered as written, got {html}"
        );
    }

    /// The markdown is written by an agent and read straight into the page, so a
    /// note containing markup must not be able to close the element around it.
    #[test]
    fn markup_in_the_source_is_escaped() {
        let html = list(&[artifact("note.md", "</pre><script>alert(1)</script>")]);

        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;/pre&gt;"), "{html}");
    }

    #[test]
    fn a_scope_with_no_artifacts_says_so() {
        assert!(list(&[]).contains("No artifact on disk"));
    }
}
