use derivative::Derivative;
use melodeon::typesys::Type;
use mil::compiler::{BinCode, Compile};
use std::path::{Path, PathBuf};
use themelio_stf::melvm::{self, Covenant, Executor};
use themelio_structs::{Denom, TxKind};
use thiserror::Error;

use crate::{EnvFile, SpendContext};

/// A high-level runner for Melodeon files.
pub struct Runner {
    /// Contents of a valid Melodeon program.
    src_contents: String,
    /// Path to that valid Melodeon program.
    src_path: PathBuf,
    /// Covenant spend context.
    ctx: SpendContext,
}

#[derive(Error, Derivative)]
#[derivative(Debug)]
pub enum LoadFileError {
    #[error("could not read file: {0:?}")]
    IoError(std::io::Error),
    #[error("cannot compile Melodeon covenant: {0:?}")]
    MeloError(melodeon::context::CtxErr),
    #[error("MelVM execution failed")]
    VmError(#[derivative(Debug = "ignore")] Executor),
}

#[derive(Error, Derivative)]
#[derivative(Debug)]
pub enum ReplError {
    #[error("cannot compile Melodeon covenant: {0:?}")]
    MeloError(melodeon::context::CtxErr),
    #[error("MelVM execution failed")]
    VmError(#[derivative(Debug = "ignore")] Executor),
}

impl Runner {
    /// Create a new Runner with nothing loaded.
    pub fn new(ctx: Option<SpendContext>) -> Self {
        Self {
            src_contents:
                "def z8990eb86ebbfda6ffb26010466539320a33a165388b1d0d028dff489841952d9() = 0".into(), // complete garbage
            src_path: PathBuf::from("."),
            ctx: ctx.unwrap_or_else(|| SpendContext {
                spender_txkind: TxKind::Normal,
                spender_other_inputs: Default::default(),
                spender_index: Default::default(),
                spender_data: Default::default(),
                spender_output: Default::default(),
                parent_value: Default::default(),
                parent_denom: Denom::Mel,
                parent_additional_data: Default::default(),
                ed25519_signers: Default::default(),
            }),
        }
    }

    /// Loads a file containing a Melodeon program, returning the  
    pub fn load_file(&mut self, path: &Path) -> Result<(melvm::Value, Type), LoadFileError> {
        let melo_str = std::fs::read_to_string(path).map_err(LoadFileError::IoError)?;

        self.load_str(path, &melo_str)
    }

    /// Load a string, read from a given path.
    pub fn load_str(
        &mut self,
        path: &Path,
        melo_str: &str,
    ) -> Result<(melvm::Value, Type), LoadFileError> {
        let (s, t) = melodeon::compile(melo_str, path).map_err(LoadFileError::MeloError)?;
        let parsed = mil::parser::parse_no_optimize(&s).expect("BUG: mil compilation failed");
        let melvm_ops = parsed.compile_onto(BinCode::default()).0;
        let env =
            EnvFile::from_spend_context(Covenant::from_ops(&melvm_ops).unwrap(), self.ctx.clone());
        let mut executor = Executor::new_from_env(melvm_ops, env.spender_tx, Some(env.environment));
        if executor.run_discerning_to_end_preserve_stack().is_none() {
            return Err(LoadFileError::VmError(executor));
        }
        let val = executor.stack.pop().unwrap();
        self.src_contents = melo_str.to_owned();
        self.src_path = path.to_owned();
        Ok((val, t))
    }

    /// Runs a REPL line, returning the execution result.
    pub fn run_repl_line(&mut self, line: &str) -> Result<(melvm::Value, Type), ReplError> {
        let line = line.trim();
        let s = if self.src_contents.contains("---") {
            self.src_contents.split("---").next().unwrap()
        } else if self.src_contents.contains("def")
            || self.src_contents.contains("provide")
            || self.src_contents.contains("require")
        {
            &self.src_contents
        } else {
            ""
        };
        let s = format!("{}\n---\n{}\n\n", s, line);
        // eprintln!("{}", s);
        let (s, t) = melodeon::compile(&s, &self.src_path).map_err(ReplError::MeloError)?;
        let parsed = mil::parser::parse_no_optimize(&s).expect("BUG: mil compilation failed");
        let melvm_ops = parsed.compile_onto(BinCode::default()).0;
        let env =
            EnvFile::from_spend_context(Covenant::from_ops(&melvm_ops).unwrap(), self.ctx.clone());
        let mut executor = Executor::new_from_env(melvm_ops, env.spender_tx, Some(env.environment));
        if executor.run_discerning_to_end_preserve_stack().is_none() {
            return Err(ReplError::VmError(executor));
        }
        let val = executor.stack.pop().unwrap();
        Ok((val, t))
    }
}
