use serde::{Deserialize, Serialize};
use themelio_stf::{
    melvm::Covenant, CoinData, CoinDataHeight, CoinID, CoinValue, Header, HexBytes, TxKind,
};

/// YAML/TOML/JSON-encoded environment file
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvFile {
    /// Input transaction
    pub spender_tx: TransactionRepr,
    /// Execution environment
    pub environment: CovenantEnvRepr,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionRepr {
    #[serde(default)]
    pub kind: Option<TxKind>,
    #[serde(default)]
    pub inputs: Vec<CoinID>,
    #[serde(default)]
    pub outputs: Vec<CoinData>,
    #[serde(default)]
    pub fee: CoinValue,
    #[serde(default)]
    pub scripts: Vec<Covenant>,
    #[serde(with = "stdcode::hex")]
    #[serde(default)]
    pub data: Vec<u8>,
    #[serde(default)]
    pub sigs: Vec<HexBytes>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CovenantEnvRepr {
    #[serde(default)]
    pub parent_coinid: Option<CoinID>,
    #[serde(default)]
    pub parent_cdh: Option<CoinDataHeight>,
    #[serde(default)]
    pub spender_index: u8,
    #[serde(default)]
    pub last_header: Option<Header>,
}
