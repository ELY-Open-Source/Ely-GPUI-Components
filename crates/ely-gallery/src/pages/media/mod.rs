mod audio;
mod devices;
mod edit;
mod images;
mod sculpt;
mod video;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 26,
    slug: "media",
    title: "Media",
    summary: "Pictures, video and sound: looking closely, cropping and marking up, playing, and the controls and devices around them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("media-viewer", 320.0, 240.0),
    Step::UpAt("media-viewer", 320.0, 240.0),
    Step::Key("="),
    Step::Key("="),
    Step::Wait(300),
    Step::Shot("zoomed"),
    Step::Key("r"),
    Step::Wait(500),
    Step::Shot("turned"),
    Step::DownAt("media-thumbs", 237.0, 50.0),
    Step::UpAt("media-thumbs", 237.0, 50.0),
    Step::Wait(500),
    Step::Shot("lightbox"),
    Step::Key("escape"),
    Step::Wait(300),
    Step::DownAt("media-cropper", 458.0, 317.0),
    Step::DragTo("media-cropper", 400.0, 280.0),
    Step::UpAt("media-cropper", 400.0, 280.0),
    Step::Wait(200),
    Step::Shot("cropped"),
    Step::DownAt("media-annotator", 500.0, 80.0),
    Step::UpAt("media-annotator", 500.0, 80.0),
    Step::Key("v"),
    Step::DownAt("media-annotator", 168.0, 310.0),
    Step::UpAt("media-annotator", 168.0, 310.0),
    Step::Wait(200),
    Step::Shot("chosen"),
    Step::Key("b"),
    Step::DownAt("media-annotator", 60.0, 90.0),
    Step::DragTo("media-annotator", 200.0, 200.0),
    Step::UpAt("media-annotator", 200.0, 200.0),
    Step::Wait(200),
    Step::Shot("marked"),
    Step::DownAt("media-upload", 388.0, 307.0),
    Step::UpAt("media-upload", 388.0, 307.0),
    Step::Wait(300),
    Step::Shot("emptied"),
    Step::HoverAt("media-scrubber", 320.0, 31.0),
    Step::Wait(200),
    Step::Shot("scrub-tip"),
    Step::DownAt("media-strip", 467.0, 25.0),
    Step::DragTo("media-strip", 560.0, 25.0),
    Step::UpAt("media-strip", 560.0, 25.0),
    Step::Wait(200),
    Step::Shot("trimmed"),
    Step::DownAt("media-subtitles", 80.0, 142.0),
    Step::UpAt("media-subtitles", 80.0, 142.0),
    Step::Type("00:20.000"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("retimed"),
    Step::HoverAt("media-waveform", 400.0, 48.0),
    Step::Wait(200),
    Step::Shot("wave-tip"),
    Step::DownAt("media-speed", 30.0, 16.0),
    Step::UpAt("media-speed", 30.0, 16.0),
    Step::Wait(300),
    Step::Shot("speeds"),
    Step::Key("escape"),
    Step::DownAt("media-volume", 14.0, 14.0),
    Step::UpAt("media-volume", 14.0, 14.0),
    Step::Wait(200),
    Step::Shot("muted"),
    Step::DownAt("media-playlist", 200.0, 124.0),
    Step::UpAt("media-playlist", 200.0, 124.0),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("played"),
    Step::DownAt("media-transport", 180.0, 20.0),
    Step::UpAt("media-transport", 180.0, 20.0),
    Step::Wait(300),
    Step::DownAt("media-recorder", 45.0, 18.0),
    Step::UpAt("media-recorder", 45.0, 18.0),
    Step::Wait(1500),
    Step::Shot("recording"),
    Step::DownAt("media-devices", 140.0, 18.0),
    Step::UpAt("media-devices", 140.0, 18.0),
    Step::Wait(300),
    Step::Shot("devices"),
    Step::Key("escape"),
    Step::DownAt("media-model", 240.0, 160.0),
    Step::DragTo("media-model", 360.0, 200.0),
    Step::UpAt("media-model", 360.0, 200.0),
    Step::Key("="),
    Step::Wait(300),
    Step::Shot("orbited"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(images::viewer(cx))
        .child(images::thumbnails(window, cx))
        .child(edit::cropper(window, cx))
        .child(edit::upload(window, cx))
        .child(edit::annotator(window, cx))
        .child(video::video(window, cx))
        .child(video::scrubber(window, cx))
        .child(video::strip(window, cx))
        .child(video::subtitles(window, cx))
        .child(audio::player(window, cx))
        .child(audio::waveform(window, cx))
        .child(audio::spectrum(window, cx))
        .child(audio::volume_control(window, cx))
        .child(audio::speed(window, cx))
        .child(audio::transport(window, cx))
        .child(audio::playlist(window, cx))
        .child(devices::recorder(window, cx))
        .child(devices::camera(cx))
        .child(devices::mic(cx))
        .child(devices::selectors(window, cx))
        .child(devices::model(window, cx))
        .into_any_element()
}
