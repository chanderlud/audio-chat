use std::mem;
use std::ptr::{null, null_mut};
use std::sync::atomic::Ordering::Relaxed;

use log::info;
use widestring::U16CString;
use winapi::shared::minwindef::{BYTE, HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, POINT, RECT, SIZE};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION, CreateCompatibleDC, DeleteDC};
use winapi::um::wingdi::{DeleteObject, SelectObject};
use winapi::um::winuser::{
    BeginPaint, CreateWindowExW, CS_HREDRAW, CS_VREDRAW, DefWindowProcW, DestroyWindow, EndPaint,
    GetClientRect, GetDC, HWND_TOPMOST, PostQuitMessage, RegisterClassW,
    ReleaseDC, SetWindowPos, ShowWindow, SW_HIDE, SWP_NOMOVE, SWP_NOSIZE,
    ULW_ALPHA, UpdateLayeredWindow, UpdateWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    WS_POPUP, WS_VISIBLE,
};
use winapi::um::winuser::{WM_CLOSE, WM_CREATE, WM_DESTROY, WM_PAINT};

use gdiplus_sys2::{
    GdipCreateBitmapFromScan0, GdipCreateFont, GdipCreateFontFamilyFromName,
    GdipCreateHBITMAPFromBitmap, GdipCreateSolidFill, GdipDrawString, GdipFillRectangle,
    GdipGetImageGraphicsContext, GdiplusStartup, GdiplusStartupInput,
    GdipMeasureString, GdipSetTextRenderingHint, GdipStringFormatGetGenericDefault, GpFont, GpGraphics,
    GpStringFormat, REAL, RectF,
};

use crate::api::overlay::{BACKGROUND_COLOR, CONNECTED, ErrorKind, FONT_COLOR, FONT_HEIGHT, LATENCY, LOSS, Result};
use crate::api::overlay::color::{BAD_COLOR, percent_to_color};

pub(crate) const CLASS_NAME: &str = "audio_chat_overlay";

pub(crate) unsafe fn build_window(
    width: i32,
    height: i32,
    x: i32,
    y: i32,
) -> Result<HWND> {
    let input: GdiplusStartupInput = GdiplusStartupInput {
        GdiplusVersion: 1,
        DebugEventCallback: mem::zeroed(),
        SuppressBackgroundThread: 0,
        SuppressExternalCodecs: 0,
    };

    let mut token = 0;
    let status = GdiplusStartup(&mut token, &input, null_mut());
    info!("GdiplusStartup status: {}", status);

    let class_name = U16CString::from_str(CLASS_NAME)?;
    let window_name = U16CString::from_str("Overlay")?;

    let h_instance: HINSTANCE = GetModuleHandleA(null());

    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: class_name.as_ptr(),
    };

    RegisterClassW(&wc);

    let hwnd: HWND = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_POPUP | WS_VISIBLE,
        x,
        y,
        width,
        height,
        null_mut(),
        null_mut(),
        h_instance,
        null_mut(),
    );

    if hwnd.is_null() {
        return Err(ErrorKind::CreateWindow.into());
    }

    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        x,
        y,
        width,
        height,
        SWP_NOMOVE | SWP_NOSIZE,
    );

    ShowWindow(hwnd, SW_HIDE);
    UpdateWindow(hwnd);

    Ok(hwnd)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => 0,
        WM_PAINT => {
            let mut info = mem::zeroed();
            let _hdc = BeginPaint(hwnd, &mut info);

            draw_overlay(hwnd);

            EndPaint(hwnd, &info);
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// TODO pre measure text to determine the necessary width
unsafe fn draw_overlay(hwnd: HWND) {
    let hdc_screen = GetDC(null_mut());
    let hdc_mem = CreateCompatibleDC(hdc_screen);

    let mut rect: RECT = mem::zeroed();
    // get the client area
    GetClientRect(hwnd, &mut rect);

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    let mut bitmap = mem::zeroed();
    GdipCreateBitmapFromScan0(width, height, 0, 925707, null_mut::<BYTE>(), &mut bitmap);

    let mut graphics = mem::zeroed();
    GdipGetImageGraphicsContext(bitmap.cast(), &mut graphics);

    // set the font rendering to smooth
    GdipSetTextRenderingHint(graphics, 4);

    let mut background_brush = mem::zeroed();
    GdipCreateSolidFill(BACKGROUND_COLOR.load(Relaxed), &mut background_brush);

    GdipFillRectangle(
        graphics,
        background_brush.cast(),
        0_f32,
        0_f32,
        width as f32,
        height as f32,
    );

    let font_name = U16CString::from_str("Inconsolata").unwrap();

    let mut font_family = mem::zeroed();
    GdipCreateFontFamilyFromName(font_name.as_ptr(), null_mut(), &mut font_family);

    let mut font = mem::zeroed();
    GdipCreateFont(
        font_family,
        FONT_HEIGHT.load(Relaxed) as REAL,
        0,
        0,
        &mut font,
    );

    let mut string_format = mem::zeroed();
    GdipStringFormatGetGenericDefault(&mut string_format);

    let bounding = draw_text(
        "Latency:",
        (0_f32, 0_f32),
        FONT_COLOR.load(Relaxed),
        graphics,
        font,
        string_format,
    );

    let latency = LATENCY.load(Relaxed);
    let color = percent_to_color(latency as f64 / 200_f64);

    let bounding = draw_text(
        latency.to_string().as_str(),
        (bounding.Width, 0_f32),
        color.argb(),
        graphics,
        font,
        string_format,
    );

    let bounding = draw_text(
        "Loss:",
        (bounding.X + bounding.Width + 30_f32, 0_f32),
        FONT_COLOR.load(Relaxed),
        graphics,
        font,
        string_format,
    );

    let loss = LOSS.load(Relaxed);
    let color = percent_to_color(loss);

    let bounding = draw_text(
        &format!("{:.2}%", loss * 100_f64),
        (bounding.X + bounding.Width, 0_f32),
        color.argb(),
        graphics,
        font,
        string_format,
    );

    if !CONNECTED.load(Relaxed) {
        draw_text(
            "Disconnected",
            (bounding.X + bounding.Width + 30_f32, 0_f32),
            BAD_COLOR.argb(),
            graphics,
            font,
            string_format,
        );
    }

    let mut h_bitmap = mem::zeroed();
    GdipCreateHBITMAPFromBitmap(bitmap, &mut h_bitmap, 0);

    let old_bitmap = SelectObject(hdc_mem, h_bitmap.cast());

    let mut point_source = POINT { x: 0, y: 0 };
    let mut size = SIZE {
        cx: width,
        cy: height,
    };

    let mut blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA,
    };

    UpdateLayeredWindow(
        hwnd,
        hdc_screen,
        null_mut(),
        &mut size,
        hdc_mem,
        &mut point_source,
        0,
        &mut blend,
        ULW_ALPHA,
    );

    SelectObject(hdc_mem, old_bitmap);
    DeleteObject(h_bitmap as *mut _);
    DeleteDC(hdc_mem);
    ReleaseDC(null_mut(), hdc_screen);
}

unsafe fn draw_text(
    text: &str,
    position: (f32, f32),
    color: u32,
    graphics: *mut GpGraphics,
    font: *mut GpFont,
    string_format: *mut GpStringFormat,
) -> RectF {
    let mut brush = mem::zeroed();
    GdipCreateSolidFill(color, &mut brush);

    let point_f = RectF {
        X: position.0,
        Y: position.1,
        Width: 0_f32,
        Height: 0_f32,
    };

    let message = U16CString::from_str(text).unwrap();

    let mut bounding_box = mem::zeroed();
    GdipMeasureString(
        graphics,
        message.as_ptr(),
        -1,
        font,
        &point_f,
        string_format,
        &mut bounding_box,
        null_mut(),
        null_mut(),
    );

    GdipDrawString(
        graphics,
        message.as_ptr(),
        -1,
        font,
        &point_f,
        string_format,
        brush.cast(),
    );

    bounding_box
}
