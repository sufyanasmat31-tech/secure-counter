use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, info, warn};

const MAX_VALUE: i64 = 1_000_000;
const MIN_VALUE: i64 = -1_000_000;
const STATE_FILE: &str = "counter.state";

#[derive(Debug, Error)]
enum CounterError {
    #[error("counter would exceed maximum allowed value of {MAX_VALUE}")]
    Overflow,
    #[error("counter would go below minimum allowed value of {MIN_VALUE}")]
    Underflow,
    #[error("failed to read state file: {0}")]
    StateRead(#[from] std::io::Error),
    #[error("state file contains invalid data: '{0}'")]
    StateParse(String),
    #[error("step must be between 1 and 1000, got {0}")]
    InvalidStep(i64),
}

#[derive(Parser)]
#[command(
    name = "secure-counter",
    about = "Bounds-checked, persistent counter with structured logging and atomic state writes",
    version
)]
struct Cli {
    /// Path to the counter state file
    #[arg(long, default_value = STATE_FILE)]
    state: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Increment the counter by STEP (default: 1)
    Increment {
        #[arg(default_value_t = 1)]
        step: i64,
    },
    /// Decrement the counter by STEP (default: 1)
    Decrement {
        #[arg(default_value_t = 1)]
        step: i64,
    },
    /// Reset the counter to zero
    Reset,
    /// Print the current counter value
    Get,
}

fn load(path: &PathBuf) -> Result<i64, CounterError> {
    if !path.exists() {
        info!(path = %path.display(), "state file not found — initialising at 0");
        return Ok(0);
    }
    let raw = fs::read_to_string(path)?;
    raw.trim()
        .parse::<i64>()
        .map_err(|_| CounterError::StateParse(raw.trim().to_string()))
}

fn save(path: &PathBuf, value: i64) -> Result<(), CounterError> {
    // Write to .tmp then rename — guarantees the state file is never half-written
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, value.to_string())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn validate_step(step: i64) -> Result<i64, CounterError> {
    if !(1..=1000).contains(&step) {
        return Err(CounterError::InvalidStep(step));
    }
    Ok(step)
}

fn run() -> Result<(), CounterError> {
    let cli = Cli::parse();
    let current = load(&cli.state)?;
    info!(value = current, "loaded counter state");

    let next = match cli.command {
        Command::Increment { step } => {
            let step = validate_step(step)?;
            let next = current
                .checked_add(step)
                .filter(|&v| v <= MAX_VALUE)
                .ok_or(CounterError::Overflow)?;
            info!(from = current, step, to = next, "incremented");
            next
        }
        Command::Decrement { step } => {
            let step = validate_step(step)?;
            let next = current
                .checked_sub(step)
                .filter(|&v| v >= MIN_VALUE)
                .ok_or(CounterError::Underflow)?;
            info!(from = current, step, to = next, "decremented");
            next
        }
        Command::Reset => {
            warn!(current_value = current, "resetting counter to 0");
            0
        }
        Command::Get => {
            println!("{current}");
            return Ok(());
        }
    };

    save(&cli.state, next)?;
    println!("{next}");
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(false)
        .compact()
        .init();

    if let Err(e) = run() {
        error!(error = %e, "fatal");
        std::process::exit(1);
    }
}
