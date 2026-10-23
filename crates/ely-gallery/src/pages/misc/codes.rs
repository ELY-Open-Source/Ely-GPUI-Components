use std::sync::Arc;

use ely_gpui_component::misc::{Captcha, CaptchaState, QrCodeScanner};
use gpui::{App, IntoElement, ParentElement, RenderImage, Styled, Window, div, px};

use crate::probe::probe;
use crate::ui::{picture, section};

const ANSWERS: [&str; 2] = ["W7XK", "Q3NP"];
const PICTURES: &str = asset!("captcha");

/// The frame a camera would hand in: a QR code on a card over a dim scene, in gpui's BGRA pixels.
fn scene() -> Arc<RenderImage> {
    let code = qrcode::QrCode::new("https://example.com/ely").expect("the link fits a QR code");
    let (modules, scale, quiet) = (code.width() as u32, 4, 4);
    let (width, height, side) = (360, 240, (modules + quiet * 2) * scale);
    let (left, top) = ((width - side) / 2, (height - side) / 2);
    let mut pixels = image::RgbaImage::from_fn(width, height, |x, y| {
        let shade = (60 + (x + y) / 10) as u8;
        image::Rgba([shade + 12, shade + 6, shade, 255])
    });
    let dark: Vec<bool> = code
        .to_colors()
        .into_iter()
        .map(|color| color == qrcode::Color::Dark)
        .collect();
    for (x, y) in (0..side).flat_map(|x| (0..side).map(move |y| (x, y))) {
        let module = (x / scale)
            .checked_sub(quiet)
            .zip((y / scale).checked_sub(quiet));
        let ink = module
            .filter(|(mx, my)| *mx < modules && *my < modules)
            .is_some_and(|(mx, my)| dark[(my * modules + mx) as usize]);
        let shade = if ink { 20 } else { 245 };
        pixels.put_pixel(left + x, top + y, image::Rgba([shade, shade, shade, 255]));
    }
    Arc::new(RenderImage::new(vec![image::Frame::new(pixels)]))
}

pub fn render(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let frame = window.use_keyed_state("scanner-scene", cx, |_, _| scene());
    let desk = window.use_keyed_state("captcha-desk", cx, |_, _| (0, CaptchaState::Asking));
    let (ix, state) = *desk.read(cx);
    let answer = ANSWERS[ix];
    let (checked, refreshed) = (desk.clone(), desk);
    let picture = picture(format!("{PICTURES}/{}.png", answer.to_lowercase()));
    div()
        .child(
            section(
                "QRCodeScanner",
                "A camera's view with a square to aim at, reading QR codes off the main thread from frames the host hands in. gpui has no camera, so the gallery hands in one frame of its own.",
                cx,
            )
            .child(
                div().w(px(280.0)).child(
                    QrCodeScanner::new("scanner", 1.5, |text, _, _| log::info!("gallery: scanned {text}"))
                        .frame(frame.read(cx).clone()),
                ),
            ),
        )
        .child(
            section(
                "Captcha",
                "A picture of characters to copy, from the owner, who checks the answer and says how it stood; the refresh asks for another picture.",
                cx,
            )
            .child(probe(
                "captcha",
                div().w(px(280.0)).child(
                    Captcha::new("captcha", picture, 3.0, move |text, _, cx| {
                            checked.update(cx, |desk, cx| {
                                desk.1 = match text.eq_ignore_ascii_case(answer) {
                                    true => CaptchaState::Passed,
                                    false => CaptchaState::Wrong,
                                };
                                cx.notify();
                            })
                        })
                        .state(state)
                        .on_refresh(move |_, cx| {
                            refreshed.update(cx, |desk, cx| {
                                *desk = ((desk.0 + 1) % ANSWERS.len(), CaptchaState::Asking);
                                cx.notify();
                            })
                        }),
                ),
            )),
        )
        .child(section(
            "BarcodeDisplay · Signature · Rating Widget",
            "Each has one home, shown on its own chapter's page: BarcodeDisplay is data_display::Barcode, Signature is forms::SignaturePad, and Rating Widget is forms::Rating.",
            cx,
        ))
}
