//! The page's client side: enough vanilla JavaScript to turn a text selection
//! into a comment and to swap the review state when the server pushes, and no
//! more. No framework, no build step.

/// Selection capture and the live-update swap.
///
/// A selection is read straight off the rendered artifact, which shows the
/// markdown source verbatim — so `selection.toString()` is already a substring
/// of the file the server will anchor against, with no offset mapping to get
/// wrong. The server re-finds the text in the file itself, so a selection that
/// has gone stale is refused there rather than anchored to a guess here.
///
/// The update path refetches the review fragment and swaps it in, rather than
/// reloading: a reviewer part-way through typing a comment must not lose it
/// because someone else commented.
pub(super) const REVIEW: &str = r#"
const composer = document.querySelector("[data-composer]");
const artifactOf = (node) => {
  const element = node?.nodeType === Node.TEXT_NODE ? node.parentElement : node;
  return element?.closest ? element.closest(".artifact") : null;
};

document.addEventListener("selectionchange", () => {
  if (!composer) return;
  const selection = document.getSelection();
  const text = selection ? selection.toString() : "";
  const artifact = selection && selection.rangeCount > 0 ? artifactOf(selection.anchorNode) : null;
  if (!artifact || text.trim() === "") return;

  composer.querySelector("[data-composer-artifact]").value = artifact.dataset.artifactPath;
  composer.querySelector("[data-composer-selection]").value = text;
  composer.querySelector("[data-composer-context]").textContent =
    artifact.dataset.artifactPath + " · " + text.slice(0, 120);
  composer.hidden = false;
});

composer?.querySelector("[data-composer-cancel]").addEventListener("click", () => {
  composer.hidden = true;
});

const review = document.body.dataset.review;
const events = document.body.dataset.events;
if (events && review) {
  new EventSource(events).onmessage = async () => {
    const response = await fetch(review);
    if (!response.ok) return;
    const container = document.getElementById(document.body.dataset.reviewContainer);
    if (container) container.outerHTML = await response.text();
  };
}
"#;

/// Enough style to tell an artifact, a comment and a form apart. The source is
/// shown as written, so it wraps rather than scrolling off the page.
pub(super) const STYLE: &str = r#"
body { font: 15px/1.5 system-ui, sans-serif; margin: 0 auto; max-width: 60rem; padding: 1rem 2rem; }
.artifact { border: 1px solid #d0d0d0; border-radius: 4px; margin: 1rem 0; padding: 0 1rem 1rem; }
.artifact h2 { font: 600 13px ui-monospace, monospace; color: #555; }
pre.source { white-space: pre-wrap; word-break: break-word; margin: 0; }
.comments, .verdicts { list-style: none; padding: 0; }
.comment, .verdicts li { border-left: 3px solid #d0d0d0; margin: 0.75rem 0; padding: 0 0 0 0.75rem; }
.comment[data-anchor-state="fuzzy"] { border-left-color: #c88a00; }
.comment[data-anchor-state="orphaned"], .comment[data-anchor-state="missing"] { border-left-color: #b00; }
.meta { color: #666; font-size: 13px; margin: 0.25rem 0; }
.empty { color: #666; }
blockquote { border-left: 2px solid #ccc; color: #333; margin: 0.25rem 0; padding-left: 0.75rem; }
form.composer, form.verdict { border: 1px solid #d0d0d0; border-radius: 4px; margin: 1rem 0; padding: 1rem; }
textarea { display: block; width: 100%; margin: 0.5rem 0; }
"#;
