use ely_gpui_component::{
    data_display::Presence,
    messaging::{
        ClearAfter, Member, MemberList, OnlineStatus, Status, StatusSetter, UserProfileCard,
    },
    theme::{ActiveTheme, Radius, TextSize},
};
use gpui::{App, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::tz::TimeZone;

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// A member's card: their title and the zone they work in.
struct Person {
    member: Member,
    title: &'static str,
    zone: &'static str,
}

fn people() -> Vec<Person> {
    let person = |member: Member, title, zone| Person {
        member,
        title,
        zone,
    };
    vec![
        person(
            Member::new("ana", "Ana Lima", Presence::Away).status("🌴 Back on Monday"),
            "Architect",
            "Europe/Lisbon",
        ),
        person(
            Member::new("ben", "Ben Ito", Presence::Online).role("Admin"),
            "Structural engineer",
            "Asia/Tokyo",
        ),
        person(
            Member::new("chloe", "Chloé Martin", Presence::Busy).status("🎧 Drawing the stair"),
            "Lead designer",
            "Europe/Paris",
        ),
        person(
            Member::new("dev", "Dev Rao", Presence::Offline),
            "Model maker",
            "Asia/Kolkata",
        ),
        person(
            Member::new("eli", "Eli Park", Presence::Online).role("Owner"),
            "Principal",
            "America/New_York",
        ),
        person(
            Member::new("farah", "Farah Haddad", Presence::Offline),
            "Landscape architect",
            "Asia/Dubai",
        ),
    ]
}

pub fn members(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep(
        "messaging-member",
        || SharedString::from("chloe"),
        window,
        cx,
    );
    let key = chosen.read(cx).clone();
    let all = people();
    let shown = all
        .iter()
        .find(|person| person.member.key == key)
        .expect("the chosen member is listed");
    let member = &shown.member;
    let zone = TimeZone::get(shown.zone).expect("a known zone");
    let card = UserProfileCard::new(
        (ElementId::from("messaging-profile"), member.key.to_string()),
        member.name.clone(),
        member.presence,
    )
    .title(shown.title)
    .zone(zone)
    .on_message(|_, _| log::info!("gallery: message"))
    .on_call(|_, _| log::info!("gallery: call"));
    let card = match member.status.clone() {
        Some(status) => card.status(status),
        None => card,
    };
    let theme = cx.theme();
    let frame = div()
        .w(px(280.))
        .p_4()
        .border_1()
        .border_color(theme.colors.border)
        .rounded(theme.radius(Radius::Lg))
        .child(card);
    section(
        "MemberList / UserProfileCard",
        "Members online and offline by name, each group under its count; offline members go quiet. A press or an arrow shows a member's card: their standing, title and status, the time where they are, and ways to reach them.",
        cx,
    )
    .child(probe(
        "messaging-members",
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(300.)).child(
                    MemberList::new(
                        "messaging-members",
                        all.iter().map(|person| person.member.clone()),
                    )
                    .selected(key)
                    .on_select(move |key, _, cx| change(&chosen, cx, |chosen| *chosen = key.clone()))
                    .on_activate(|key, _, _| log::info!("gallery: message {key}")),
                ),
            )
            .child(frame),
    ))
}

pub fn statuses(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "OnlineStatus",
        "Where someone stands, as a dot in its color and its word.",
        cx,
    )
    .child(probe(
        "messaging-online",
        div().flex().flex_wrap().gap_6().children(
            [
                Presence::Online,
                Presence::Away,
                Presence::Busy,
                Presence::Offline,
            ]
            .map(OnlineStatus::new),
        ),
    ))
}

fn suggestion(emoji: &str, text: &str, clear: ClearAfter) -> Status {
    Status {
        emoji: Some(SharedString::from(emoji.to_string())),
        text: SharedString::from(text.to_string()),
        clear,
    }
}

pub fn setter(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep("messaging-status", || None::<Status>, window, cx);
    let status = kept.read(cx).clone();
    let (saving, clearing) = (kept.clone(), kept);
    let theme = cx.theme();
    let now = match &status {
        Some(status) => {
            let words = match &status.emoji {
                Some(emoji) => format!("{emoji} {}", status.text),
                None => status.text.to_string(),
            };
            format!("{words} · {}", status.clear.label())
        }
        None => "No status set.".to_string(),
    };
    let setter = StatusSetter::new("messaging-status")
        .suggestions([
            suggestion("📅", "In a meeting", ClearAfter::OneHour),
            suggestion("🚌", "Commuting", ClearAfter::ThirtyMinutes),
            suggestion("🤒", "Out sick", ClearAfter::Today),
            suggestion("🌴", "On leave", ClearAfter::Never),
        ])
        .on_save(move |status, _, cx| {
            let status = status.clone();
            change(&saving, cx, |kept| *kept = Some(status))
        })
        .on_clear(move |_, cx| change(&clearing, cx, |kept| *kept = None));
    let setter = match status {
        Some(status) => setter.status(status),
        None => setter,
    };
    let shown = div()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(now);
    section(
        "StatusSetter",
        "An emoji, a few words, and when they clear. A suggestion fills all three; Save sets the status, and Clear status removes it.",
        cx,
    )
    .child(probe(
        "messaging-status",
        div()
            .w(px(380.))
            .flex()
            .flex_col()
            .gap_4()
            .child(shown)
            .child(setter),
    ))
}
