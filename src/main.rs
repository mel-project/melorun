use std::path::PathBuf;

use colored::Colorize;

use melodeon::typesys::Type;
use melorun::LoadFileError;
#[cfg(feature = "rustyline")]
use rustyline::Editor;
use structopt::StructOpt;
use themelio_stf::melvm::Value;

use melorun::Runner;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    /// Starts the interactive REPL.
    interactive: bool,

    #[structopt(short, long)]
    /// Compiles the program dumps to stdout the hash and hex-encoded MelVM bytecode.
    compile: bool,

    #[structopt(short, long)]
    /// An optional spend context YAML file.
    spend_ctx: Option<PathBuf>,

    /// The Melodeon program to run.
    input: Option<PathBuf>,
}

#[cfg(not(feature = "rustyline"))]
fn main() {
    todo!()
}

#[cfg(feature = "rustyline")]
fn main() -> anyhow::Result<()> {
    use melorun::SpendContext;

    env_logger::init();
    // std::env::set_var("CLICOLOR_FORCE", "1");
    let mut rl = Editor::<()>::new();

    let args = Args::from_args();
    // try to read the environment file
    let env_file: Option<SpendContext> = if let Some(ef) = args.spend_ctx.as_ref() {
        serde_yaml::from_str(&std::fs::read_to_string(ef)?)?
    } else {
        None
    };

    let mut runner = if let Some(env_file) = env_file {
        Runner::new(Some(env_file))
    } else {
        Runner::new(None)
    };

    // Treat input directory as a project
    //env_logger::init();
    if let Some(input) = args.input.as_ref() {
        match runner.load_file(input) {
            Ok((val, cov, t)) => {
                if args.compile {
                    println!("{}", cov.hash());
                    println!("{}", hex::encode(cov.0));
                }
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
    eprint!("{}", melorun::mvm_pretty(val));
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
