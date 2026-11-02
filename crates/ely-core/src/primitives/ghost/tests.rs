use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use gpui::{
    Bounds, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId, Hsla, LineLayout,
    NoopTextSystem, Pixels, PlatformTextSystem, RenderGlyphParams, SharedString, Size,
    TestAppContext, TestDispatcher, TextRenderingMode,
};

use super::DragGhost;
use crate::theme::ActiveTheme;

/// Gpui's test text, keeping each font family asked for.
#[derive(Default)]
struct Asked(Mutex<Vec<SharedString>>);

impl PlatformTextSystem for Asked {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> gpui::Result<()> {
        NoopTextSystem.add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        NoopTextSystem.all_font_names()
    }

    fn font_id(&self, descriptor: &Font) -> gpui::Result<FontId> {
        self.0
            .lock()
            .expect("families")
            .push(descriptor.family.clone());
        NoopTextSystem.font_id(descriptor)
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        NoopTextSystem.font_metrics(font_id)
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> gpui::Result<Bounds<f32>> {
        NoopTextSystem.typographic_bounds(font_id, glyph_id)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> gpui::Result<Size<f32>> {
        NoopTextSystem.advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        NoopTextSystem.glyph_for_char(font_id, ch)
    }

    fn glyph_raster_bounds(
        &self,
        params: &RenderGlyphParams,
    ) -> gpui::Result<Bounds<DevicePixels>> {
        NoopTextSystem.glyph_raster_bounds(params)
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> gpui::Result<(Size<DevicePixels>, Vec<u8>)> {
        NoopTextSystem.rasterize_glyph(params, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        NoopTextSystem.layout_line(text, font_size, runs)
    }

    fn recommended_rendering_mode(&self, font_id: FontId, font_size: Pixels) -> TextRenderingMode {
        NoopTextSystem.recommended_rendering_mode(font_id, font_size)
    }

    fn glyph_dilation_for_color(&self, color: Hsla) -> u8 {
        NoopTextSystem.glyph_dilation_for_color(color)
    }
}

#[test]
fn a_ghost_wears_the_theme_font() {
    let asked = Arc::new(Asked::default());
    let mut cx =
        TestAppContext::build_with_text_system(TestDispatcher::new(0), None, asked.clone());
    cx.update(crate::init_for_tests);
    // Gpui draws a drag as a root of its own.
    let (_, window) = cx.add_window_view(|_, _| DragGhost {
        title: "main.rs".into(),
        icon: None,
    });
    window.run_until_parked();
    let family = window.update(|_, cx| cx.theme().font_family.clone());
    cx.quit();
    let asked = asked.0.lock().expect("families").clone();
    assert!(asked.contains(&family), "asked {asked:?}, not {family}");
    assert!(
        !asked.iter().any(|name| name == ".SystemUIFont"),
        "asked {asked:?}"
    );
}
