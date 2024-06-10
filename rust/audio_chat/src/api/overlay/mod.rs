#[cfg(windows)]
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize};
use std::sync::Arc;

use atomic_float::AtomicF64;
use lazy_static::lazy_static;
#[cfg(windows)]
use widestring::error::ContainsNul;

#[cfg(windows)]
mod color;
pub mod overlay;
#[cfg(windows)]
mod windows;

lazy_static! {
    pub(crate) static ref LATENCY: Arc<AtomicUsize> = Default::default();
    pub(crate) static ref LOSS: Arc<AtomicF64> = Default::default();
    pub(crate) static ref CONNECTED: Arc<AtomicBool> = Default::default();
    static ref FONT_HEIGHT: Arc<AtomicI32> = Default::default();
    static ref BACKGROUND_COLOR: Arc<AtomicU32> = Default::default();
    static ref FONT_COLOR: Arc<AtomicU32> = Default::default();
}

#[cfg(windows)]
type Result<T> = std::result::Result<T, Error>;

#[cfg(windows)]
#[derive(Debug)]
struct Error {
    _kind: ErrorKind,
}

#[cfg(windows)]
#[derive(Debug)]
enum ErrorKind {
    CreateWindow,
    ContainsNul,
}

#[cfg(windows)]
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self._kind {
            ErrorKind::CreateWindow => write!(f, "failed to create window"),
            ErrorKind::ContainsNul => write!(f, "string contains nul byte"),
        }
    }
}

#[cfg(windows)]
impl From<ContainsNul<u16>> for Error {
    fn from(_: ContainsNul<u16>) -> Self {
        Error {
            _kind: ErrorKind::ContainsNul,
        }
    }
}

#[cfg(windows)]
impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { _kind: kind }
    }
}

#[cfg(windows)]
#[cfg(test)]
mod tests {
    use fast_log::Config;
    use log::{LevelFilter, Log};
    use tokio::time::sleep;

    use crate::api::overlay::color::Color;
    use crate::api::overlay::overlay::Overlay;
    use crate::api::overlay::{CONNECTED, LATENCY, LOSS};

    #[tokio::test]
    async fn test_overlay() {
        let logger = fast_log::init(
            Config::new()
                .chan_len(Some(100))
                .file("tests.log")
                .level(LevelFilter::Debug),
        )
        .unwrap();

        let bcolor = Color::new(0, 0, 0, 125);
        let fcolor = Color::new(255, 255, 255, 255);
        let overlay = Overlay::new(true, 100, 0, 600, 36, 36, bcolor.argb(), fcolor.argb()).await;

        sleep(std::time::Duration::from_secs(1)).await;

        overlay.show();

        CONNECTED.store(false, std::sync::atomic::Ordering::Relaxed);

        for x in 0..=500 {
            sleep(std::time::Duration::from_millis(100)).await;

            LOSS.store(x as f64 / 500_f64, std::sync::atomic::Ordering::Relaxed);

            if x == 250 {
                CONNECTED.store(false, std::sync::atomic::Ordering::Relaxed);
            }

            // if x % 10 == 0 {
            //     overlay.move_overlay(600 + x, 2, 545 + x * 8, 36 + x);
            //     overlay.set_font_height(36 + x / 2);
            // }

            if x % 5 == 0 {
                LATENCY.store(x as usize / 5, std::sync::atomic::Ordering::Relaxed);
            }
        }

        logger.flush();
    }
}
