use ely_gpui_component::{
    Assets, buttons::Button, primitives::FocusScope, rendering::RenderSurface, theme::ActiveTheme,
};
use gpui::{
    App, AppContext, Context, DevicePixels, FocusHandle, IntoElement, ParentElement, Render,
    RenderImage, Size, Styled, Window, WindowOptions,
};
use image::{Frame, RgbaImage};

struct Stripes {
    focus: FocusHandle,
    count: u32,
}

/// `count` vertical stripes across `size`, in BGRA as gpui holds frames.
fn stripes(size: Size<DevicePixels>, count: u32) -> RenderImage {
    let (width, height) = (size.width.0 as u32, size.height.0 as u32);
    let pixels = RgbaImage::from_fn(width, height, |x, _| {
        let lit = (x * count / width).is_multiple_of(2);
        let shade = if lit { 220 } else { 40 };
        image::Rgba([shade, shade, shade, 255])
    });
    RenderImage::new(vec![Frame::new(pixels)])
}

impl Render for Stripes {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let count = self.count;
        let this = cx.entity();
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(colors.bg)
            .text_color(colors.fg)
            .child(
                RenderSurface::new("stripes", 2.0, move |size, _| stripes(size, count))
                    .revision(u64::from(count)),
            )
            .child(
                Button::new("more", "More stripes").on_click(move |_, _, cx| {
                    this.update(cx, |stripes, cx| {
                        stripes.count += 2;
                        cx.notify();
                    });
                }),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Stripes { focus, count: 8 }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
