use serde::{Deserialize, Serialize};
use themelio_stf::melvm::CovenantEnv;
use themelio_structs::Transaction;

/// YAML/TOML/JSON-encoded environment file
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvFile {
    /// Input transaction
    pub spender_tx: Transaction,
    /// Execution environment
    pub environment: CovenantEnv,
}
