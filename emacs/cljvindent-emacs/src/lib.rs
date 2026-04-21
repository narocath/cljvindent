// SPDX-License-Identifier: Apache-2.0

use std::sync::{Mutex, OnceLock};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::Layer, registry::Registry, filter::LevelFilter, prelude::*};
use emacs::{defun, Env, Result, Vector, Value};
use cljvindent_core::{indent_clojure_string,
                      indent_clojure_string_collection,
                      indent_clojure_file_no_return};

emacs::plugin_is_GPL_compatible!();

static LOG_GUARD: OnceLock<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    OnceLock::new();

#[derive(Copy, Clone, Debug)]
pub enum LogOutputType {
    Json,
    Compact
}

#[derive(Copy, Clone, Debug)]
pub enum LogMode {
    Off,
    Stdout,
    StdoutFile,
}

#[derive(Copy, Clone, Debug)]
pub enum LogLevel {
    Info,
    Debug
}

pub fn init_logging(enabled: bool, lvl: LevelFilter) {
    let level = if enabled { lvl } else { LevelFilter::OFF };

    let _ = fmt()
        .with_max_level(level)
        .pretty()
        .try_init();
}

pub fn init_logging_with_file(
    enabled: bool,
    level: LevelFilter,
    file_out_type: LogOutputType,
) {
    let slot = LOG_GUARD.get_or_init(|| Mutex::new(None));

    if slot.lock().unwrap().is_some(){
        return;
    }
    let level = if enabled { level } else { LevelFilter::OFF };

    let _ = std::fs::create_dir_all(".cljvindent_logs").ok();

    let file_appender = rolling::daily(".cljvindent_logs", "cljvindent.log");
    let (file_writer, guard) = non_blocking(file_appender);

    let stdout_layer = fmt::layer().pretty().with_writer(std::io::stdout);
    let file_layer:  Box<dyn Layer<Registry> + Send + Sync> = match file_out_type {
        LogOutputType::Compact => fmt::layer()
            .with_ansi(false)
            .compact()
            .with_writer(file_writer)
            .boxed(),
        LogOutputType::Json => fmt::layer()
            .with_ansi(false)
            .json()
            .with_writer(file_writer)
            .boxed()
    };
    if tracing_subscriber::registry()
        .with(file_layer)
        .with(level)
        .with(stdout_layer)
        .try_init()
        .is_ok()
    {
        *slot.lock().unwrap() = Some(guard);
    }
}

#[emacs::module(
    name = "cljvindent-native",
    defun_prefix = "cljvindent",
    separator = "--",
    mod_in_name = false
)]
pub fn init(_: &Env) -> Result<()>{
    Ok(())
}

#[defun]
pub fn indent_string(
    input: String,
    base_col: usize,
    enable_logs: Value<'_>,
    loglvl: String,
    file_out_type: String
) -> Result<String> {
    let logs = enable_logs.is_not_nil();
    let logs_file_output_type = match file_out_type.as_str() {
        "json" => LogOutputType::Json,
        "compact" => LogOutputType::Compact,
        _ => LogOutputType::Compact
    };
    let log_level = match loglvl.as_str() {
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        _ => LevelFilter::INFO
    };
    if logs {
        let _ = init_logging_with_file(logs, log_level, logs_file_output_type);
    }
    Ok(indent_clojure_string(&input, base_col as usize))
}


#[defun]
pub fn indent_string_collection<'e>(
    env: &'e Env,
    input: Vector<'e>,
    enable_logs: Value<'_>,
    loglvl: String,
    file_out_type: String
) -> Result<Vector<'e>> {
    let mut rust_input: Vec<(String, usize)> = Vec::with_capacity(input.len());
    let logs = enable_logs.is_not_nil();
    let logs_file_output_type = match file_out_type.as_str() {
        "json" => LogOutputType::Json,
        "compact" => LogOutputType::Compact,
        _ => LogOutputType::Compact
    };
    let log_level = match loglvl.as_str() {
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        _ => LevelFilter::INFO
    };
    if logs {
        let _ = init_logging_with_file(logs, log_level, logs_file_output_type);
    }
    for i in 0..input.len() {
        let pair = input.get::<Vector>(i)?;

        if pair.len() != 2 {
            return Err(emacs::Error::msg(
                "Each item must be a 2-element vector: [STRING BASE-COL]",
            ));
        }

        let text = pair.get::<String>(0)?;
        let base_col = pair.get::<usize>(1)?;

        rust_input.push((text, base_col));
    }

    let rust_output = indent_clojure_string_collection(&rust_input);

    let out = env.make_vector(rust_output.len(), "")?;
    for (i, s) in rust_output.into_iter().enumerate() {
        out.set(i, s)?;
    }

    Ok(out)
}

#[defun]
pub fn indent_clj_file(
    path: String,
    enable_logs: Value<'_>,
    loglvl: String,
    file_out_type: String) -> Result<bool>{
    let logs = enable_logs.is_not_nil();
    let logs_file_output_type = match file_out_type.as_str() {
        "json" => LogOutputType::Json,
        "compact" => LogOutputType::Compact,
        _ => LogOutputType::Compact
    };
    let log_level = match loglvl.as_str() {
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        _ => LevelFilter::INFO
    };
    if logs {
        let _ = init_logging_with_file(logs, log_level, logs_file_output_type);
    }
    let _ = indent_clojure_file_no_return(path);
    Ok(true)
}
