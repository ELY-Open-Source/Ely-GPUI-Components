use std::time::Duration;

use ely_gpui_component::media::{
    AudioPlayer, AudioSpectrum, AudioWaveform, MediaControls, PlaybackSpeedControl, Playlist,
    Repeat, Track, VolumeControl,
};
use gpui::{
    App, Entity, IntoElement, ParentElement, SharedString, Styled, TaskExt as _, Window, div, px,
};

use crate::{
    probe::probe,
    ui::{change, keep, noise, picture, section},
};

const COVER: &str = asset!("dunes-square.jpg");
/// How often the demo's clock moves while a song plays.
const TICK: Duration = Duration::from_millis(100);
const SONGS: [(&str, &str, &str, u64); 5] = [
    ("tall-windows", "Tall Windows", "Aster Quartet", 222),
    ("stone-stair", "Stone Stair", "Aster Quartet", 245),
    ("olive-court", "Olive Court", "Lumen Trio", 178),
    ("dune-hours", "Dune Hours", "Lumen Trio", 311),
    ("low-sun", "Low Sun", "Marram", 207),
];

/// The demo's listening: the song under way and where, how it plays, and its volume.
#[derive(Clone)]
struct Listening {
    song: usize,
    at: Duration,
    playing: bool,
    ticking: bool,
    speed: f32,
    shuffle: bool,
    repeat: Repeat,
    level: f32,
    muted: bool,
}

impl Listening {
    fn length(&self) -> Duration {
        Duration::from_secs(SONGS[self.song].3)
    }

    /// Moves `by` songs along the list, from the start of the song.
    fn skip(&mut self, by: isize) {
        self.song = (self.song as isize + by).rem_euclid(SONGS.len() as isize) as usize;
        self.at = Duration::ZERO;
    }
}

fn tracks() -> Vec<Track> {
    SONGS
        .iter()
        .map(|(key, title, artist, seconds)| Track {
            key: SharedString::from(*key),
            title: SharedString::from(*title),
            artist: SharedString::from(*artist),
            length: Duration::from_secs(*seconds),
        })
        .collect()
}

/// A song's peaks, the same each run: sections that swell and fall, faded at both ends.
fn peaks(song: usize) -> Vec<f32> {
    let mut next = noise(song as u64 + 7);
    let count = 180;
    (0..count)
        .map(|ix| {
            let fade = (ix.min(count - 1 - ix) as f32 / 10.0).min(1.0);
            let swell = 0.55 + 0.45 * (ix as f32 / 23.0 + song as f32).sin();
            (fade * swell * (0.35 + 0.65 * next() as f32)).clamp(0.0, 1.0)
        })
        .collect()
}

/// Levels by pitch at a moment of the song: lows louder, each band beating at its own rate.
fn levels(at: Duration) -> Vec<f32> {
    let time = at.as_secs_f32();
    (0..40)
        .map(|band| {
            let band = band as f32;
            let beat = (time * 2.3 + band * 0.7).sin() * (time * 0.9 + band * 0.31).cos();
            ((1.0 - band / 52.0) * (0.3 + 0.35 * (1.0 + beat))).clamp(0.0, 1.0)
        })
        .collect()
}

/// Runs the demo's clock while a song plays, one loop at a time; at the end the next song starts, or the same one on repeat.
fn tick(state: &Entity<Listening>, window: &mut Window, cx: &mut App) {
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
                        now.at = (now.at + TICK.mul_f32(now.speed)).min(now.length());
                        if now.at == now.length() {
                            match now.repeat {
                                Repeat::One => now.at = Duration::ZERO,
                                Repeat::All => now.skip(1),
                                Repeat::Off if now.song + 1 < SONGS.len() => now.skip(1),
                                Repeat::Off => now.playing = false,
                            }
                        }
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

fn state(window: &mut Window, cx: &mut App) -> Entity<Listening> {
    keep(
        "media-audio",
        || Listening {
            song: 0,
            at: Duration::from_secs(64),
            playing: false,
            ticking: false,
            speed: 1.0,
            shuffle: false,
            repeat: Repeat::All,
            level: 0.7,
            muted: false,
        },
        window,
        cx,
    )
}

fn controls(id: &'static str, state: &Entity<Listening>, now: &Listening) -> MediaControls {
    let (play, back, forward, shuffle, repeat) = (
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
    );
    MediaControls::new(id, now.playing)
        .on_play(move |on, window, cx| {
            change(&play, cx, |now| now.playing = on);
            if on {
                tick(&play, window, cx);
            }
        })
        .on_back(move |_, cx| change(&back, cx, |now| now.skip(-1)))
        .on_forward(move |_, cx| change(&forward, cx, |now| now.skip(1)))
        .shuffle(now.shuffle, move |on, _, cx| {
            change(&shuffle, cx, |now| now.shuffle = on)
        })
        .repeat(now.repeat, move |next, _, cx| {
            change(&repeat, cx, |now| now.repeat = next)
        })
}

fn volume(id: &'static str, state: &Entity<Listening>, now: &Listening) -> VolumeControl {
    let (level, mute) = (state.clone(), state.clone());
    VolumeControl::new(id, now.level, now.muted)
        .on_level(move |to, _, cx| change(&level, cx, |now| now.level = to as f32))
        .on_mute(move |on, _, cx| change(&mute, cx, |now| now.muted = on))
}

pub fn player(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let (title, artist) = (SONGS[now.song].1, SONGS[now.song].2);
    let seek = state.clone();
    let player = AudioPlayer::new("media-audio", title, artist, now.length(), now.at)
        .artwork(picture(COVER))
        .controls(controls("media-audio-transport", &state, &now))
        .volume(volume("media-audio-volume", &state, &now))
        .on_seek(move |at, _, cx| change(&seek, cx, |now| now.at = at));
    section(
        "AudioPlayer",
        "A song the host plays: its cover, the time played and left along a scrubber, the transport and the volume.",
        cx,
    )
    .child(probe("media-audio", div().w(px(360.)).child(player)))
}

pub fn waveform(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let seek = state.clone();
    section(
        "AudioWaveform",
        "A sound's peaks, dark as far as it has played. A press or a drag seeks, the time under the pointer shows above it, and with focus the arrows step five seconds.",
        cx,
    )
    .child(probe(
        "media-waveform",
        div().w(px(640.)).pt_6().child(
            AudioWaveform::new("media-waveform", peaks(now.song), now.length(), now.at)
                .on_seek(move |at, _, cx| change(&seek, cx, |now| now.at = at)),
        ),
    ))
}

pub fn spectrum(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let at = state(window, cx).read(cx).at;
    section(
        "AudioSpectrum / Visualizer",
        "Sound by pitch, low to high. The host sends new levels as it plays; they hold while it pauses.",
        cx,
    )
    .child(probe(
        "media-spectrum",
        div().w(px(360.)).child(AudioSpectrum::new(levels(at))),
    ))
}

pub fn volume_control(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    section(
        "VolumeControl",
        "A speaker that mutes, drawn by the level, beside a slider. Muted, the slider rests at zero; moving it unmutes.",
        cx,
    )
    .child(probe(
        "media-volume",
        div().w(px(240.)).child(volume("media-volume", &state, &now)),
    ))
}

pub fn speed(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).speed;
    let pace = state.clone();
    section(
        "PlaybackSpeedControl",
        "How fast it plays, from half to double, chosen from a menu.",
        cx,
    )
    .child(probe(
        "media-speed",
        div().child(
            PlaybackSpeedControl::new("media-speed", now)
                .on_change(move |to, _, cx| change(&pace, cx, |now| now.speed = to)),
        ),
    ))
}

pub fn transport(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    section(
        "MediaControls",
        "Shuffle, back, play, forward and repeat. Shuffle and repeat stay lit while on; repeat steps from all to one to off.",
        cx,
    )
    .child(probe(
        "media-transport",
        div().w(px(360.)).child(controls("media-transport", &state, &now)),
    ))
}

pub fn playlist(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = state(window, cx);
    let now = state.read(cx).clone();
    let pick = state.clone();
    section(
        "Playlist",
        "Songs in order. The one under way is lit, its bars moving while it plays; Enter or a double press plays another.",
        cx,
    )
    .child(probe(
        "media-playlist",
        div().w(px(420.)).child(
            Playlist::new("media-playlist", tracks())
                .current(SONGS[now.song].0, now.playing)
                .on_play(move |key, window, cx| {
                    let song = SONGS
                        .iter()
                        .position(|(each, ..)| *each == key.as_ref())
                        .expect("a listed song");
                    change(&pick, cx, |now| {
                        (now.song, now.at, now.playing) = (song, Duration::ZERO, true)
                    });
                    tick(&pick, window, cx);
                }),
        ),
    ))
}
