use std::time::Duration;

use ely_gpui_component::media::{Cue, Scrubber, SubtitleEditor, VideoPlayer, VideoThumbnailStrip};
use gpui::{
    App, Entity, IntoElement, ParentElement, SharedString, Styled, TaskExt as _, Window, div, px,
};

use crate::{
    probe::probe,
    ui::{keep, picture, section, set},
};

const FRAMES: [&str; 3] = [
    asset!("atrium.jpg"),
    asset!("atrium-stair.jpg"),
    asset!("atrium-olive.jpg"),
];
const LENGTH: Duration = Duration::from_secs(90);
/// How often the demo's clock moves while it plays.
const TICK: Duration = Duration::from_millis(250);

/// The demo video: where it stands, how it plays, its trim and its captions.
#[derive(Clone)]
struct Playback {
    at: Duration,
    playing: bool,
    ticking: bool,
    speed: f32,
    captions: bool,
    trim: (Duration, Duration),
    cues: Vec<Cue>,
}

fn cue(key: &str, start: u64, end: u64, text: &str) -> Cue {
    Cue {
        key: key.to_string().into(),
        start: Duration::from_secs(start),
        end: Duration::from_secs(end),
        text: text.to_string().into(),
    }
}

fn frame(at: Duration) -> &'static str {
    FRAMES[(at.as_secs() / 30) as usize % FRAMES.len()]
}

/// Runs the demo's clock while it plays, one loop at a time.
fn tick(state: &Entity<Playback>, window: &mut Window, cx: &mut App) {
    if state.update(cx, |now, _| std::mem::replace(&mut now.ticking, true)) {
        return;
    }
    let state = state.clone();
    window
        .spawn(cx, async move |cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let going = cx.update(|_, cx| {
                    state.update(cx, |now, cx| {
                        let step = TICK.mul_f32(now.speed);
                        now.at = (now.at + step).min(LENGTH);
                        now.playing &= now.at < LENGTH;
                        now.ticking = now.playing;
                        cx.notify();
                        now.playing
                    })
                })?;
                if !going {
                    return anyhow::Ok(());
                }
            }
        })
        .detach_and_log_err(cx);
}

/// The one video every section shows.
fn state(window: &mut Window, cx: &mut App) -> Entity<Playback> {
    keep(
        "media-video",
        || Playback {
            at: Duration::from_secs(34),
            playing: false,
            ticking: false,
            speed: 1.0,
            captions: true,
            trim: (Duration::from_secs(12), Duration::from_secs(66)),
            cues: vec![
                cue("cue-1", 2, 9, "Morning comes in through the tall window."),
                cue("cue-2", 31, 38, "Light falls on the stair."),
                cue("cue-3", 61, 70, "The shadow walks the length of the hall."),
            ],
        },
        window,
        cx,
    )
}

pub fn video(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let caption: Option<SharedString> = now
        .cues
        .iter()
        .find(|cue| cue.start <= now.at && now.at < cue.end)
        .map(|cue| cue.text.clone());
    let (play, seek, speed, captions) =
        (state.clone(), state.clone(), state.clone(), state.clone());
    let player = VideoPlayer::new(
        "media-player",
        Some(picture(frame(now.at))),
        1.5,
        LENGTH,
        now.at,
    )
    .loaded(LENGTH.mul_f32(0.8))
    .playing(now.playing)
    .speed(now.speed)
    .captions(now.captions, caption)
    .on_play(move |on, window, cx| {
        play.update(cx, |now, cx| {
            now.playing = on;
            cx.notify();
        });
        if on {
            tick(&play, window, cx);
        }
    })
    .on_seek(move |at, _, cx| {
        seek.update(cx, |now, cx| {
            now.at = at;
            cx.notify();
        })
    })
    .on_speed(move |to, _, cx| {
        speed.update(cx, |now, cx| {
            now.speed = to;
            cx.notify();
        })
    })
    .on_captions(move |on, _, cx| {
        captions.update(cx, |now, cx| {
            now.captions = on;
            cx.notify();
        })
    })
    .on_picture(|_, _| log::info!("gallery: picture in picture"))
    .on_fullscreen(|window, _| window.toggle_fullscreen());
    section(
        "VideoPlayer",
        "A video the host plays, a frame at a time. Its controls fade once the pointer rests while it plays. Space plays, J and L jump ten seconds, C turns captions, F fills the screen.",
        cx,
    )
    .child(probe("media-player", div().w(px(640.)).child(player)))
}

pub fn scrubber(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let at = state.read(cx).at;
    let seek = state.clone();
    section(
        "VideoTimeline / Scrubber",
        "Where a video stands: what has loaded, how far it has played, its chapters, and the time under the pointer. A press or a drag seeks; with focus the arrows step five seconds.",
        cx,
    )
    .child(probe(
        "media-scrubber",
        div().w(px(640.)).pt_6().child(
            Scrubber::new("media-scrubber", LENGTH, at)
                .loaded(LENGTH.mul_f32(0.8))
                .chapter(Duration::ZERO, "Morning")
                .chapter(Duration::from_secs(30), "The stair")
                .chapter(Duration::from_secs(60), "The hall")
                .on_seek(move |at, _, cx| seek.update(cx, |now, cx| {
                    now.at = at;
                    cx.notify();
                })),
        ),
    ))
}

pub fn strip(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let (seek, cut) = (state.clone(), state.clone());
    let frames = (0..8).map(|ix| picture(FRAMES[ix * 3 / 8]));
    section(
        "VideoThumbnailStrip",
        "Frames along the video under its playhead, and the part kept between two handles. A press seeks; a handle trims; I and O set the ends at the playhead.",
        cx,
    )
    .child(probe(
        "media-strip",
        div().w(px(640.)).child(
            VideoThumbnailStrip::new("media-strip", frames, LENGTH, now.at)
                .trim(now.trim.0, now.trim.1)
                .on_seek(move |at, _, cx| seek.update(cx, |now, cx| {
                    now.at = at;
                    cx.notify();
                }))
                .on_trim(move |trim, _, cx| cut.update(cx, |now, cx| {
                    now.trim = trim;
                    cx.notify();
                })),
        ),
    ))
}

pub fn subtitles(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let (change, seek) = (state.clone(), state.clone());
    section(
        "SubtitleEditor",
        "Captions to edit in place, their times as mm:ss.mmm over their words. The one under the playhead is lit; Add puts a new cue there.",
        cx,
    )
    .child(probe(
        "media-subtitles",
        div().w(px(480.)).child(
            SubtitleEditor::new("media-subtitles", now.cues, now.at)
                .on_change(move |cues, _, cx| {
                    let mut next = change.read(cx).clone();
                    next.cues = cues.to_vec();
                    set(&change, next, cx)
                })
                .on_seek(move |at, _, cx| seek.update(cx, |now, cx| {
                    now.at = at;
                    cx.notify();
                })),
        ),
    ))
}
