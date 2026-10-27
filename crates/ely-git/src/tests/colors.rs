use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use gpui::{
    Bounds, Context, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId, Hsla, IntoElement,
    LineLayout, NoopTextSystem, ParentElement, Pixels, PlatformTextSystem, Render,
    RenderGlyphParams, Size, Styled, TestAppContext, TestDispatcher, TextRenderingMode, Window,
    div, hsla, px,
};

use super::super::{DiffLayout, DiffViewer};
use crate::theme::{ActiveTheme, Mode, Palette, Theme};

/// Gpui's test text, keeping each drawn glyph's color.
#[derive(Default)]
struct Painted(Mutex<Vec<Hsla>>);

impl PlatformTextSystem for Painted {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> gpui::Result<()> {
        NoopTextSystem.add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        NoopTextSystem.all_font_names()
    }

    fn font_id(&self, descriptor: &Font) -> gpui::Result<FontId> {
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
        self.0.lock().expect("colors").push(color);
        0
    }
}

/// One added line of `path`, drawn in `layout`.
struct Shown {
    path: &'static str,
    line: &'static str,
    layout: DiffLayout,
}

impl Render for Shown {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(640.0)).h(px(240.0)).child(
            DiffViewer::new("diff", self.path, "", &format!("{}\n", self.line))
                .headless()
                .layout(self.layout),
        )
    }
}

/// The glyph colors drawn, then keyword, number, comment.
fn drawn(path: &'static str, line: &'static str, layout: DiffLayout) -> (Vec<Hsla>, [Hsla; 3]) {
    let painted = Arc::new(Painted::default());
    let mut cx =
        TestAppContext::build_with_text_system(TestDispatcher::new(0), None, painted.clone());
    // Hues no other part of the view draws.
    let own = [
        hsla(0.11, 1.0, 0.5, 1.0),
        hsla(0.37, 1.0, 0.5, 1.0),
        hsla(0.83, 1.0, 0.5, 1.0),
    ];
    cx.update(|cx| {
        crate::init_for_tests(cx);
        let mut palette = Palette::light(false);
        [
            palette.syntax.keyword,
            palette.syntax.number,
            palette.syntax.comment,
        ] = own;
        Theme::set_palette(Mode::Light, Some(palette), cx);
        Theme::set_mode_now(Mode::Light, cx);
    });
    let (_, window) = cx.add_window_view(|_, _| Shown { path, line, layout });
    window.run_until_parked();
    window.update(|window, _| window.refresh());
    window.run_until_parked();
    let syntax = window.update(|_, cx| cx.theme().colors.syntax.clone());
    assert_eq!([syntax.keyword, syntax.number, syntax.comment], own);
    cx.quit();
    let colors = painted.0.lock().expect("colors").clone();
    (colors, own)
}

#[test]
fn the_drawn_diff_reads_its_own_language() {
    for layout in [DiffLayout::Unified, DiffLayout::Split] {
        let (rust, [keyword, number, comment]) = drawn("main.rs", "return 42", layout);
        assert!(
            rust.contains(&keyword) && rust.contains(&number),
            "{layout:?}: Rust is code"
        );
        let (markdown, _) = drawn("notes.md", "return 42", layout);
        assert!(
            !markdown.contains(&keyword) && !markdown.contains(&number),
            "{layout:?}: Markdown is prose"
        );
        let (python, _) = drawn("a.py", "# return 42", layout);
        assert!(
            python.contains(&comment) && !python.contains(&keyword),
            "{layout:?}: Python's # opens a comment"
        );
    }
}
