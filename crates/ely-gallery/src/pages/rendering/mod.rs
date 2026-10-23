use std::hash::{DefaultHasher, Hash, Hasher};

use ely_gpui_component::{forms::Slider, rendering::RenderSurface, theme::ActiveTheme};
use gpui::{
    AnyElement, App, DevicePixels, Hsla, IntoElement, ParentElement, RenderImage, Rgba, Size,
    Styled, Window, div, px,
};
use image::{Frame, RgbaImage};

use super::Page;
use crate::{
    probe::probe,
    step::Step,
    ui::{keep, section, set},
};

pub const PAGE: Page = Page {
    number: 44,
    slug: "rendering",
    title: "Rendering",
    summary: "Pixels the host draws at the box's size in device pixels, drawn again for a new size or content.",
    render,
    script: &[
        Step::Click("surface-slider"),
        Step::Key("right"),
        Step::Key("right"),
        Step::Wait(300),
        Step::Shot("moved"),
    ],
};

/// Where the sources sit, as shares of the frame; the first moves.
const SOURCES: [(f32, f32); 3] = [(0.2, 0.5), (0.62, 0.3), (0.78, 0.74)];
const BANDS: f32 = 8.0;

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div().child(surface(window, cx)).into_any_element()
}

fn surface(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let place = keep("surface-place", || 0.2, window, cx);
    let at = *place.read(cx);
    let colors = &cx.theme().colors;
    let (ground, ink) = (colors.sunken, colors.accent);
    section(
        "RenderSurface",
        "The host draws every pixel of each frame at the box's size in device pixels, here a field of three sources in contour bands. The slider moves a source; the frame draws again only for a new size or revision. Shaders, cameras, video and other GPU work run on the host's side and hand their frames in: gpui shares no device, so zero-copy textures and Metal, OpenGL, Vulkan or Direct3D surfaces have no way in.",
        cx,
    )
    .child(
        div().w(px(720.0)).child(
            RenderSurface::new("field", 2.0, move |size, _| field(size, at, ground, ink))
                .revision(revision(at, ground, ink)),
        ),
    )
    .child(probe(
        "surface-slider",
        div().w(px(320.0)).child(
            Slider::new("surface-place", f64::from(at) * 100.0)
                .step(5.0)
                .on_change(move |next, _, cx| set(&place, next as f32 / 100.0, cx)),
        ),
    ))
}

/// What the frame reads, so a theme fade draws it again.
fn revision(at: f32, ground: Hsla, ink: Hsla) -> u64 {
    let mut hasher = DefaultHasher::new();
    for value in [at, ground.h, ground.s, ground.l, ink.h, ink.s, ink.l] {
        value.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

/// The field at `size`, one source at `at` across, banded from `ground` to `ink`, in BGRA.
fn field(size: Size<DevicePixels>, at: f32, ground: Hsla, ink: Hsla) -> RenderImage {
    let (width, height) = (size.width.0 as u32, size.height.0 as u32);
    let (ground, ink) = (Rgba::from(ground), Rgba::from(ink));
    let reach = height as f32 * 0.22;
    let mut sources = SOURCES.map(|(x, y)| (x * width as f32, y * height as f32));
    sources[0].0 = at * width as f32;
    let pixels = RgbaImage::from_fn(width, height, |x, y| {
        let heat: f32 = sources
            .iter()
            .map(|(sx, sy)| {
                let (dx, dy) = ((x as f32 - sx) / reach, (y as f32 - sy) / reach);
                1.0 / (1.0 + dx * dx + dy * dy)
            })
            .sum();
        let band = (heat.min(1.0) * BANDS).floor() / BANDS;
        let mix = |from: f32, to: f32| ((from + (to - from) * band) * 255.0).round() as u8;
        image::Rgba([
            mix(ground.b, ink.b),
            mix(ground.g, ink.g),
            mix(ground.r, ink.r),
            255,
        ])
    });
    RenderImage::new(vec![Frame::new(pixels)])
}
