use std::time::Duration;

use ely_gpui_component::generative::{
    AudioGenerationPlayer, Outcome, Shot, TTSVoicePicker, VideoGenerationTimeline, Voice,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// A waveform for the demo: a swell and a fall, with a beat.
fn peaks() -> Vec<f32> {
    (0..56)
        .map(|ix| {
            let t = ix as f32 / 55.0;
            let swell = (t * std::f32::consts::PI).sin();
            let beat = 0.6 + 0.4 * ((ix * 7 % 11) as f32 / 10.0);
            (0.15 + 0.8 * swell * beat).min(1.0)
        })
        .collect()
}

pub fn audio(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let player = keep("gen-audio", || (Duration::from_secs(12), false), window, cx);
    let (played, playing) = *player.read(cx);
    let (toggled, sought) = (player.clone(), player);
    let length = Duration::from_secs(32);
    section(
        "AudioGenerationPlayer",
        "A generated sound: its prompt, model and length, a bar while it is made, then a player whose waveform lights as it plays. A press on the waveform seeks, and so do Left and Right once it has focus.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_4()
            .child(
                probe(
                    "gen-audio",
                    div().w(px(440.)).child(
                        AudioGenerationPlayer::new("gen-audio", "rain on a limestone courtyard, far thunder, a slow piano", "Orchid Audio", peaks(), length)
                            .played(played, playing)
                            .on_toggle(move |play, _, cx| {
                                let at = toggled.read(cx).0;
                                set(&toggled, (at, play), cx)
                            })
                            .on_seek(move |share, _, cx| {
                                let playing = sought.read(cx).1;
                                set(&sought, (length.mul_f32(share), playing), cx)
                            })
                            .on_regenerate(|_, _| log::info!("gallery: regenerate"))
                            .on_download(|_, _| log::info!("gallery: download")),
                    ),
                ),
            )
            .child(
                div().w(px(440.)).child(
                    AudioGenerationPlayer::new("gen-audio-making", "a cello under a high window, morning", "Orchid Audio", peaks(), Duration::from_secs(45))
                        .making(0.45)
                        .on_regenerate(|_, _| log::info!("gallery: regenerate"))
                        .on_download(|_, _| log::info!("gallery: download")),
                ),
            ),
    )
}

fn voices() -> Vec<Voice> {
    let voice = |key: &str, name: &str, about: &str, tags: &[&str]| Voice {
        key: key.to_string().into(),
        name: name.to_string().into(),
        about: about.to_string().into(),
        tags: tags
            .iter()
            .map(|tag| SharedString::from(tag.to_string()))
            .collect(),
    };
    vec![
        voice(
            "iris",
            "Iris",
            "Warm and unhurried, for long reads",
            &["English", "US"],
        ),
        voice(
            "theo",
            "Theo",
            "Bright and quick, for product tours",
            &["English", "UK"],
        ),
        voice(
            "mei",
            "Mei",
            "Calm, with clear diction",
            &["Mandarin", "English"],
        ),
        voice(
            "ines",
            "Inés",
            "Soft, close to the microphone",
            &["Spanish"],
        ),
    ]
}

pub fn voice(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep("gen-voice", || SharedString::from("iris"), window, cx);
    let sample = keep("gen-voice-sample", || None::<SharedString>, window, cx);
    let (now, sounding) = (chosen.read(cx).clone(), sample.read(cx).clone());
    let picker = TTSVoicePicker::new("gen-voices", voices())
        .selected(now)
        .on_select(move |key, _, cx| set(&chosen, key.clone(), cx))
        .on_play(move |key, _, cx| set(&sample, key, cx));
    let picker = match sounding {
        Some(key) => picker.playing(key),
        None => picker,
    };
    section(
        "TTSVoicePicker",
        "Voices to speak with: play a sample, then choose one by its row. The chosen voice takes the active wash and a check; tags drop below the name when narrow.",
        cx,
    )
    .child(probe("gen-voices", div().w(px(460.)).child(picker)))
}

fn shots() -> Vec<Shot> {
    let shot = |key: &str, prompt: &str, seconds: u64, outcome| Shot {
        key: key.to_string().into(),
        prompt: prompt.to_string().into(),
        length: Duration::from_secs(seconds),
        outcome,
    };
    vec![
        shot(
            "s1",
            "a slow pan across a white atrium",
            5,
            Outcome::Done(asset!("atrium.jpg").into()),
        ),
        shot(
            "s2",
            "light moves over the stair",
            4,
            Outcome::Done(asset!("atrium-stair.jpg").into()),
        ),
        shot("s3", "dusk over the dunes", 6, Outcome::Pending(Some(0.35))),
        shot(
            "s4",
            "grass bends in the wind",
            4,
            Outcome::Failed("timed out".into()),
        ),
    ]
}

pub fn video(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let list = keep("gen-shots", shots, window, cx);
    let chosen = keep("gen-shot", || SharedString::from("s1"), window, cx);
    let (now, pick) = (list.read(cx).clone(), chosen.read(cx).clone());
    section(
        "VideoGenerationTimeline",
        "A video made shot by shot along its time: each shot as wide as it runs, its poster once made, a placeholder while it comes. Choose a shot to read it; Extend asks for the next.",
        cx,
    )
    .child(
        probe(
            "gen-timeline",
            div().w(px(560.)).child(
                VideoGenerationTimeline::new("gen-timeline", now, Duration::from_millis(3_200))
                    .selected(pick)
                    .on_select(move |key, _, cx| set(&chosen, key.clone(), cx))
                    .on_extend(move |_, cx| {
                        let mut next = list.read(cx).clone();
                        let key = format!("s{}", next.len() + 1);
                        next.push(Shot {
                            key: key.into(),
                            prompt: "the camera rises over the dunes".into(),
                            length: Duration::from_secs(5),
                            outcome: Outcome::Pending(Some(0.0)),
                        });
                        set(&list, next, cx)
                    }),
            ),
        ),
    )
}
