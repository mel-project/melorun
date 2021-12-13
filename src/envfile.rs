use serde::Deserialize;
use themelio_stf::{CoinDataHeight, CoinID, Header, Transaction};

/// YAML/TOML/JSON-encoded environment file
#[derive(Deserialize, Clone, Debug)]
pub struct EnvFile {
    /// Input transaction
    pub spender_tx: Transaction,
    /// Execution environment
    pub environment: CovenantEnvRepr,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CovenantEnvRepr {
    pub parent_coinid: CoinID,
    pub parent_cdh: CoinDataHeight,
    pub spender_index: u8,
    pub last_header: Header,
}
