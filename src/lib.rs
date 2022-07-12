mod envfile;
mod runner;
pub use envfile::*;
pub use runner::*;
use themelio_stf::melvm::Value;

/// Converts a melvm value to a Melodeon-esque string representation.
pub fn mvm_pretty(val: &Value) -> String {
    match val {
        Value::Int(i) => i.to_string(),
        Value::Bytes(v) => {
            let raw = (0..v.len()).map(|i| *v.get(i).unwrap()).collect::<Vec<_>>();
            if let Some(string) = String::from_utf8(raw.clone()).ok().and_then(|s| {
                if s.chars().all(|c| !c.is_control()) {
                    Some(s)
                } else {
                    None
                }
            }) {
                let quoted = snailquote::escape(&string);
                if quoted.starts_with('\'') {
                    quoted.replace('\'', "\"")
                } else if quoted.starts_with('\"') {
                    quoted.into_owned()
                } else {
                    format!("\"{}\"", quoted)
                }
            } else {
                let raw_repr = hex::encode(raw);
                format!("x\"{}\"", raw_repr)
            }
        }
        Value::Vector(vv) => {
            let vv: Vec<_> = (0..vv.len())
                .map(|i| mvm_pretty(vv.get(i).unwrap()))
                .collect();
            format!("[{}]", vv.join(", "))
        }
    }
}
