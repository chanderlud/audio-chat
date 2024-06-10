// This is a modified version of the code found at
// https://github.com/fzyzcjy/flutter_rust_bridge/issues/486

use std::fs::File;
use std::sync::{Mutex, Once};

use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::Config;
use flutter_rust_bridge::frb;
use gag::Redirect;
use lazy_static::lazy_static;
use log::{info, warn, LevelFilter};
use parking_lot::RwLock;

use crate::frb_generated::StreamSink;

static INIT_LOGGER_ONCE: Once = Once::new();

lazy_static! {
    static ref SEND_TO_DART_LOGGER_STREAM_SINK: RwLock<Option<StreamSink<String>>> =
        RwLock::new(None);
    static ref GAG: Mutex<Option<Redirect<File>>> = Mutex::new(None);
}

pub fn init_logger() {
    // https://stackoverflow.com/questions/30177845/how-to-initialize-the-logger-for-integration-tests
    INIT_LOGGER_ONCE.call_once(|| {
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        {
            // Redirect stderr to a file for the entire execution of the program
            let gag = Redirect::stderr(File::create("stderr.log").unwrap()).unwrap();
            GAG.lock().unwrap().replace(gag);
        }

        // let level = if cfg!(debug_assertions) {
        //     LevelFilter::Debug
        // } else {
        //     LevelFilter::Warn
        // };

        let level = LevelFilter::Debug;

        assert!(
            level <= log::STATIC_MAX_LEVEL,
            "Should respect log::STATIC_MAX_LEVEL={:?}, which is done in compile time. level{:?}",
            log::STATIC_MAX_LEVEL,
            level
        );

        let mut config = Config::new().chan_len(Some(100)).level(level);

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "darwin"))]
        {
            config = config.file("audio_chat.log");
        }

        #[cfg(target_os = "android")]
        {
            config = config.custom(SendToDartLogger {});
        }

        fast_log::init(config).unwrap();

        info!("init_logger (inside 'once') finished");

        warn!(
            "init_logger finished, chosen level={:?} (deliberately output by warn level)",
            level
        );
    });
}

pub struct SendToDartLogger {}

impl SendToDartLogger {
    pub fn set_stream_sink(stream_sink: StreamSink<String>) {
        let mut guard = SEND_TO_DART_LOGGER_STREAM_SINK.write();
        let overriding = guard.is_some();

        *guard = Some(stream_sink);

        drop(guard);

        if overriding {
            warn!(
                "SendToDartLogger::set_stream_sink but already exist a sink, thus overriding. \
                (This may or may not be a problem. It will happen normally if hot-reload Flutter app.)"
            );
        }
    }

    fn record_to_formatted(record: &FastLogRecord) -> String {
        record.formated.replace('\n', "")
    }
}

impl LogAppender for SendToDartLogger {
    fn do_logs(&self, records: &[FastLogRecord]) {
        for record in records {
            let entry = Self::record_to_formatted(record);
            if let Some(sink) = &*SEND_TO_DART_LOGGER_STREAM_SINK.read() {
                _ = sink.add(entry);
            }
        }
    }
}

#[frb(sync)]
pub fn create_log_stream(s: StreamSink<String>) {
    SendToDartLogger::set_stream_sink(s);
}

#[frb(sync)]
pub fn rust_set_up() {
    init_logger();
}
