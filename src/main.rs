use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use colored::Colorize;
use regex::Regex;
use rustyline::Editor;
use structopt::StructOpt;
use themelio_stf::melvm::{Covenant, Executor, Value};

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    interactive: bool,

    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    std::env::set_var("CLICOLOR_FORCE", "1");
    let mut rl = Editor::<()>::new();

    let args = Args::from_args();
    let (success, exec) = run_file(&args.input)?;
    eprintln!(
        "{}: {}",
        "result".bold(),
        if success {
            "success".green()
        } else {
            "failed".red()
        }
    );
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
                format!("let {} = ({}) in ({})", k, v, a)
            });
            std::fs::write(
                Path::new(&tempfile_name),
                format!("{}\n{}", definitions, expr).as_bytes(),
            )?;
            let (success, exec) = run_file(Path::new(&tempfile_name))?;
            if success {
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
            if let Ok(string) = String::from_utf8(raw.clone()) {
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
fn run_file(input: &Path) -> anyhow::Result<(bool, Executor)> {
    let mut mil_tempfile = tempfile::tempdir()?.into_path();
    mil_tempfile.push("temp.mil");
    // run meloc
    let meloc_result = Command::new("meloc")
        .arg(input)
        .arg("--output")
        .arg(&mil_tempfile)
        .output()?;
    if !meloc_result.status.success() {
        eprint!("{}", String::from_utf8_lossy(&meloc_result.stderr));
        anyhow::bail!("meloc failed")
    }
    // run mil
    let melvm_hex = hex::decode(
        String::from_utf8_lossy(&Command::new("mil").arg(mil_tempfile).output()?.stdout).trim(),
    )?;
    let melvm_ops = Covenant(melvm_hex).to_ops()?;
    let mut executor = Executor::new(melvm_ops, HashMap::new());
    let success = executor.run_to_end_preserve_stack();
    Ok((success, executor))
}
