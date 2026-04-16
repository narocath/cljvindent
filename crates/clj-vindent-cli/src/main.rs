use cljvindent::{indent_clojure_file_no_return, indent_clojure_string,
                         init_logging, init_logging_with_file, LogOutputType, LogMode, LogLevel};
use std::time::Instant;
use std::path::PathBuf;
use clap::Parser;
use tracing::{info, debug, instrument};
use tracing_subscriber::filter::LevelFilter;


fn existing_file_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if !path.exists() {
        return Err(format!("file does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("not a regular file: {}", path.display()));
    }

    Ok(path)
}

fn non_empty_string(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        Err("string must not be empty".to_string())
    } else {
        Ok(s.to_string())
    }
}

#[derive(Parser, Debug)]
#[command(name = "cljvindent")]
#[command(version)]
#[command(about = "Format Clojure code")]
struct Cli {
    /// Set the type of logs to be saved to log file
    #[arg(
        short = 'o',
        long = "file-log-output-type",
        value_enum,
        default_value_t = LogOutputType::Compact
    )]
    file_log_output_type:  LogOutputType,
    /// Enable logs
    #[arg(
        short = 'g',
        long = "logs",
        value_enum,
        default_value_t = LogMode::Off
    )]
    logs: LogMode,
     /// Set log level
    #[arg(
        short = 'l',
        long = "level",
        value_enum,
        default_value_t = LogLevel::Info
    )]
    level: LogLevel,
    /// Format file at path and write result back
    #[arg(
        short = 'f',
        long = "file",
        conflicts_with = "string",
        value_parser = existing_file_path
    )]
    file: Option<PathBuf>,

    /// Format the provided string and print result
    #[arg(
        short = 's',
        long = "string",
        conflicts_with = "file",
        value_parser = non_empty_string
    )]
    string: Option<String>,

    /// Base starting column for string mode
    #[arg(
        short = 'c',
        long = "start-column",
        default_value_t = 0,
        value_parser = clap::value_parser!(u64).range(0..=10_000)
    )]
    base_col: u64,
}

#[instrument]
fn main() {
    let cli = Cli::parse();
    let type_of_log_file_output = cli.file_log_output_type;
    
    let log_level = match cli.level{
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Debug => LevelFilter::DEBUG
    };
    let _log_guard = match cli.logs {
        LogMode::Off => None,
        LogMode::Stdout => {
            init_logging(true, log_level);
            None
        },
        LogMode::StdoutFile => Some(init_logging_with_file(true, log_level, type_of_log_file_output))
    };
    match (cli.file, cli.string) {
        (Some(path), None) => {
            let start = Instant::now();
            indent_clojure_file_no_return(path.to_string_lossy().to_string())
                .expect("indent failed");
            let elapsed = start.elapsed();
            //println!("Done!!\nElapsed: {:.3?}", elapsed);
            info!("Done!! Elapsed: {:.3?}", elapsed);
        }
        (None, Some(s)) => {
            let start = Instant::now();
            let out = indent_clojure_string(&s, cli.base_col as usize);
            let elapsed = start.elapsed();
            //println!("{out}");
            debug!("{out}");
            //eprintln!("Elapsed: {:.3?}", elapsed);
            info!("Done!! Elapsed: {:.3?}", elapsed);
        }
        _ => {
            eprintln!("Use either --file/-f for a file path or --string/-s for a literal string to indent");
            std::process::exit(2);
        }
    }
}
