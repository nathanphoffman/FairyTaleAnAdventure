//! Logging input events for FTA_RECORD.

use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

/// Tick counter set by the display loop; stamps the lines of the FTA_RECORD log.
pub static TICK: AtomicU32 = AtomicU32::new(0);

const RECORD_VAR: &str = "FTA_RECORD";

/// FTA_RECORD=file logs every key and mouse event as "tick key NAME 1|0", "tick move X Y" or
/// "tick button left|right 1|0". Feed the file back with FTA_REPLAY=file to repeat the session.
pub(super) fn record(line: std::fmt::Arguments) {
    static LOG: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    let log = LOG.get_or_init(|| {
        let path = std::env::var_os(RECORD_VAR)?;

        match std::fs::File::create(&path) {
            Ok(f) => Some(Mutex::new(f)),
            Err(e) => {
                eprintln!("Could not create the record file: {e}");

                None
            }
        }
    });

    if let Some(f) = log {
        let tick = TICK.load(Ordering::Relaxed);

        if let Ok(mut f) = f.lock()
            && let Err(e) = writeln!(f, "{tick} {line}")
        {
            eprintln!("Could not write to the record file: {e}");
        }
    }
}
