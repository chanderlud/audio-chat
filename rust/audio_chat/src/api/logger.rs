// This is a modified version of the code found at
// https://github.com/fzyzcjy/flutter_rust_bridge/issues/486

use std::sync::Once;

use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::Config;
use flutter_rust_bridge::frb;
use lazy_static::lazy_static;
use log::{info, warn, LevelFilter};
use parking_lot::RwLock;

use crate::frb_generated::StreamSink;

static INIT_LOGGER_ONCE: Once = Once::new();

lazy_static! {
    static ref SEND_TO_DART_LOGGER_STREAM_SINK: RwLock<Option<StreamSink<String>>> =
        RwLock::new(None);
}

pub fn init_logger() {
    // https://stackoverflow.com/questions/30177845/how-to-initialize-the-logger-for-integration-tests
    INIT_LOGGER_ONCE.call_once(|| {
        let level = if cfg!(debug_assertions) {
            LevelFilter::Info // Debug is spammed by sctp
        } else {
            LevelFilter::Warn
        };

        // let level = LevelFilter::Info;

        assert!(
            level <= log::STATIC_MAX_LEVEL,
            "Should respect log::STATIC_MAX_LEVEL={:?}, which is done in compile time. level{:?}",
            log::STATIC_MAX_LEVEL,
            level
        );

        fast_log::init(
            Config::new()
                .custom(SendToDartLogger {})
                // .file("audio_chat.log")
                .chan_len(Some(100))
                .level(level),
        )
        .unwrap();

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
