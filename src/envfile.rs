use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use themelio_stf::melvm::{Covenant, CovenantEnv};
use themelio_structs::{
    Address, CoinData, CoinDataHeight, CoinID, CoinValue, Denom, Header, Transaction, TxHash,
    TxKind,
};
use tmelcrypt::{Ed25519SK, HashVal};

/// YAML/TOML/JSON-encoded environment file
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnvFile {
    /// Input transaction
    pub spender_tx: Transaction,
    /// Execution environment
    pub environment: CovenantEnv,
}

impl EnvFile {
    /// Conversion from a spend context
    pub fn from_spend_context(covenant: Covenant, sc: SpendContext) -> Self {
        let mut input_map = sc.spender_other_inputs.clone();
        input_map.insert(
            sc.spender_index,
            CoinID {
                txhash: TxHash(Default::default()),
                index: 0,
            },
        );
        let mut spender_tx = Transaction {
            kind: sc.spender_txkind,
            inputs: map_to_vec(
                input_map,
                CoinID {
                    txhash: TxHash(Default::default()),
                    index: 0,
                },
            ),
            outputs: map_to_vec(
                sc.spender_output.clone(),
                CoinData {
                    covhash: Address(Default::default()),
                    value: CoinValue(0),
                    denom: Denom::Mel,
                    additional_data: vec![],
                },
            ),
            fee: CoinValue(0),
            covenants: vec![covenant.0.clone()],
            data: sc.spender_data.clone(),
            sigs: vec![],
        };
        let txhash = spender_tx.hash_nosigs();
        spender_tx.sigs = map_to_vec(
            sc.ed25519_signers
                .iter()
                .filter_map(|(k, v)| {
                    Some((
                        *k,
                        Ed25519SK::from_bytes(&hex::decode(&v).ok()?)?.sign(&txhash.0),
                    ))
                })
                .collect(),
            vec![],
        );
        Self {
            spender_tx,
            environment: CovenantEnv {
                parent_coinid: CoinID {
                    txhash: TxHash(Default::default()),
                    index: 0,
                },
                parent_cdh: CoinDataHeight {
                    height: 1000.into(),
                    coin_data: CoinData {
                        covhash: covenant.hash(),
                        value: sc.parent_value,
                        denom: sc.parent_denom,
                        additional_data: sc.parent_additional_data,
                    },
                },
                spender_index: sc.spender_index,
                last_header: Header {
                    network: themelio_structs::NetID::Custom05,
                    previous: HashVal::default(),
                    height: 2000.into(),
                    history_hash: HashVal::default(),
                    coins_hash: HashVal::default(),
                    transactions_hash: HashVal::default(),
                    fee_pool: 10000.into(),
                    fee_multiplier: 10000,
                    dosc_speed: 100000,
                    pools_hash: HashVal::default(),
                    stakes_hash: HashVal::default(),
                },
            },
        }
    }
}

/// A YAML-serializable *partial* specification of a spending context that can translate into a full EnvFile.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpendContext {
    #[serde(default = "default_txkind")]
    /// Optionally indicate the spending transaction kind.
    pub spender_txkind: TxKind,
    #[serde(default)]
    /// Optionally fill in the other inputs of the transaction.
    pub spender_other_inputs: BTreeMap<u8, CoinID>,
    /// Which index is the coin?
    pub spender_index: u8,
    /// Data field of the spending transaction.
    #[serde(with = "stdcode::hex", default)]
    pub spender_data: Vec<u8>,
    #[serde(default)]
    /// Optionally fill in the outputs of the transaction.
    pub spender_output: BTreeMap<u8, CoinData>,
    /// Value of the parent coin.
    #[serde(with = "serde_with::rust::display_fromstr", default)]
    pub parent_value: CoinValue,
    /// Denom of the parent coin.
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub parent_denom: Denom,
    /// Additional data of the parent coin.
    #[serde(with = "stdcode::hex", default)]
    pub parent_additional_data: Vec<u8>,
    /// Private keys (in hex, proper deserialization TODO) that the transaction is signed *by*.
    #[serde(default)]
    pub ed25519_signers: BTreeMap<u8, String>,
}

fn default_txkind() -> TxKind {
    TxKind::Normal
}

fn map_to_vec<V: Clone>(map: BTreeMap<u8, V>, default: V) -> Vec<V> {
    let mut list = vec![];
    for (k, v) in map {
        let k = k as usize;
        list.resize(k + 1, default.clone());
        list[k] = v;
    }
    list
}
