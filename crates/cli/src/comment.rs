use openspec_doc_core::Project;
use openspec_doc_core::comments::{self, ScopeKey, Status, Thread};

use crate::error::Error;

/// Anchor a comment to `selected_text` in `artifact` and record it under
/// `scope`, printing the new comment's id and where it landed.
pub fn add(
    project: Project,
    scope: ScopeKey,
    artifact: &str,
    selected_text: &str,
    body: &str,
) -> Result<(), Error> {
    let comment = comments::add(&project.root, &scope, artifact, selected_text, body)?;
    let anchor = &comment.anchor;

    println!("added comment {}", comment.id);
    println!("  artifact: {}", anchor.artifact_path);
    println!("  offsets: {}..{}", anchor.start_offset, anchor.end_offset);
    if !anchor.heading_path.is_empty() {
        println!("  under: {}", anchor.heading_path.join(" > "));
    }

    Ok(())
}

/// Print every comment under `scope` with its status, and how confidently its
/// anchor still resolves against the artifact as it stands now.
pub fn list(project: Project, scope: ScopeKey) -> Result<(), Error> {
    let threads = comments::read(&project.root, &scope)?;

    println!("comments ({}):", threads.len());
    for thread in &threads {
        print_thread(&project, thread)?;
    }

    Ok(())
}

/// Append `body` to `comment_id`'s thread under `scope`.
pub fn reply(project: Project, scope: ScopeKey, comment_id: &str, body: &str) -> Result<(), Error> {
    let reply = comments::reply(&project.root, &scope, comment_id, body)?;

    println!("added reply {} to comment {}", reply.id, reply.comment_id);

    Ok(())
}

/// Record `comment_id` under `scope` as resolved.
pub fn resolve(project: Project, scope: ScopeKey, comment_id: &str) -> Result<(), Error> {
    let update = comments::resolve(&project.root, &scope, comment_id)?;

    println!("resolved comment {}", update.comment_id);

    Ok(())
}

fn print_thread(project: &Project, thread: &Thread) -> Result<(), Error> {
    let anchor = &thread.comment.anchor;
    let resolution = comments::resolve_anchor(&project.root, anchor)?;
    let status = match thread.status {
        Status::Open => "open",
        Status::Resolved => "resolved",
    };

    println!();
    println!("  {} [{status}]", thread.comment.id);
    println!("    artifact: {}", anchor.artifact_path);
    println!(
        "    anchor: {} {}",
        resolution.state,
        match resolution.offset {
            Some(offset) => format!("at offset {offset}"),
            None => "nowhere in the artifact".to_owned(),
        }
    );
    println!("    selected: {:?}", anchor.selected_text);
    println!("    {}", thread.comment.body);
    for reply in &thread.replies {
        println!("    reply: {}", reply.body);
    }

    Ok(())
}
