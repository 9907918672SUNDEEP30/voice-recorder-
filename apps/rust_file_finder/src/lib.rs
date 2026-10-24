//! # Sovereign Fast File Finder Core Library
//!
//! 100% Free & Open Source Sub-Millisecond File Search Engine.

pub mod renamer;
pub mod scanner;

pub use renamer::rename_file_on_disk;
pub use scanner::{FastDirectoryScanner, IndexedFileRecord};
