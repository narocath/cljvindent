mod indentation_engine;
use std::sync::{Mutex, OnceLock};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::Layer, registry::Registry, filter::LevelFilter, prelude::*};

static LOG_GUARD: OnceLock<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    OnceLock::new();

pub use indentation_engine::{
    helpers,
    engine,
    model::AlignKind,
    indent_current_form_once,
    indent_bottom_up,
    indent_whole_file_parallel,
    indent_clojure_file,
    indent_clojure_file_no_return,
    indent_clojure_string,
    indent_clojure_string_collection,
};

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

#[cfg(feature = "emacs-module")]
mod emacs_module;
