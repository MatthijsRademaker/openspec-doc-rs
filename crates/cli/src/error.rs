use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] openspec_doc_core::Error),

    #[error(transparent)]
    Server(#[from] openspec_doc_server::Error),

    #[error("failed to determine the current directory")]
    CurrentDir {
        #[source]
        source: io::Error,
    },

    #[error("failed to read the hook payload from stdin")]
    Stdin {
        #[source]
        source: io::Error,
    },
}
