use log::LevelFilter;

pub struct IVLogger;
impl log::Log for IVLogger {
    fn flush(&self) {}
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        eprintln!(
            "{}:{}: {}",
            record.target(),
            record.level().as_str(),
            record.args()
        )
    }
}
static LOGGER: IVLogger = IVLogger;
pub fn init_logger(verbose: bool) {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };
    log::set_max_level(level);
    log::set_logger(&LOGGER).expect("Failed to instantiate Logger");
}
