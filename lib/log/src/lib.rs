use fern::colors::{Color, ColoredLevelConfig};
use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, DateFrom, FileLimit},
    ContentLimit, TimeFrequency,
};
use zipall_core::{ZipAllError, ZipAllResult};

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

pub fn setup_logger(module_name: &'static str) -> ZipAllResult<()> {
    let module = module_name.to_string();

    let log_path = zipall_paths::log_file(module_name)?;
    let log_file = file_rotate::FileRotate::new(
        &log_path,
        AppendTimestamp::with_format("%Y%m%d", FileLimit::MaxFiles(30), DateFrom::DateYesterday),
        ContentLimit::Time(TimeFrequency::Daily),
        Compression::None,
        None,
    );

    let colors = ColoredLevelConfig::new().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] [{} {} {}] {}",
                module,
                chrono::Utc::now().to_rfc3339(),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level_for(module_name, log::LevelFilter::Trace)
        .level_for("zipall_log", log::LevelFilter::Info)
        .level_for("zipall_core", log::LevelFilter::Info)
        .level(log::LevelFilter::Warn)
        // .chain(fern::Output::stderr(LINE_ENDING))
        .chain(fern::Output::writer(Box::new(log_file), LINE_ENDING))
        .apply()
        .map_err(|e| ZipAllError::Logger(e.to_string()))?;

    log::info!("Logger configured for module: {}", module_name);
    log::info!("Logging to log file at: {:?}", log_path);

    Ok(())
}
