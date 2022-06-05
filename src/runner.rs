use std::path::{Path, PathBuf};
use themelio_stf::melvm;
use thiserror::Error;

/// A high-level runner for Melodeon files.
pub struct Runner {
    /// Contents of a valid Melodeon program.
    src_contents: String,
    /// Path to that valid Melodeon program.
    src_path: PathBuf,
}

#[derive(Error, Debug)]
pub enum LoadFileError {
    #[error("could not read file: {0:?}")]
    IoError(std::io::Error),
    #[error("cannot compile Melodeon covenant: {0:?}")]
    MeloError(melodeon::context::CtxErr),
}

#[derive(Error, Debug)]
pub enum ReplError {
    #[error("cannot compile Melodeon covenant: {0:?}")]
    MeloError(melodeon::context::CtxErr),
    #[error("MelVM execution failed")]
    VmError,
}

impl Runner {
    /// Create a new Runner with nothing loaded.
    pub fn new() -> Self {
        Self {
            src_contents: "0".into(),
            src_path: PathBuf::from("."),
        }
    }

    /// Loads a file containing a Melodeon program, returning the  
    pub fn load_program(&mut self, path: &Path) -> Result<melvm::Value, LoadFileError> {
        let melo_str = std::fs::read_to_string(path).map_err(LoadFileError::IoError)?;

        todo!()
    }

    /// Runs a REPL line, which may be a definition or an expression.
    pub fn run_repl_line(&mut self, line: &str) -> Result<Option<melvm::Value>, ReplError> {
        todo!()
    }
}
