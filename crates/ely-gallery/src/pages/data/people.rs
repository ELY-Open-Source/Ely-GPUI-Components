use std::path::PathBuf;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    data_display::{
        Avatar, AvatarGroup, Badge, CountBadge, DotBadge, KpiCard, Presence, Statistic, Tag, Tone,
        TrendIndicator, UserChip,
    },
    forms::AvatarUpload,
    primitives::{Icon, IconName, Severity},
    theme::{ActiveTheme, AvatarSize, IconSize, TextSize},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div};

use crate::{
    probe::probe,
    ui::{keep, picture, row, section, set, specimen, specimens},
};

const DUNES: &str = asset!("dunes-square.jpg");

pub fn badges(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let unread = keep("badge-unread", || 3usize, window, cx);
    let (count, more) = (*unread.read(cx), unread.clone());
    let muted = cx.theme().colors.fg_muted;
    let bell = || Icon::new(IconName::Bell).size(IconSize::Lg).color(muted);
    section(
        "Badge / CountBadge / DotBadge",
        "Small labels tinted by what they mean, counts whose digits roll, and a dot for something new.",
        cx,
    )
    .child(
        row()
            .child(Badge::new("Draft"))
            .child(Badge::new("Beta").tone(Tone::Accent))
            .child(Badge::new("Live").tone(Severity::Success).dot())
            .child(Badge::new("Review").tone(Severity::Warning).dot())
            .child(Badge::new("Failed").tone(Severity::Danger).dot())
            .child(Badge::new("New").tone(Severity::Info)),
    )
    .child(
        specimens()
            .child(specimen("count", CountBadge::new("count-bell", count).over(bell()), cx))
            .child(specimen("capped", CountBadge::new("count-inbox", 128).over(Icon::new(IconName::Inbox).size(IconSize::Lg).color(muted)), cx))
            .child(specimen("dot", DotBadge::new().over(bell()), cx))
            .child(specimen("alone", CountBadge::new("count-alone", 7), cx)),
    )
    .child(row().child(probe(
        "badge-more",
        Button::new("badge-more", "One more")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| set(&more, count + 1, cx)),
    )))
}

pub fn tags(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const START: [&str; 4] = ["Design", "Motion", "Accessibility", "Rust"];
    let kept = keep("tags-kept", || START.to_vec(), window, cx);
    let now = kept.read(cx).clone();
    let reset = kept.clone();
    section(
        "Tag / Chip",
        "Short labels for a kind, a topic or a filter; an x takes one away.",
        cx,
    )
    .child(
        row()
            .child(Tag::new("tag-plain", "Research"))
            .child(Tag::new("tag-icon", "Pinned").icon(IconName::Pin))
            .child(
                Tag::new("tag-tone", "Shipped")
                    .tone(Severity::Success)
                    .icon(IconName::Check),
            ),
    )
    .child(
        row()
            .children(now.iter().enumerate().map(|(ix, name)| {
                let kept = kept.clone();
                let name = *name;
                Tag::new(SharedString::from(format!("tag-remove-{ix}")), name).on_remove(
                    move |_, cx| {
                        let then: Vec<&str> = kept
                            .read(cx)
                            .iter()
                            .copied()
                            .filter(|tag| *tag != name)
                            .collect();
                        set(&kept, then, cx)
                    },
                )
            }))
            .child(
                Button::new("tags-reset", "Reset")
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, _, cx| set(&reset, START.to_vec(), cx)),
            ),
    )
}

pub fn avatars(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep("avatar-picked", || None::<PathBuf>, window, cx);
    let (chosen, take) = (picked.read(cx).clone(), picked.clone());
    let sizes = [
        (AvatarSize::Xs, "xs"),
        (AvatarSize::Sm, "sm"),
        (AvatarSize::Md, "md"),
        (AvatarSize::Lg, "lg"),
        (AvatarSize::Xl, "xl"),
    ];
    let upload = Avatar::new("upload-face", "Noor Haddad").size(AvatarSize::Xl);
    let upload = match chosen.clone() {
        Some(path) => upload.image(path),
        None => upload,
    };
    section(
        "Avatar / AvatarUpload",
        "A picture, or initials on a tone the name keeps, or an icon. Presence sits on the corner. Hover the last one, then press it to choose a picture, or drop one on it.",
        cx,
    )
    .child(specimens().children(sizes.map(|(size, name)| {
        specimen(name, Avatar::new(SharedString::from(format!("avatar-{name}")), "Kofi Mensah").size(size), cx)
    })))
    .child(
        specimens()
            .child(specimen("picture", Avatar::new("avatar-photo", "Dunes").image(picture(DUNES)).size(AvatarSize::Lg), cx))
            .child(specimen("online", Avatar::new("avatar-online", "Aiko Tanaka").size(AvatarSize::Lg).presence(Presence::Online), cx))
            .child(specimen("away", Avatar::new("avatar-away", "Lucía Romero").size(AvatarSize::Lg).presence(Presence::Away), cx))
            .child(specimen("busy", Avatar::new("avatar-busy", "Ben Carter").size(AvatarSize::Lg).presence(Presence::Busy), cx))
            .child(specimen("team", Avatar::new("avatar-team", "Studio North").size(AvatarSize::Lg).square(), cx))
            .child(specimen("bot", Avatar::new("avatar-bot", "Build bot").icon(IconName::Bot).size(AvatarSize::Lg), cx))
            .child(specimen(
                "upload",
                probe("avatar-upload", AvatarUpload::new("avatar-upload", upload).on_change(move |path, _, cx| set(&take, Some(path), cx))),
                cx,
            )),
    )
    .child(Caption::new(match chosen {
        Some(path) => format!("Chose {}", path.display()),
        None => "No picture chosen yet.".to_string(),
    }))
}

pub fn groups(cx: &mut App) -> impl IntoElement + use<> {
    const PEOPLE: [&str; 7] = [
        "Ada Lovelace",
        "Alan Turing",
        "Grace Hopper",
        "Katherine Johnson",
        "Tim Berners-Lee",
        "Radia Perlman",
        "Sophie Wilson",
    ];
    let theme = cx.theme();
    let people = |prefix: &'static str| {
        PEOPLE
            .iter()
            .enumerate()
            .map(move |(ix, name)| Avatar::new(SharedString::from(format!("{prefix}-{ix}")), *name))
    };
    section(
        "AvatarGroup / UserChip",
        "People side by side, each tucked under the next, with a count of the rest; someone named inline.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("four of seven", AvatarGroup::new(people("group")), cx))
            .child(specimen("small", AvatarGroup::new(people("group-sm")).max(5).size(AvatarSize::Sm), cx)),
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1p5()
            .text_size(theme.text_size(TextSize::Base))
            .child("Assigned to")
            .child(UserChip::new("chip-grace", "Grace Hopper"))
            .child("and")
            .child(UserChip::new("chip-dunes", "Dune Studio").image(picture(DUNES)))
            .child("for review."),
    )
}

pub fn numbers(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turn = keep("stats-turn", || 0usize, window, cx);
    let (now, next) = (*turn.read(cx), turn.clone());
    let revenue = [48_210.0, 51_380.0, 49_905.0][now % 3];
    let users = [3_812.0, 3_994.0, 4_120.0][now % 3];
    let latency = [212.0, 198.0, 187.0][now % 3];
    section(
        "Statistic / KPI Card / Metric / TrendIndicator",
        "Figures that matter, rolling to each new value, with how they moved. For latency, down is the good way.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_3()
            .child(
                div().flex_1().child(
                    KpiCard::new(Statistic::new("kpi-revenue", "Revenue", revenue).prefix("$").trend(TrendIndicator::new(0.124)))
                        .icon(IconName::DollarSign)
                        .caption("This month, against the last"),
                ),
            )
            .child(
                div().flex_1().child(
                    KpiCard::new(Statistic::new("kpi-users", "Active people", users).trend(TrendIndicator::new(0.031)))
                        .icon(IconName::Users)
                        .caption("Seen in the last seven days"),
                ),
            )
            .child(
                div().flex_1().child(
                    KpiCard::new(
                        Statistic::new("kpi-latency", "p95 latency", latency)
                            .suffix("ms")
                            .trend(TrendIndicator::new(-0.08).down_is_good()),
                    )
                    .icon(IconName::Gauge)
                    .caption("Across all regions"),
                ),
            ),
    )
    .child(
        specimens()
            .child(specimen("metric", Statistic::new("metric-uptime", "Uptime", 99.98).decimals(2).suffix("%").size(TextSize::Lg), cx))
            .child(specimen("metric", Statistic::new("metric-queue", "In queue", 14.0).size(TextSize::Lg), cx))
            .child(specimen("flat", TrendIndicator::new(0.0), cx)),
    )
    .child(row().child(probe(
        "stats-next",
        Button::new("stats-next", "Next week")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| set(&next, now + 1, cx)),
    )))
}
