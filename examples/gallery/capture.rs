use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::theme::Theme;
use gpui::{App, AsyncApp, Window, WindowHandle, px};

use crate::{
    pages, script,
    shell::{Choice, Gallery},
};

const SETTLE: Duration = Duration::from_millis(700);
const SCROLL_SETTLE: Duration = Duration::from_millis(250);

/// Shoots one page, or all, in both modes, then quits; exits 1 on failure.
pub fn run(
    window: WindowHandle<Gallery>,
    dir: PathBuf,
    only: Option<usize>,
    scripted: bool,
    cx: &mut App,
) {
    Theme::update(cx, |theme| theme.reduced_motion = true);
    cx.spawn(async move |cx| {
        let outcome = shoot_all(window, &dir, only, scripted, cx).await;
        if let Err(error) = outcome.map(|()| cx.update(|cx| cx.quit())) {
            log::error!("capture failed: {error:#}");
            std::process::exit(1);
        }
    })
    .detach();
}

async fn shoot_all(
    window: WindowHandle<Gallery>,
    dir: &Path,
    only: Option<usize>,
    scripted: bool,
    cx: &mut AsyncApp,
) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    let chosen = pages::ALL
        .iter()
        .enumerate()
        .filter(|(ix, _)| only.is_none_or(|only| only == *ix));
    let gallery = window.update(cx, |_, window, _| number(window))??;
    window.update(cx, |_, window, _| take_posted_moves(window))??;
    for (ix, page) in chosen {
        for (choice, name) in [(Choice::Light, "light"), (Choice::Dark, "dark")] {
            window.update(cx, |gallery, window, cx| {
                gallery.select(ix, cx);
                gallery.fresh(cx);
                gallery.choose(choice, window, cx);
            })?;
            // Only AppKit takes a posted pointer.
            if cfg!(target_os = "macos") {
                script::park(window, cx)?;
            }
            cx.background_executor().timer(SETTLE).await;
            let (viewport, max) = window.update(cx, |gallery, _, _| gallery.scroll_extent())?;
            if viewport <= px(0.0) {
                bail!("page {} has no viewport", page.slug);
            }
            let mut offset = px(0.0);
            for segment in 0.. {
                let suffix = if segment == 0 {
                    String::new()
                } else {
                    format!("-{segment}")
                };
                let path = dir.join(format!("{}-{name}{suffix}.png", page.slug));
                script::shown(window.into(), cx).await?;
                snapshot(gallery, &path)?;
                log::info!("capture: wrote {}", path.display());
                if offset >= max {
                    break;
                }
                offset = (offset + viewport).min(max);
                window.update(cx, |gallery, _, cx| gallery.scroll_to(offset, cx))?;
                cx.background_executor().timer(SCROLL_SETTLE).await;
            }
            if scripted && !page.script.is_empty() {
                window.update(cx, |gallery, _, cx| gallery.select(ix, cx))?;
                cx.background_executor().timer(SETTLE).await;
                let shot = |step: &str| dir.join(format!("{}-{step}-{name}.png", page.slug));
                script::play(window, page.script, shot, cx).await?;
            }
        }
    }
    Ok(())
}

/// A gpui window's AppKit window.
#[cfg(target_os = "macos")]
fn native(window: &Window) -> Result<cocoa::base::id> {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("window has no native handle: {error:?}"))?;
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        bail!("window is not an AppKit window");
    };
    Ok(unsafe { msg_send![appkit.ns_view.as_ptr() as id, window] })
}

/// A gpui window's number, as AppKit and CoreGraphics know it.
#[cfg(target_os = "macos")]
pub fn number(window: &Window) -> Result<u32> {
    use objc::{msg_send, sel, sel_impl};

    let number: i64 = unsafe { msg_send![native(window)?, windowNumber] };
    u32::try_from(number).context("window number out of range")
}

/// Takes posted mouse moves; a tracking area takes real ones.
#[cfg(target_os = "macos")]
fn take_posted_moves(window: &Window) -> Result<()> {
    use cocoa::{appkit::NSWindow, base::YES};

    unsafe { native(window)?.setAcceptsMouseMovedEvents_(YES) };
    log::info!("capture: the window takes posted mouse moves");
    Ok(())
}

/// Windows posts no pointer, so there is nothing to take.
#[cfg(target_os = "windows")]
fn take_posted_moves(_: &Window) -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn take_posted_moves(_: &Window) -> Result<()> {
    bail!("capture needs macOS or Windows window APIs")
}

/// This process's frontmost window above the normal level: a popover or menu.
#[cfg(target_os = "macos")]
pub fn popup_number() -> Result<u32> {
    use core_foundation::{
        base::{CFType, TCFType},
        dictionary::CFDictionary,
        number::CFNumber,
        string::CFString,
    };
    use core_graphics::window;

    let read = |dict: &CFDictionary<CFString, CFType>, key: &'static str| {
        dict.find(CFString::from_static_string(key))
            .and_then(|value| value.downcast::<CFNumber>())
            .and_then(|value| value.to_i64())
    };
    let pid = i64::from(std::process::id());
    let list = window::copy_window_info(
        window::kCGWindowListOptionOnScreenOnly,
        window::kCGNullWindowID,
    )
    .context("window list unavailable")?;
    let id = list
        .iter()
        .map(|item| unsafe { CFDictionary::<CFString, CFType>::wrap_under_get_rule(*item as _) })
        .find(|dict| {
            read(dict, "kCGWindowOwnerPID") == Some(pid)
                && read(dict, "kCGWindowLayer").is_some_and(|layer| layer > 0)
        })
        .and_then(|dict| read(&dict, "kCGWindowNumber"))
        .context("no popup window of this process is on screen")?;
    u32::try_from(id).context("window number out of range")
}

#[cfg(not(target_os = "macos"))]
pub fn popup_number() -> Result<u32> {
    bail!("capture needs macOS window APIs")
}

/// A gpui window's Win32 handle, which `snapshot` takes.
#[cfg(target_os = "windows")]
pub fn number(window: &Window) -> Result<u32> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("window has no native handle: {error:?}"))?;
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        bail!("window is not a Win32 window");
    };
    // Window handles keep their significant 32 bits on 64-bit Windows too.
    u32::try_from(win32.hwnd.get()).context("window handle out of range")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn number(_: &Window) -> Result<u32> {
    bail!("capture needs macOS or Windows window APIs")
}

#[cfg(target_os = "macos")]
pub fn snapshot(number: u32, path: &Path) -> Result<()> {
    use core_graphics::{
        geometry::{CGPoint, CGRect, CGSize},
        window,
    };

    let image = window::create_image(
        CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(0.0, 0.0)),
        window::kCGWindowListOptionIncludingWindow,
        number,
        window::kCGWindowImageBoundsIgnoreFraming | window::kCGWindowImageBestResolution,
    )
    .context("window capture returned nothing")?;
    if image.bits_per_pixel() != 32 {
        bail!(
            "unexpected capture format: {} bits per pixel",
            image.bits_per_pixel()
        );
    }
    let (width, height, stride) = (image.width(), image.height(), image.bytes_per_row());
    let data = image.data();
    let bytes = data.bytes();
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let i = y * stride + x * 4;
            rgba.extend_from_slice(&[bytes[i + 2], bytes[i + 1], bytes[i], bytes[i + 3]]);
        }
    }
    image::save_buffer(
        path,
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .with_context(|| format!("write {}", path.display()))
}

/// A window's client area as Windows composes it, which holds what DirectX presents.
#[cfg(target_os = "windows")]
pub fn snapshot(hwnd: u32, path: &Path) -> Result<()> {
    use windows_sys::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
            DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC, ReleaseDC,
            SelectObject,
        },
        Storage::Xps::PrintWindow,
        UI::WindowsAndMessaging::GetClientRect,
    };

    /// Asks PrintWindow for the whole window, DirectX content included.
    const PW_RENDERFULLCONTENT: u32 = 2;

    /// The contexts and bitmap of one capture, released on every way out.
    struct Surface {
        window: HWND,
        window_dc: HDC,
        memory_dc: HDC,
        bitmap: HBITMAP,
    }

    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                DeleteObject(self.bitmap);
                DeleteDC(self.memory_dc);
                ReleaseDC(self.window, self.window_dc);
            }
        }
    }

    let window = hwnd as usize as HWND;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetClientRect(window, &mut rect) } == 0 {
        bail!("window {hwnd} has no client area");
    }
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
    if width <= 0 || height <= 0 {
        bail!("window {hwnd} is {width}x{height}");
    }
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    unsafe {
        let window_dc = GetDC(window);
        let memory_dc = CreateCompatibleDC(window_dc);
        let bitmap = CreateCompatibleBitmap(window_dc, width, height);
        let surface = Surface {
            window,
            window_dc,
            memory_dc,
            bitmap,
        };
        if window_dc.is_null() || memory_dc.is_null() || bitmap.is_null() {
            bail!("no bitmap of {width}x{height} for window {hwnd}");
        }
        SelectObject(surface.memory_dc, surface.bitmap);
        if PrintWindow(window, surface.memory_dc, PW_RENDERFULLCONTENT) == 0 {
            bail!("PrintWindow could not draw window {hwnd}");
        }
        let mut info: BITMAPINFO = std::mem::zeroed();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width;
        // A negative height asks for the top row first.
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;
        let rows = GetDIBits(
            surface.memory_dc,
            surface.bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr().cast(),
            &mut info,
            DIB_RGB_COLORS,
        );
        if rows == 0 {
            bail!("could not read the pixels of window {hwnd}");
        }
    }
    // Windows hands back BGRA with no alpha.
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        pixel[3] = 255;
    }
    image::save_buffer(
        path,
        &pixels,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .with_context(|| format!("write {}", path.display()))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn snapshot(_: u32, _: &Path) -> Result<()> {
    bail!("capture needs macOS or Windows window APIs")
}
