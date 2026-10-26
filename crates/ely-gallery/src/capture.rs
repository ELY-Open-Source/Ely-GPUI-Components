use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::theme::Theme;
use gpui::{App, AsyncApp, Pixels, Size, Window, WindowHandle, px};

use crate::{
    pages, script,
    shell::{Choice, Gallery},
};

/// The moment every capture draws.
pub const AT: &str = "2026-10-20T14:30:00Z";
const SETTLE: Duration = Duration::from_millis(700);
const STEADY: Duration = Duration::from_millis(250);
// Past a second, so a counter that ticks each second cannot pass for still.
const STILL: Duration = Duration::from_millis(1100);
const PATIENCE: Duration = Duration::from_secs(4);
const SCROLL_SETTLE: Duration = Duration::from_millis(250);

/// Shoots one page, or all, in both modes, then quits; exits 1 on failure.
pub fn run(
    window: WindowHandle<Gallery>,
    asked: Size<Pixels>,
    dir: PathBuf,
    only: Option<usize>,
    scripted: bool,
    cx: &mut App,
) {
    Theme::update(cx, |theme| theme.reduced_motion = true);
    cx.spawn(async move |cx| {
        let outcome = shoot_all(window, asked, &dir, only, scripted, cx).await;
        if let Err(error) = outcome.map(|()| cx.update(|cx| cx.quit())) {
            log::error!("capture failed: {error:#}");
            std::process::exit(1);
        }
    })
    .detach();
}

async fn shoot_all(
    window: WindowHandle<Gallery>,
    asked: Size<Pixels>,
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
    // The system may trim or round a window it places; two captures must share one size.
    let given = window.update(cx, |_, window, _| window.viewport_size())?;
    if given != asked {
        log::warn!("capture: the system gave {given:?}, not {asked:?}; resizing");
        window.update(cx, |_, window, _| window.resize(asked))?;
        cx.background_executor().timer(SETTLE).await;
    }
    window.update(cx, |_, window, _| {
        let size = window.viewport_size();
        anyhow::ensure!(
            size == asked,
            "the window is {size:?}, not the {asked:?} asked"
        );
        let scale = window.scale_factor();
        let (width, height) = (f32::from(size.width), f32::from(size.height));
        log::info!("capture: window {width}x{height} at scale {scale}");
        Ok(())
    })??;
    for (ix, page) in chosen {
        for (choice, name) in [(Choice::Light, "light"), (Choice::Dark, "dark")] {
            window.update(cx, |gallery, window, cx| {
                gallery.select(ix, cx);
                gallery.fresh(cx);
                gallery.choose(choice, window, cx);
            })?;
            script::park(window, cx)?;
            cx.background_executor().timer(SETTLE).await;
            // A picture that lands late grows the page, which would move every segment below it.
            let mut extent = window.update(cx, |gallery, _, _| gallery.scroll_extent())?;
            let start = cx.background_executor().now();
            let mut since = start;
            loop {
                cx.background_executor().timer(STEADY).await;
                let now = window.update(cx, |gallery, _, _| gallery.scroll_extent())?;
                let at = cx.background_executor().now();
                if now != extent {
                    (extent, since) = (now, at);
                } else if at - since >= STILL {
                    break;
                }
                if at - start >= PATIENCE {
                    bail!("page {} kept changing height for {PATIENCE:?}", page.slug);
                }
            }
            let (viewport, _) = extent;
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
                steady(window.into(), &path, |file| snapshot(gallery, file), cx).await?;
                log::info!("capture: wrote {}", path.display());
                // Read again: content that lands late may still grow the page.
                let (_, max) = window.update(cx, |gallery, _, _| gallery.scroll_extent())?;
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

/// Shoots to `path` until it holds still for `STILL`, so neither a picture still decoding nor a counter between ticks is caught; what still moves after `PATIENCE` goes to `NAME.moving.png`, which the comparison passes over.
pub async fn steady(
    frames: gpui::AnyWindowHandle,
    path: &Path,
    shoot: impl Fn(&Path) -> Result<()>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let (next, mask) = (
        path.with_extension("next.png"),
        path.with_extension("moving.png"),
    );
    if mask.exists() {
        std::fs::remove_file(&mask).with_context(|| format!("remove {}", mask.display()))?;
    }
    script::shown(frames, cx).await?;
    shoot(path)?;
    let start = cx.background_executor().now();
    let (mut since, mut moves) = (start, Vec::new());
    let end = loop {
        cx.background_executor().timer(STEADY).await;
        script::shown(frames, cx).await?;
        shoot(&next)?;
        let now = cx.background_executor().now();
        if std::fs::read(path)? != std::fs::read(&next)? {
            moves.push((now, moved(path, &next)?));
            since = now;
        }
        std::fs::rename(&next, path).with_context(|| format!("replace {}", path.display()))?;
        if now - since >= STILL {
            return Ok(());
        }
        if now - start >= PATIENCE {
            break now;
        }
    };
    // Only what moved in the last `STILL` kept the view from holding still; what settled before stays compared.
    let moving = moves
        .into_iter()
        .filter(|(at, _)| end - *at < STILL)
        .map(|(_, mask)| mask)
        .reduce(join)
        .context("a shot that never held still marked no pixel")?;
    moving
        .save(&mask)
        .with_context(|| format!("write {}", mask.display()))?;
    log::warn!(
        "capture: {} still moved after {PATIENCE:?}; its moving pixels are in {}",
        path.display(),
        mask.display()
    );
    Ok(())
}

/// The pixels that differ between two shots.
fn moved(before: &Path, after: &Path) -> Result<image::GrayImage> {
    let (before, after) = (
        image::open(before)?.to_rgb8(),
        image::open(after)?.to_rgb8(),
    );
    let (width, height) = after.dimensions();
    // A new size moves every pixel; what came before it no longer lines up.
    if before.dimensions() != after.dimensions() {
        log::info!(
            "capture: the window went from {:?} to {:?} while it settled",
            before.dimensions(),
            after.dimensions()
        );
        return Ok(image::GrayImage::from_pixel(
            width,
            height,
            image::Luma([255]),
        ));
    }
    Ok(image::GrayImage::from_fn(width, height, |x, y| {
        image::Luma([if before.get_pixel(x, y) == after.get_pixel(x, y) {
            0
        } else {
            255
        }])
    }))
}

/// Two masks' moved pixels together; a later size replaces an earlier one.
fn join(mut earlier: image::GrayImage, later: image::GrayImage) -> image::GrayImage {
    if earlier.dimensions() != later.dimensions() {
        return later;
    }
    for (pixel, moved) in earlier.pixels_mut().zip(later.pixels()) {
        pixel.0[0] = pixel.0[0].max(moved.0[0]);
    }
    earlier
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
    // SAFETY: gpui hands its live NSView; `window` takes no arguments.
    Ok(unsafe { msg_send![appkit.ns_view.as_ptr() as id, window] })
}

/// A gpui window's number, as AppKit and CoreGraphics know it.
#[cfg(target_os = "macos")]
pub fn number(window: &Window) -> Result<u32> {
    use objc::{msg_send, sel, sel_impl};

    // SAFETY: `native` gives gpui's live NSWindow; `windowNumber` returns an NSInteger.
    let number: i64 = unsafe { msg_send![native(window)?, windowNumber] };
    u32::try_from(number).context("window number out of range")
}

/// Takes posted mouse moves; a tracking area takes real ones.
#[cfg(target_os = "macos")]
fn take_posted_moves(window: &Window) -> Result<()> {
    use cocoa::{appkit::NSWindow, base::YES};

    // SAFETY: gpui's live NSWindow, set on the main thread gpui runs this on.
    unsafe { native(window)?.setAcceptsMouseMovedEvents_(YES) };
    log::info!("capture: the window takes posted mouse moves");
    Ok(())
}

/// Windows and X11 get no posted pointer, so there is nothing to take.
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn take_posted_moves(_: &Window) -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn take_posted_moves(_: &Window) -> Result<()> {
    bail!("capture needs macOS, Windows or X11 window APIs")
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
        // SAFETY: each item is a CFDictionary that the retained list keeps alive.
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

/// A gpui window's X11 id, which `snapshot` takes; Wayland lets no client read a window back.
#[cfg(target_os = "linux")]
pub fn number(window: &Window) -> Result<u32> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("window has no native handle: {error:?}"))?;
    match handle.as_raw() {
        RawWindowHandle::Xcb(xcb) => Ok(xcb.window.get()),
        RawWindowHandle::Xlib(xlib) => {
            u32::try_from(xlib.window).context("X11 window id out of range")
        }
        other => bail!("capture needs an X11 window, not {other:?}; unset WAYLAND_DISPLAY"),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn number(_: &Window) -> Result<u32> {
    bail!("capture needs macOS, Windows or X11 window APIs")
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

/// Moves the system cursor to `at` in the window: Windows sends a window a move at the real cursor when what lies under it changes.
#[cfg(target_os = "windows")]
pub fn point_cursor(hwnd: u32, at: gpui::Point<gpui::Pixels>, scale: f32) -> Result<()> {
    use windows_sys::Win32::{
        Foundation::{HWND, POINT},
        Graphics::Gdi::ClientToScreen,
        UI::WindowsAndMessaging::SetCursorPos,
    };

    let window = hwnd as usize as HWND;
    let mut spot = POINT {
        x: (f32::from(at.x) * scale).round() as i32,
        y: (f32::from(at.y) * scale).round() as i32,
    };
    // SAFETY: `spot` is a valid POINT to rewrite; the handle came from the window itself.
    if unsafe { ClientToScreen(window, &mut spot) } == 0 {
        bail!("window {hwnd} placed no point on the screen");
    }
    // SAFETY: plain coordinates; the call reads nothing else.
    if unsafe { SetCursorPos(spot.x, spot.y) } == 0 {
        bail!("the cursor did not move to {}, {}", spot.x, spot.y);
    }
    Ok(())
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
            // SAFETY: the surface made these handles and nothing else frees them.
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
    // SAFETY: `rect` is a valid RECT to write; the handle came from the window itself.
    if unsafe { GetClientRect(window, &mut rect) } == 0 {
        bail!("window {hwnd} has no client area");
    }
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
    if width <= 0 || height <= 0 {
        bail!("window {hwnd} is {width}x{height}");
    }
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    // SAFETY: each handle is checked before use and the surface frees them; `pixels` holds the whole bitmap.
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
    for pixel in pixels.as_chunks_mut::<4>().0 {
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

/// A window's contents as the X server holds them.
#[cfg(target_os = "linux")]
pub fn snapshot(window: u32, path: &Path) -> Result<()> {
    use x11rb::{
        connection::Connection as _,
        protocol::xproto::{ConnectionExt as _, ImageFormat, ImageOrder},
    };

    let (conn, _) = x11rb::connect(None).context("connect to the X server named by DISPLAY")?;
    if conn.setup().image_byte_order != ImageOrder::LSB_FIRST {
        bail!("the X server sends pixels most significant byte first");
    }
    let geometry = conn
        .get_geometry(window)?
        .reply()
        .with_context(|| format!("no geometry for X11 window {window}"))?;
    let (width, height) = (geometry.width, geometry.height);
    let shot = conn
        .get_image(ImageFormat::Z_PIXMAP, window, 0, 0, width, height, u32::MAX)?
        .reply()
        .with_context(|| format!("could not read the pixels of X11 window {window}"))?;
    let size = usize::from(width) * usize::from(height) * 4;
    if !matches!(shot.depth, 24 | 32) || shot.data.len() != size {
        bail!(
            "X11 window {window} came as depth {} with {} bytes, not {size}",
            shot.depth,
            shot.data.len()
        );
    }
    // A 24- or 32-bit ZPixmap holds each pixel as blue, green, red and a pad.
    let rgba: Vec<u8> = shot
        .data
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], 255])
        .collect();
    image::save_buffer(
        path,
        &rgba,
        u32::from(width),
        u32::from(height),
        image::ColorType::Rgba8,
    )
    .with_context(|| format!("write {}", path.display()))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn snapshot(_: u32, _: &Path) -> Result<()> {
    bail!("capture needs macOS, Windows or X11 window APIs")
}
