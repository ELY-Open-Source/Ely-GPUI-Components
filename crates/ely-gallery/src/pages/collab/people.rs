use ely_gpui_component::{
    collab::{FollowMode, LiveIndicator, Peer, PresenceAvatars, RemoteCursor, RemoteSelection},
    forms::{Input, TextInput},
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    App, Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// The people in the demo document, each somewhere in it.
pub fn peers() -> Vec<Peer> {
    [
        ("ada", "Ada Park", "Reading “In dark”"),
        ("grace", "Grace Lin", "Writing the opening"),
        ("alan", "Alan Reyes", "In the comments"),
        ("mae", "Mae Okafor", "Viewing"),
        ("tim", "Tim Novak", "Viewing"),
        ("joan", "Joan Ito", "Viewing"),
    ]
    .into_iter()
    .enumerate()
    .map(|(hue, (key, name, place))| Peer {
        place: Some(place.into()),
        ..Peer::new(key, name, hue)
    })
    .collect()
}

/// The one reading the demo.
pub fn you() -> Peer {
    Peer::new("you", "You", 6)
}

const DRAFT: &str = "A lift blends a color toward white.\nHalf a lift sits midway; a full lift is white itself.\nIn dark, accents lift further so they read with the same weight.";

fn draft(window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    window.use_keyed_state("collab-draft", cx, |window, cx| {
        let mut input = TextInput::new(window, cx).multi_line(3, 6);
        input.set_text(DRAFT, cx);
        input
    })
}

pub fn presence(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let following = keep("collab-following", || None::<SharedString>, window, cx);
    let now = following.read(cx).clone();
    let people = peers();
    let followed = now
        .as_ref()
        .and_then(|key| people.iter().find(|peer| &peer.key == key).cloned());
    let draft = draft(window, cx);
    let selected = DRAFT
        .find("a full lift")
        .expect("the draft names a full lift");
    let caret = DRAFT.find("accents").expect("the draft names accents");
    let theme = cx.theme();
    let (toggle, stop) = (following.clone(), following);
    section(
        "PresenceAvatars / LiveIndicator / FollowMode / RemoteCursor / RemoteSelection",
        "Who else is here, ringed in their colors, their carets and selections live in the text. Rest on someone to see where they are; press to follow their view, and Escape to stop.",
        cx,
    )
    .child(
        div()
            .w(px(760.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .text_size(theme.text_size(TextSize::Sm))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Handbook / Color / Lift"),
                    )
                    .child(LiveIndicator::new("collab-live", people.len()))
                    .child(probe(
                        "collab-peers",
                        PresenceAvatars::new("collab-peers", people.clone()).follow(
                            now,
                            move |key, _, cx| {
                                let next =
                                    (toggle.read(cx).as_ref() != Some(key)).then(|| key.clone());
                                set(&toggle, next, cx)
                            },
                        ),
                    )),
            )
            .child(
                FollowMode::new("collab-follow", followed, move |_, cx| set(&stop, None, cx))
                    .child(
                        div().p_2().child(
                            div()
                                .relative()
                                .child(Input::new(&draft))
                                .child(RemoteSelection::new(
                                    &people[2],
                                    &draft,
                                    selected..selected + "a full lift".len(),
                                ))
                                .child(RemoteCursor::new(&people[1], &draft, caret)),
                        ),
                    ),
            ),
    )
}
