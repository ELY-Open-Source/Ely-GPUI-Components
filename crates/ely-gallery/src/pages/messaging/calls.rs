use ely_gpui_component::{
    buttons::Button,
    data_display::Avatar,
    messaging::{
        CallControls, HuddleIndicator, IncomingCallDialog, ParticipantTile, ScreenShareView,
        VideoCallGrid, VoiceCallBar,
    },
    theme::{ActiveTheme, TextSize},
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use web_time::Instant;

use crate::{
    probe::probe,
    ui::{change, keep, picture, section},
};

const ROOM: &str = asset!("atrium-olive.jpg");
const STAIR: &str = asset!("atrium-stair.jpg");
const SCREEN: &str = asset!("frames/screen.jpg");
/// The demo cameras' frames are the atrium pictures, 3:2.
const CAMERA: f32 = 1.5;

/// The demo call: your microphone, camera and share, and whether you are in it and since when.
#[derive(Clone)]
struct Call {
    mic: bool,
    camera: bool,
    share: bool,
    joined: Option<Instant>,
}

fn call(window: &mut Window, cx: &mut App) -> Entity<Call> {
    keep(
        "messaging-call",
        || Call {
            mic: true,
            camera: true,
            share: false,
            joined: Some(Instant::now()),
        },
        window,
        cx,
    )
}

fn huddlers() -> [Avatar; 3] {
    [
        Avatar::new("messaging-huddler-ana", "Ana Lima"),
        Avatar::new("messaging-huddler-ben", "Ben Ito"),
        Avatar::new("messaging-huddler-chloe", "Chloé Martin"),
    ]
}

pub fn controls(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = call(window, cx);
    let now = state.read(cx).clone();
    let join = state.clone();
    let huddle = HuddleIndicator::new("messaging-huddle", huddlers()).on_join(move |_, cx| {
        change(&join, cx, |call| {
            call.joined.get_or_insert_with(Instant::now);
        })
    });
    let body = match now.joined {
        Some(since) => {
            let (mic, bar_mic, camera, share, leave, bar_leave) = (
                state.clone(),
                state.clone(),
                state.clone(),
                state.clone(),
                state.clone(),
                state,
            );
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    VoiceCallBar::new("messaging-voice", "Huddle in # design", since)
                        .people(huddlers())
                        .mic(now.mic, move |on, _, cx| {
                            change(&bar_mic, cx, |call| call.mic = on)
                        })
                        .on_leave(move |_, cx| change(&bar_leave, cx, |call| call.joined = None)),
                )
                .child(
                    div().flex().child(
                        CallControls::new("messaging-controls")
                            .mic(now.mic, move |on, _, cx| {
                                change(&mic, cx, |call| call.mic = on)
                            })
                            .camera(now.camera, move |on, _, cx| {
                                change(&camera, cx, |call| call.camera = on)
                            })
                            .share(now.share, move |on, _, cx| {
                                change(&share, cx, |call| call.share = on)
                            })
                            .on_leave(move |_, cx| change(&leave, cx, |call| call.joined = None)),
                    ),
                )
        }
        None => div()
            .text_size(cx.theme().text_size(TextSize::Sm))
            .child("You left the huddle. Join it again from its mark."),
    };
    section(
        "CallControls / VoiceCallBar / HuddleIndicator",
        "A huddle's mark rings while it is live; a press joins. In the call, a bar tells where it is, how long it has run and who is in it, and the controls turn the microphone, camera and a shared screen on and off, or leave.",
        cx,
    )
    .child(probe(
        "messaging-call",
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(div().flex().child(huddle))
            .child(body),
    ))
}

pub fn grid(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let now = call(window, cx).read(cx).clone();
    let tile = |key: &'static str, name: &'static str| ParticipantTile::new(key, name, CAMERA);
    let you = tile("messaging-tile-you", "You").muted(!now.mic);
    let you = match now.camera {
        true => you.frame(picture(ROOM)),
        false => you,
    };
    section(
        "ParticipantTile / VideoCallGrid",
        "Everyone in the call as tiles in even rows, a short last row centered: a camera's frame or the person's initials, their name with a mark when muted, and a ring while they speak. Your tile follows the controls above.",
        cx,
    )
    .child(probe(
        "messaging-grid",
        div().w(px(640.)).child(
            VideoCallGrid::new("messaging-grid")
                .child(tile("messaging-tile-ana", "Ana Lima").frame(picture(STAIR)).speaking(true))
                .child(tile("messaging-tile-ben", "Ben Ito").muted(true))
                .child(you)
                .child(tile("messaging-tile-chloe", "Chloé Martin"))
                .child(tile("messaging-tile-dev", "Dev Rao").muted(true)),
        ),
    ))
}

pub fn share(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = call(window, cx);
    let sharing = state.read(cx).share;
    let tile = |key: &'static str, name: &'static str| ParticipantTile::new(key, name, CAMERA);
    let view = ScreenShareView::new("messaging-share", "Ana Lima", 16.0 / 10.0)
        .frame(picture(SCREEN))
        .child(tile("messaging-strip-ben", "Ben Ito").muted(true))
        .child(tile("messaging-strip-chloe", "Chloé Martin").speaking(true));
    let view = match sharing {
        true => view.on_stop(move |_, cx| change(&state, cx, |call| call.share = false)),
        false => view,
    };
    section(
        "ScreenShareView",
        "A shared screen in its shape under a bar that names who presents, the call's tiles beside it. Share your screen from the controls above, and the bar offers Stop presenting.",
        cx,
    )
    .child(probe("messaging-share", div().w(px(760.)).child(view)))
}

pub fn incoming(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let ringing = keep("messaging-ringing", || false, window, cx);
    let answer = keep("messaging-answer", || None::<SharedString>, window, cx);
    let rings = *ringing.read(cx);
    let said = answer.read(cx).clone();
    let ring = ringing.clone();
    let (accepted, declined) = ((ringing.clone(), answer.clone()), (ringing, answer));
    let dialog = rings.then(|| {
        IncomingCallDialog::new(
            "messaging-incoming",
            "Ana Lima",
            move |_, cx| {
                change(&accepted.0, cx, |ringing| *ringing = false);
                change(&accepted.1, cx, |answer| {
                    *answer = Some("You accepted Ana's call.".into())
                });
            },
            move |_, cx| {
                change(&declined.0, cx, |ringing| *ringing = false);
                change(&declined.1, cx, |answer| {
                    *answer = Some("You declined Ana's call.".into())
                });
            },
        )
        .video()
    });
    let theme = cx.theme();
    section(
        "IncomingCallDialog",
        "A call coming in: the caller ringing, their name and the kind of call, with Decline and Accept. Escape declines.",
        cx,
    )
    .child(probe(
        "messaging-incoming",
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(
                Button::new("messaging-ring", "Ring")
                    .on_click(move |_, _, cx| change(&ring, cx, |ringing| *ringing = true)),
            )
            .children(said.map(|said| {
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(said)
            })),
    ))
    .children(dialog)
}
