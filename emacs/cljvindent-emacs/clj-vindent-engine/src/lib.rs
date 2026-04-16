mod indentation_engine;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::Layer, registry::Registry, filter::LevelFilter, prelude::*};
use clap::ValueEnum;

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

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogOutputType {
    Json,
    Compact
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogMode {
    Off,
    Stdout,
    StdoutFile,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug
}

pub fn init_logging(enabled: bool, lvl: LevelFilter) {
    let level = if enabled { lvl } else { LevelFilter::OFF };

    fmt()
        .with_max_level(level)
        .pretty()
        .init();
}

pub fn init_logging_with_file(
    enabled: bool,
    level: LevelFilter,
    file_out_type: LogOutputType,
) -> tracing_appender::non_blocking::WorkerGuard {
    let level = if enabled { level } else { LevelFilter::OFF };

    let file_appender = rolling::daily("logs", "cljvindent.log");
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

    tracing_subscriber::registry()
        .with(file_layer)
        .with(level)
        .with(stdout_layer)
        .init();

    guard
}

#[cfg(feature = "emacs-module")]
mod emacs_module;
