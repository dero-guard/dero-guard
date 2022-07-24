use fern::colors::{ColoredLevelConfig, Color};
use log::Level;

pub use log;
pub use clap;

pub mod command;
pub mod dero;
pub mod json_rpc;
pub mod service;
pub mod wg;

pub const SCID: &str = "b0b6eff653ef41ea5a73e7f0ee24833a930969a93ea91664e06a828d1be60997";

// configure fern and print prompt message after each new output
pub fn setup_logger(debug: bool, disable_file_logging: bool) -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Green)
        .info(Color::Cyan)
        .warn(Color::Yellow)
        .error(Color::Red);
    let base = fern::Dispatch::new();
    let stdout_log = fern::Dispatch::new()
        .format(move |out, message, record| {
            let target = record.target();
            let mut target_with_pad = " ".repeat((30i16 - target.len() as i16).max(0) as usize) + target;
            if record.level() != Level::Error && record.level() != Level::Debug {
                target_with_pad = " ".to_owned() + &target_with_pad;
            }

            out.finish(format_args!(
                "\r\x1B[90m{} {}\x1B[0m \x1B[{}m{}\x1B[0m \x1B[90m>\x1B[0m {}",
                chrono::Local::now().format("[%Y-%m-%d] (%H:%M:%S%.3f)"),
                colors.color(record.level()),
                Color::BrightBlue.to_fg_str(),
                target_with_pad,
                message
            ))
        }).chain(std::io::stdout());

    let mut base = base.chain(stdout_log);
    if !disable_file_logging {
        let file_log = fern::Dispatch::new()
        .format(move |out, message, record| {
            let pad = " ".repeat((30i16 - record.target().len() as i16).max(0) as usize);
            let level_pad = if record.level() == Level::Error || record.level() == Level::Debug { "" } else { " " };
            out.finish(format_args!(
                "{} [{}{}] [{}]{} | {}",
                chrono::Local::now().format("[%Y-%m-%d] (%H:%M:%S%.3f)"),
                record.level(),
                level_pad,
                record.target(),
                pad,
                message
            ))
        }).chain(fern::log_file("dero-guard.log")?);
        base = base.chain(file_log);
    }

    base = if debug {
        base.level(log::LevelFilter::Debug)
    } else {
        base.level(log::LevelFilter::Info)
    };
    base.apply()?;
    Ok(())
}