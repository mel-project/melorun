mod envfile;
pub mod runner;

use std::path::{Path, PathBuf};

use colored::Colorize;

use melodeon::typesys::Type;
use runner::LoadFileError;
use rustyline::Editor;
use structopt::StructOpt;
use themelio_stf::melvm::Value;

use crate::{envfile::EnvFile, runner::Runner};

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    interactive: bool,

    #[structopt(short, long)]
    environment: Option<PathBuf>,

    input: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    // std::env::set_var("CLICOLOR_FORCE", "1");
    let mut rl = Editor::<()>::new();

    let args = Args::from_args();
    // try to read the environment file
    let env_file: Option<EnvFile> = if let Some(ef) = args.environment.as_ref() {
        serde_json::from_str(&std::fs::read_to_string(ef)?)?
    } else {
        None
    };

    let mut runner = if let Some(env_file) = env_file {
        Runner::new(Some(env_file.environment), Some(env_file.spender_tx))
    } else {
        Runner::new(None, None)
    };

    // Treat input directory as a project
    //env_logger::init();
    if let Some(input) = args.input.as_ref() {
        match runner.load_file(input) {
            Ok((val, t)) => {
                print_val_and_type(&val, &t, true);
            }
            Err(LoadFileError::MeloError(ctx)) => {
                eprintln!("{}", ctx);
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        }
    };
    if !args.interactive {
        Ok(())
    } else {
        loop {
            match rl.readline(&"melorun> ".bold().bright_blue().to_string()) {
                Ok(line) => {
                    rl.add_history_entry(line.clone());
                    match runner.run_repl_line(&line) {
                        Ok((val, t)) => {
                            print_val_and_type(&val, &t, false);
                        }
                        Err(err) => eprintln!("{}", err),
                    }

                    eprintln!();
                }
                Err(_) => anyhow::bail!("interrupted"),
            }
        }
    }
}

fn print_val_and_type(val: &Value, t: &Type, truthiness: bool) {
    if t.subtype_of(&Type::all_nat()) && format!("- : {:?}", t).contains('}') {
        eprintln!(
            "{} {}",
            "- : Nat".bright_purple().bold(),
            format!("[more specifically: {:?}]", t)
                .as_str()
                .bright_purple()
        );
    } else {
        eprintln!("{}", format!("- : {:?}", t).as_str().bright_purple().bold());
    }
    eprint!("{}", mvm_pretty(val));
    if truthiness {
        if val.clone().into_bool() {
            eprintln!("{}", " (truthy)".bright_green());
        } else {
            eprintln!("{}", " (falsy)".bright_red());
        }
    } else {
        eprintln!()
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
