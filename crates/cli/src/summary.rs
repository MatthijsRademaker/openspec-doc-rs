use openspec_doc_core::{NamedDir, Project, scan};

use crate::error::Error;

/// Print the resolved root and the project's active changes, archived changes,
/// and current specs.
pub fn run(project: Project) -> Result<(), Error> {
    let result = scan(project)?;

    println!("root: {}", result.project.root.display());
    print_section("active changes", &result.active_changes);
    print_section("archived changes", &result.archived_changes);
    print_section("specs", &result.specs);

    Ok(())
}

fn print_section(label: &str, dirs: &[NamedDir]) {
    println!("\n{label} ({}):", dirs.len());
    for dir in dirs {
        println!("  {}", dir.name);
    }
}
