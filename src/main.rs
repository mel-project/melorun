mod envfile;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use colored::Colorize;
use mil::compiler::{BinCode, Compile};
use regex::Regex;
use rustyline::Editor;
use structopt::StructOpt;
use themelio_stf::melvm::{Covenant, CovenantEnv, Executor, Value};
use themelio_structs::{
    Address, CoinData, CoinDataHeight, CoinID, Denom, Header, NetID, Transaction, TxKind,
};

use crate::envfile::EnvFile;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    interactive: bool,

    #[structopt(short, long)]
    environment: Option<PathBuf>,

    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    std::env::set_var("CLICOLOR_FORCE", "1");
    let mut rl = Editor::<()>::new();

    let args = Args::from_args();
    // try to read the environment file
    let env_file: Option<EnvFile> = if let Some(ef) = args.environment.as_ref() {
        serde_json::from_str(&std::fs::read_to_string(ef)?)?
    } else {
        None
    };
    // Treat input directory as a project
    //env_logger::init();
    let (success, exec, covenant) = if args.input.is_dir() {
        let main_file = Path::new(&args.input).join("main.melo");
        run_file(&main_file, env_file.clone())?
    }
    // Input is a single file
    else {
        run_file(&args.input, env_file.clone())?
    };
    eprintln!(
        "{}: {}",
        "result".bold(),
        if let Some(res) = success {
            if res {
                "Covenant evaluates true".green()
            } else {
                "Covenant evaluates false".red()
            }
        } else {
            "Early termination from program failure".red()
        }
    );
    if success.is_none() {
        eprintln!("-- PROGRAM --");
        for (i, elem) in covenant.to_ops().unwrap().into_iter().enumerate() {
            if i == exec.pc() {
                eprintln!("{} <- \t {}", i, elem)
            } else {
                eprintln!("{} \t {}", i, elem)
            }
        }
        eprintln!("-- STACK --");
        for (i, elem) in exec.stack.iter().enumerate() {
            eprintln!("{}: {}", i, mvm_pretty(elem));
        }
        eprintln!("-- HEAP --");
        let mut hh = exec.heap.clone().into_iter().collect::<Vec<_>>();
        hh.sort_unstable_by_key(|d| d.0);
        for (k, v) in hh.iter() {
            eprintln!("{}: {}", k, mvm_pretty(v));
        }
    }
    //eprintln!("{:?}", exec.stack);
    eprintln!(
        "{}: {}",
        "value".bold(),
        exec.stack
            .last()
            .map(|v| mvm_pretty(&v))
            .unwrap_or_else(|| "(none)".to_string())
    );
    if !args.interactive {
        Ok(())
    } else {
        // enter the repl loop
        // first we write the tempfile
        let source_code = std::fs::read_to_string(&args.input)?;
        let definitions = source_code
            .split("---")
            .find(|s| s.contains("def "))
            .map(|s| format!("{} ---", s))
            .unwrap_or_default();
        // then loop
        let mut repl_definitions: HashMap<String, String> = HashMap::new();
        let var_regex = Regex::new("[a-z][A-Z0-9_]*")?;
        let run_expr = |expr: &str, repl_definitions: &HashMap<String, String>| {
            let tempfile_name = format!("{}.tmp", args.input.to_string_lossy());
            let tfn = tempfile_name.clone();
            scopeguard::defer!({
                let _ = std::fs::remove_file(Path::new(&tfn));
            });
            let expr = repl_definitions.iter().fold(expr.to_string(), |a, (k, v)| {
                format!("(let {} = ({}) in\n{}\n)", k, v, a)
            });
            std::fs::write(
                Path::new(&tempfile_name),
                format!("{}\n{}", definitions, expr).as_bytes(),
            )?;
            let (_, exec, _) = run_file(Path::new(&tempfile_name), env_file.clone())?;
            if exec.at_end() {
                Ok(mvm_pretty(exec.stack.last().unwrap()))
            } else {
                Err(anyhow::anyhow!("execution failed"))
            }
        };
        loop {
            match rl.readline(&"melorun> ".bold().bright_blue().to_string()) {
                Ok(line) => {
                    rl.add_history_entry(line.clone());
                    if line
                        .split_ascii_whitespace()
                        .enumerate()
                        .find(|a| a.1 == "=")
                        .map(|a| a.0 == 1)
                        .unwrap_or(false)
                        || line.find('=') == Some(1)
                    {
                        let (varname, body) = line.split_once('=').unwrap();
                        if !var_regex.is_match(varname) {
                            eprintln!(
                                "{}: not a valid REPL variable name",
                                "error".bold().bright_red()
                            );
                            continue;
                        }
                        let varname = varname.trim().to_string();
                        match run_expr(body, &repl_definitions) {
                            Ok(val) => {
                                repl_definitions.insert(varname, val);
                            }
                            Err(err) => eprintln!("{}", err.to_string().bright_red()),
                        }
                    } else {
                        match run_expr(&line, &repl_definitions) {
                            Ok(val) => {
                                eprintln!("{}", val);
                            }
                            Err(err) => eprintln!("{}", err.to_string().bright_red()),
                        }
                    }
                    eprintln!();
                }
                Err(_) => anyhow::bail!("interrupted"),
            }
        }
    }
}

// Converts a melvm value to a Melodeon-esque string representation.
fn mvm_pretty(val: &Value) -> String {
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
                    quoted.replace("\'", "\"")
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

// Runs a file with little fanfare. Repeatedly called
fn run_file(
    input: &Path,
    env: Option<EnvFile>,
) -> anyhow::Result<(Option<bool>, Executor, Covenant)> {
    // Compile melodeon to mil
    let melo_str = std::fs::read_to_string(input)?;
    let mil_code = melodeon::compile(&melo_str, input)
        .map_err(|ctx| anyhow::anyhow!(format!("Melodeon compilation failed\n{}", ctx)))?;

    // Compile mil to op codes
    let parsed = mil::parser::parse_no_optimize(&mil_code).map_err(|e| {
        anyhow::anyhow!(format!(
            "Internal error, failed to parse mil output\n{:?}",
            e
        ))
    })?;
    let melvm_ops = parsed.compile_onto(BinCode::default()).0;

    let mut executor = if let Some(env) = env {
        Executor::new_from_env(
            melvm_ops.clone(),
            Transaction {
                kind: env.spender_tx.kind.unwrap_or(TxKind::Normal),
                inputs: env.spender_tx.inputs,
                outputs: env.spender_tx.outputs,
                fee: env.spender_tx.fee,
                covenants: env.spender_tx.scripts,
                data: env.spender_tx.data,
                sigs: env.spender_tx.sigs,
            },
            Some(CovenantEnv {
                parent_coinid: &env
                    .environment
                    .parent_coinid
                    .unwrap_or_else(CoinID::zero_zero),
                parent_cdh: &env
                    .environment
                    .parent_cdh
                    .unwrap_or_else(|| CoinDataHeight {
                        coin_data: CoinData {
                            covhash: Address::coin_destroy(),
                            value: 0.into(),
                            denom: Denom::Mel,
                            additional_data: vec![],
                        },
                        height: 0.into(),
                    }),
                spender_index: env.environment.spender_index,
                last_header: &env.environment.last_header.unwrap_or(Header {
                    network: NetID::Custom08,
                    previous: Default::default(),
                    height: Default::default(),
                    history_hash: Default::default(),
                    coins_hash: Default::default(),
                    transactions_hash: Default::default(),
                    fee_pool: Default::default(),
                    fee_multiplier: Default::default(),
                    dosc_speed: Default::default(),
                    pools_hash: Default::default(),
                    stakes_hash: Default::default(),
                }),
            }),
        )
    } else {
        Executor::new(melvm_ops.clone(), HashMap::new())
    };

    let success = executor.run_discerning_to_end_preserve_stack();
    Ok((success, executor, Covenant::from_ops(&melvm_ops).unwrap()))
}
