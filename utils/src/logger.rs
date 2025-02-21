use argh::FromArgs;
use simple_logger::SimpleLogger;

#[derive(FromArgs)]
/// Logger arguments
pub struct LogArgs {
    /// the log level
    #[argh(option, short = 'l', default = "log::LevelFilter::Info")]
    log: log::LevelFilter,
}

pub fn setup_logger(level: log::LevelFilter) {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(level)
        .without_timestamps()
        .init()
        .unwrap();
}

pub fn setup_logger_from_env() {
    let args: LogArgs = argh::from_env();
    setup_logger(args.log);
}
