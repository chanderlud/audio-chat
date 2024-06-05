use std::mem;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use flutter_rust_bridge::frb;
#[cfg(windows)]
use kanal::OneshotSender;
use log::error;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::interval;
#[cfg(windows)]
use winapi::shared::minwindef::TRUE;
#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use winapi::um::winuser::{DispatchMessageW, TranslateMessage};
#[cfg(windows)]
use winapi::um::winuser::{
    GetMessageW, GetMonitorInfoA, InvalidateRect, MONITOR_DEFAULTTONEAREST, MonitorFromWindow, MONITORINFO,
    MoveWindow, SendMessageA, ShowWindow, SW_HIDE, SW_SHOW, WM_CLOSE,
};

use crate::api::overlay::{BACKGROUND_COLOR, FONT_COLOR, FONT_HEIGHT};
#[cfg(windows)]
use crate::api::overlay::windows;

#[frb(opaque)]
#[derive(Clone)]
pub struct Overlay {
    /// the HWND of the overlay window
    #[cfg(windows)]
    _window: Arc<AtomicUsize>,

    /// whether the overlay is enabled
    enabled: Arc<AtomicBool>,

    /// whether the overlay is visible
    visible: Arc<AtomicBool>,

    /// the x position of the overlay
    x: Arc<AtomicI32>,

    /// the y position of the overlay
    y: Arc<AtomicI32>,

    /// the overlay window's width in pixels
    width: Arc<AtomicI32>,

    /// the overlay window's height in pixels
    height: Arc<AtomicI32>,

    /// the font height in pixels
    font_height: Arc<AtomicI32>,

    /// the background color of the overlay window
    background_color: Arc<AtomicU32>,

    /// the primary font color for the overlay
    font_color: Arc<AtomicU32>,

    /// notifies when the window moves or resizes
    window_changed: Arc<Notify>,

    /// notifies when the overlay needs to be redrawn
    redraw_overlay: Arc<Notify>,
}

impl Overlay {
    #[cfg(windows)]
    pub async fn new(
        enabled: bool,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        font_height: i32,
        background_color: u32,
        font_color: u32,
    ) -> Overlay {
        // TODO make this no longer required somehow
        FONT_HEIGHT.store(font_height, Relaxed);
        BACKGROUND_COLOR.store(background_color, Relaxed);
        FONT_COLOR.store(font_color, Relaxed);

        let (rx, tx) = kanal::oneshot_async();

        if enabled {
            let sync_rx = rx.to_sync();
            Self::start_overlay(sync_rx, width, height, x, y);
        } else {
            // set the _window to 0 if the overlay is disabled
            drop(rx);
        }

        let this = Self {
            _window: Arc::new(AtomicUsize::new(tx.recv().await.unwrap_or(0))),
            enabled: Arc::new(AtomicBool::new(enabled)),
            visible: Default::default(),
            x: Arc::new(AtomicI32::new(x)),
            y: Arc::new(AtomicI32::new(y)),
            width: Arc::new(AtomicI32::new(width)),
            height: Arc::new(AtomicI32::new(height)),
            font_height: Arc::new(AtomicI32::new(font_height)),
            background_color: Arc::new(AtomicU32::new(background_color)),
            font_color: Arc::new(AtomicU32::new(font_color)),
            window_changed: Arc::new(Default::default()),
            redraw_overlay: Arc::new(Default::default()),
        };

        let other_this = this.clone();
        tokio::spawn(async move {
            other_this.controller().await;
        });

        this
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    pub fn new() -> crate::api::overlay::Overlay {
        Self {}
    }

    #[cfg(windows)]
    fn start_overlay(rx: OneshotSender<usize>, width: i32, height: i32, x: i32, y: i32) {
        std::thread::spawn(move || unsafe {
            match windows::build_window(width, height, x, y) {
                Ok(hwnd) => {
                    if rx.send(hwnd as usize).is_ok() {
                        let mut msg = mem::zeroed();

                        while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    } else {
                        error!("Failed to send window handle");
                    }
                }
                Err(error) => {
                    error!("Failed to create overlay window: {}", error);
                }
            }
        });
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    fn start_overlay(_rx: OneshotSender<usize>, _width: i32, _height: i32, _x: i32, _y: i32) {}

    /// controls the overlay window
    async fn controller(&self) {
        // redraw the window every second if visible & enabled
        let mut redraw = interval(Duration::from_secs(1));
        let mut ratelimit = interval(Duration::from_millis(10));

        loop {
            let changed = select! {
                _ = self.window_changed.notified() => true,
                _ = self.redraw_overlay.notified() => false,
                _ = redraw.tick() => false,
            };

            if !self.enabled.load(Relaxed) || !self.visible.load(Relaxed) {
                // don't do anything while the overlay is disabled or hidden
                continue;
            }

            if changed {
                self._move_overlay();
            } else {
                self.redraw();
            }

            // prevents anything from happening too fast
            ratelimit.tick().await;
        }
    }

    /// show the overlay window irrespective of platform
    pub fn show(&self) {
        let visible = self.visible.swap(true, Relaxed);

        if visible || !self.enabled.load(Relaxed) {
            return;
        }

        // show the overlay
        self._show();
        // update the overlay
        self.redraw_overlay.notify_one();
        self.window_changed.notify_one();
    }

    /// show the overlay on windows
    #[cfg(windows)]
    fn _show(&self) {
        let hwnd = self._window.load(Relaxed) as HWND;

        unsafe {
            ShowWindow(hwnd, SW_SHOW);
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    fn _show(&self) {}

    /// hide the overlay window irrespective of platform
    pub fn hide(&self) {
        let visible = self.visible.swap(false, Relaxed);

        if !visible || !self.enabled.load(Relaxed) {
            return;
        }

        self._hide();
    }

    /// hide the overlay on windows
    #[cfg(windows)]
    fn _hide(&self) {
        let hwnd = self._window.load(Relaxed) as HWND;

        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    pub fn _hide(&self) {}

    /// move and resize the overlay window
    pub fn move_overlay(&self, x: i32, y: i32, width: i32, height: i32) {
        let old_x = self.x.swap(x, Relaxed);
        let old_y = self.y.swap(y, Relaxed);
        let old_width = self.width.swap(width, Relaxed);
        let old_height = self.height.swap(height, Relaxed);

        if old_width != width || old_x != x || old_y != y || old_height != height {
            self.window_changed.notify_one();
        }
    }

    /// internal function to move and resize the relay on windows
    #[cfg(windows)]
    fn _move_overlay(&self) {
        // get the HWND of the overlay window
        let hwnd = self._window.load(Relaxed) as HWND;

        unsafe {
            MoveWindow(
                hwnd,
                self.x.load(Relaxed),
                self.y.load(Relaxed),
                self.width.load(Relaxed),
                self.height.load(Relaxed),
                TRUE,
            );
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    fn _move_overlay(&self) {}

    /// change the font height (size) of the overlay
    pub fn set_font_height(&self, height: i32) {
        self.font_height.store(height, Relaxed);

        let old_height = FONT_HEIGHT.swap(height, Relaxed);

        if old_height != height {
            self.redraw_overlay.notify_one();
        }
    }

    /// change the background color of the overlay
    pub fn set_background_color(&self, background_color: u32) {
        self.background_color.store(background_color, Relaxed);

        let old_color = BACKGROUND_COLOR.swap(background_color, Relaxed);

        if old_color != background_color {
            self.redraw_overlay.notify_one();
        }
    }

    /// change the font color of the overlay
    pub fn set_font_color(&self, font_color: u32) {
        self.font_color.store(font_color, Relaxed);

        let old_color = FONT_COLOR.swap(font_color, Relaxed);

        if old_color != font_color {
            self.redraw_overlay.notify_one();
        }
    }

    /// redraw the overlay on windows
    #[cfg(windows)]
    fn redraw(&self) {
        // get the HWND of the overlay window
        let hwnd = self._window.load(Relaxed) as HWND;

        // redraw the window
        unsafe {
            InvalidateRect(hwnd, null(), TRUE);
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    fn redraw(&self) {}

    /// enable the overlay
    pub async fn enable(&self) {
        let enabled = self.enabled.swap(true, Relaxed);

        if !enabled {
            self._enable().await;
        }
    }

    /// enables the overlay on windows
    #[cfg(windows)]
    async fn _enable(&self) {
        let (rx, tx) = kanal::oneshot_async();
        let sync_rx = rx.to_sync();

        Self::start_overlay(
            sync_rx,
            self.width.load(Relaxed),
            self.height.load(Relaxed),
            self.x.load(Relaxed),
            self.y.load(Relaxed),
        );

        self._window.store(tx.recv().await.unwrap_or(0), Relaxed);
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    async fn _enable(&self) {}

    /// disable the overlay
    pub fn disable(&self) {
        let disabled = self.enabled.swap(false, Relaxed);

        if !disabled {
            return;
        }

        self._disable();
        self.visible.store(false, Relaxed);
    }

    /// disables the overlay on windows
    #[cfg(windows)]
    fn _disable(&self) {
        let hwnd = self._window.load(Relaxed) as HWND;

        unsafe {
            SendMessageA(hwnd, WM_CLOSE, 0, 0);
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    fn _disable(&self) {}

    /// access the screen resolution for overlay positioning in the front end
    #[cfg(windows)]
    #[frb(sync)]
    pub fn screen_resolution(&self) -> (i32, i32) {
        let hwnd = self._window.load(Relaxed) as HWND;

        unsafe {
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

            let mut monitor_info: MONITORINFO = mem::zeroed::<MONITORINFO>();
            monitor_info.cbSize = mem::size_of::<MONITORINFO>() as u32;

            GetMonitorInfoA(monitor, &mut monitor_info);

            let width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
            let height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;

            (width, height)
        }
    }

    /// non-windows platforms don't have an overlay
    #[cfg(not(windows))]
    #[frb(sync)]
    pub fn screen_resolution(&self) -> (i32, i32) {
        (0, 0)
    }
}
