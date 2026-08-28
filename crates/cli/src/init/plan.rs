//! What `init` intends to do, as a value.
//!
//! The plan is computed in full — every file read, parsed, and its final content
//! rendered — before a single byte is written. So a malformed settings file or a
//! half-delimited `AGENTS.md` fails with the project untouched, and the dry run
//! and the real run agree by construction rather than by two code paths staying
//! in step.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use openspec_doc_core::hook::Agent;

use crate::error::Error;

/// What performing a planned write would do to the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Create,
    Modify,
    Unchanged,
}

impl Kind {
    fn planned(self) -> &'static str {
        match self {
            Self::Create => "create   ",
            Self::Modify => "modify   ",
            Self::Unchanged => "unchanged",
        }
    }

    fn performed(self) -> &'static str {
        match self {
            Self::Create => "created  ",
            Self::Modify => "modified ",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug)]
pub struct Write {
    path: PathBuf,
    display: String,
    kind: Kind,
    note: &'static str,
    contents: String,
}

impl Write {
    /// A write of `contents` to `path`, its kind decided by what is there now.
    pub fn new(root: &Path, path: PathBuf, note: &'static str, contents: String) -> Self {
        let kind = match fs::read_to_string(&path) {
            Ok(current) if current == contents => Kind::Unchanged,
            Ok(_) => Kind::Modify,
            Err(_) => Kind::Create,
        };
        let display = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();

        Self {
            path,
            display,
            kind,
            note,
            contents,
        }
    }
}

#[derive(Debug)]
pub struct Plan {
    pub root: PathBuf,
    pub agents: Vec<Agent>,
    pub writes: Vec<Write>,
    pub warnings: Vec<String>,
}

impl Plan {
    /// The plan as the operator reads it. Identical whether or not `--yes` was
    /// given and whether or not a terminal is attached, so the tests exercise
    /// what a person sees.
    pub fn render(&self) -> String {
        let mut out = String::new();

        let _ = writeln!(out, "project: {}", self.root.display());
        let agents: Vec<String> = self.agents.iter().map(Agent::to_string).collect();
        let _ = writeln!(out, "harnesses: {}", agents.join(", "));

        let _ = writeln!(out, "\nfiles ({}):", self.writes.len());
        for write in &self.writes {
            let _ = writeln!(
                out,
                "  {} {}  {}",
                write.kind.planned(),
                write.display,
                write.note
            );
        }

        if !self.warnings.is_empty() {
            let _ = writeln!(out, "\nwarnings ({}):", self.warnings.len());
            for warning in &self.warnings {
                let _ = writeln!(out, "  {warning}");
            }
        }

        out
    }

    /// Perform every write the plan named, reporting each one.
    pub fn perform(&self) -> Result<String, Error> {
        let mut out = String::from("\nwrote:\n");

        for write in &self.writes {
            if write.kind != Kind::Unchanged {
                if let Some(parent) = write.path.parent() {
                    fs::create_dir_all(parent).map_err(|source| Error::Write {
                        path: parent.to_owned(),
                        source,
                    })?;
                }
                fs::write(&write.path, &write.contents).map_err(|source| Error::Write {
                    path: write.path.clone(),
                    source,
                })?;
            }

            let _ = writeln!(out, "  {} {}", write.kind.performed(), write.display);
        }

        Ok(out)
    }
}
