use std::sync::Arc;

use ely_gpui_component::{
    media::{
        CameraPreview, Device, DeviceKind, DeviceSelector, MicLevelMeter, ModelViewer, Orbit,
        Recording, ScreenRecorderControls,
    },
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    App, ImageSource, IntoElement, ParentElement, RenderImage, SharedString, Styled, Window, div,
    px,
};

use super::sculpt;
use crate::{
    probe::probe,
    ui::{change, keep, picture, section},
};

const OLIVE: &str = asset!("atrium-olive.jpg");
/// The model's frame, in pixels: twice the viewer's points.
const FRAME: (u32, u32) = (960, 640);

/// The demo recording and what it takes in.
#[derive(Clone)]
struct Recorder {
    state: Recording,
    mic: bool,
    camera: bool,
}

pub fn recorder(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "media-recorder",
        || Recorder {
            state: Recording::Idle,
            mic: true,
            camera: false,
        },
        window,
        cx,
    );
    let now = state.read(cx).clone();
    let (record, pause, resume, stop, mic, camera) = (
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
    );
    let controls = ScreenRecorderControls::new("media-recorder", now.state)
        .on_record(move |_, cx| {
            let since = cx.background_executor().now();
            change(&record, cx, |it| it.state = Recording::Running { since })
        })
        .on_pause(move |_, cx| {
            let at = cx.background_executor().now();
            change(&pause, cx, |it| {
                if let Recording::Running { since } = it.state {
                    it.state = Recording::Paused { since, at };
                }
            })
        })
        .on_resume(move |_, cx| {
            let now = cx.background_executor().now();
            change(&resume, cx, |it| {
                if let Recording::Paused { since, at } = it.state {
                    it.state = Recording::Running {
                        since: since + (now - at),
                    };
                }
            })
        })
        .on_stop(move |_, cx| change(&stop, cx, |it| it.state = Recording::Idle))
        .mic(now.mic, move |on, _, cx| change(&mic, cx, |it| it.mic = on))
        .camera(now.camera, move |on, _, cx| {
            change(&camera, cx, |it| it.camera = on)
        });
    section(
        "ScreenRecorderControls",
        "Record, then pause, resume or stop. While it runs, a live dot and the time it has run; the mic and camera toggles choose what it takes.",
        cx,
    )
    .child(probe("media-recorder", div().flex().child(controls)))
}

pub fn camera(cx: &mut App) -> impl IntoElement + use<> {
    let preview = |key: &'static str| div().w(px(300.)).child(CameraPreview::new(key, 1.5));
    section(
        "CameraPreview",
        "What a camera sees, in its own shape, from frames the host hands in. With no frame, the camera is off.",
        cx,
    )
    .child(probe(
        "media-camera",
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(
                div().w(px(300.)).child(
                    CameraPreview::new("media-camera-on", 1.5)
                        .frame(picture(OLIVE))
                        .name("Studio camera"),
                ),
            )
            .child(preview("media-camera-off")),
    ))
}

pub fn mic(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let (small, muted) = (theme.text_size(TextSize::Sm), theme.colors.fg_muted);
    let row = |label: &'static str, level: f32| {
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .w(px(80.))
                    .text_size(small)
                    .text_color(muted)
                    .child(label),
            )
            .child(div().flex_1().child(MicLevelMeter::new(level)))
    };
    section(
        "MicLevelMeter",
        "A microphone's level as it comes in: amber near the top, red at the top.",
        cx,
    )
    .child(probe(
        "media-mic",
        div()
            .w(px(360.))
            .flex()
            .flex_col()
            .gap_3()
            .child(row("Quiet", 0.3))
            .child(row("Speaking", 0.84))
            .child(row("Too loud", 0.98)),
    ))
}

fn device(key: &str, name: &str) -> Device {
    Device {
        key: SharedString::from(key.to_string()),
        name: SharedString::from(name.to_string()),
    }
}

pub fn selectors(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep(
        "media-devices",
        || {
            (
                SharedString::from("mic-desk"),
                SharedString::from("cam-studio"),
            )
        },
        window,
        cx,
    );
    let (mic_now, cam_now) = chosen.read(cx).clone();
    let (mic, cam) = (chosen.clone(), chosen);
    let mics = [
        device("mic-desk", "Desk microphone"),
        device("mic-headset", "Headset"),
    ];
    section(
        "DeviceSelector",
        "A microphone, a camera or a speaker to use, each with its kind's icon. With none attached, it says so and stays shut.",
        cx,
    )
    .child(probe(
        "media-devices",
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                DeviceSelector::new("media-mics", DeviceKind::Microphone, mics)
                    .chosen(mic_now)
                    .on_change(move |key, _, cx| change(&mic, cx, |it| it.0 = key.clone())),
            )
            .child(
                DeviceSelector::new("media-cams", DeviceKind::Camera, [device("cam-studio", "Studio camera")])
                    .chosen(cam_now)
                    .on_change(move |key, _, cx| change(&cam, cx, |it| it.1 = key.clone())),
            )
            .child(DeviceSelector::new("media-speakers", DeviceKind::Speaker, [])),
    ))
}

/// The demo model: its orbit and the frame drawn for it.
struct Model {
    orbit: Orbit,
    frame: Arc<RenderImage>,
}

fn drawn(orbit: Orbit) -> Arc<RenderImage> {
    Arc::new(RenderImage::new(vec![sculpt::draw(
        orbit, FRAME.0, FRAME.1,
    )]))
}

pub fn model(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "media-model",
        || Model {
            orbit: Orbit::START,
            frame: drawn(Orbit::START),
        },
        window,
        cx,
    );
    let (orbit, frame) = {
        let now = state.read(cx);
        (now.orbit, now.frame.clone())
    };
    let viewer = ModelViewer::new("media-model", 1.5, orbit)
        .frame(ImageSource::Render(frame))
        .on_orbit(move |orbit, window, cx| {
            let old = state.update(cx, |model, cx| {
                model.orbit = orbit;
                cx.notify();
                std::mem::replace(&mut model.frame, drawn(orbit))
            });
            cx.drop_image(old, Some(window));
        });
    section(
        "3DModelViewer",
        "A model the host renders, a frame for each view. A drag or the arrows turn around it; Command-scroll, + and - bring it nearer; 0 or the reset button returns.",
        cx,
    )
    .child(probe("media-model", div().w(px(480.)).child(viewer)))
}
