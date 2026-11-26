use std::rc::Rc;

use gpui::{TestAppContext, px, size};

use super::{MarkdownMode, frames, open};
use crate::HeldPicture;

const WEB: &str = "![logo](https://ely.dev/logo.png)\n";

/// A web picture the owner fetched draws in Visual and Preview.
#[gpui::test]
fn a_held_web_picture_draws_in_place(cx: &mut TestAppContext) {
    let mut png = Vec::new();
    image::RgbaImage::new(120, 60)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("a png");
    let held = HeldPicture::new(gpui::ImageFormat::Png, png);
    for mode in [MarkdownMode::Visual, MarkdownMode::Preview] {
        let (document, cx) = open(WEB, mode, cx);
        assert!(cx.debug_bounds("markdown-picture").is_none(), "unfetched, a note");
        let pictures = [("https://ely.dev/logo.png".to_string(), Ok(held.clone()))].into();
        document.update(cx, |document, cx| document.set_pictures(Rc::new(pictures), cx));
        let mut drawn = None;
        for _ in 0..50 {
            frames(cx);
            drawn = cx.debug_bounds("markdown-picture");
            if drawn.is_some() {
                break;
            }
        }
        assert_eq!(drawn.expect("drew").size, size(px(120.0), px(60.0)), "{mode:?}");
    }
}
