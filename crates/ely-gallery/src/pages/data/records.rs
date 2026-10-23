use ely_gpui_component::{
    buttons::{Button, ButtonVariant, SegmentedControl},
    data_display::{
        Activity, ActivityFeed, Badge, Change, Changelog, DescriptionList, PropertyGrid,
        PropertyGroup, Release, Timeline, TimelineItem, UserChip,
    },
    forms::{NumberInput, Slider, Switch},
    primitives::{IconName, Severity},
    theme::{ActiveTheme, ControlSize, Radius},
    typography::Code,
};
use gpui::{
    App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, prelude::*, px,
};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn descriptions(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "DescriptionList / KeyValueList / InfoRow",
        "Labels and their values: in a quiet column beside them, or stacked where room is narrow.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_12()
            .child(
                div().flex_1().child(
                    DescriptionList::new()
                        .lined()
                        .item("Status", Badge::new("Paid").tone(Severity::Success).dot())
                        .item(
                            "Customer",
                            UserChip::new("desc-customer", "Katherine Johnson"),
                        )
                        .item("Amount", "$1,280.00")
                        .item("Placed", "Sep 24, 2026 · 14:32")
                        .item("Reference", Code::new("ORD-2419")),
                ),
            )
            .child(
                div().w_64().min_w_0().child(
                    DescriptionList::new()
                        .stacked()
                        .item("Ship to", "12 Harbour Street, Wellington 6011, New Zealand")
                        .item("Delivery", "Standard, three to five days"),
                ),
            ),
    )
}

#[derive(Clone)]
struct Props {
    width: f64,
    height: f64,
    direction: SharedString,
    opacity: f64,
    visible: bool,
}

fn edit<V: 'static>(
    props: &Entity<Props>,
    apply: fn(&mut Props, V),
) -> impl Fn(V, &mut Window, &mut App) + 'static {
    let props = props.clone();
    move |value, _, cx| {
        props.update(cx, |props, cx| {
            apply(props, value);
            cx.notify();
        })
    }
}

pub fn properties(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let props = keep(
        "props",
        || Props {
            width: 240.0,
            height: 144.0,
            direction: "row".into(),
            opacity: 0.8,
            visible: true,
        },
        window,
        cx,
    );
    let now = props.read(cx).clone();
    let theme = cx.theme();
    let colors = &theme.colors;
    let direction = props.clone();
    let grid = PropertyGrid::new("props-grid")
        .group(
            PropertyGroup::new("Layout")
                .row(
                    "Width",
                    NumberInput::new("prop-width", now.width)
                        .range(40.0, 320.0)
                        .size(ControlSize::Sm)
                        .on_change(edit(&props, |p, v| p.width = v)),
                )
                .row(
                    "Height",
                    NumberInput::new("prop-height", now.height)
                        .range(40.0, 200.0)
                        .size(ControlSize::Sm)
                        .on_change(edit(&props, |p, v| p.height = v)),
                )
                .row(
                    "Direction",
                    SegmentedControl::new("prop-direction", now.direction.clone())
                        .segment("row", "Row", None)
                        .segment("column", "Column", None)
                        .size(ControlSize::Sm)
                        .on_change(move |value, _, cx| {
                            direction.update(cx, |props, cx| {
                                props.direction = value.clone();
                                cx.notify();
                            })
                        }),
                ),
        )
        .group(
            PropertyGroup::new("Appearance")
                .row(
                    "Opacity",
                    Slider::new("prop-opacity", now.opacity)
                        .range(0.0, 1.0)
                        .step(0.01)
                        .on_change(edit(&props, |p, v| p.opacity = v)),
                )
                .row(
                    "Visible",
                    Switch::new("prop-visible", now.visible)
                        .on_change(edit(&props, |p, v| p.visible = v)),
                ),
        )
        .group(
            PropertyGroup::new("Export").folded().row(
                "Scale",
                NumberInput::new("prop-scale", 2.0)
                    .range(1.0, 4.0)
                    .size(ControlSize::Sm),
            ),
        );
    let bar = |ix: usize| {
        div()
            .flex_1()
            .rounded(theme.radius(Radius::Sm))
            .bg(colors.accent.opacity(0.35 + 0.2 * ix as f32))
    };
    let frame = div()
        .flex()
        .when(now.direction == "column", |frame| frame.flex_col())
        .gap_2()
        .p_2()
        .w(px(now.width as f32))
        .h(px(now.height as f32))
        .rounded(theme.radius(Radius::Md))
        .border_1()
        .border_color(colors.accent)
        .opacity(now.opacity as f32)
        .children((0..3).map(bar));
    section(
        "PropertyGrid",
        "An inspector: groups fold, each row a name and its editor. The owner keeps the values; the frame on the right follows them.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(
                div()
                    .w_80()
                    .flex_none()
                    .border_1()
                    .border_color(colors.border)
                    .rounded(theme.radius(Radius::Md))
                    .child(grid),
            )
            .child(
                div()
                    .flex_1()
                    .h_64()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(theme.radius(Radius::Md))
                    .bg(colors.sunken)
                    .when(now.visible, |stage| stage.child(frame)),
            ),
    )
}

pub fn timelines(cx: &mut App) -> impl IntoElement + use<> {
    let deploy = Timeline::new()
        .item(
            TimelineItem::new("Build started")
                .time("14:02")
                .icon(IconName::GitCommitHorizontal)
                .child("main at 4f2a9c1, by Radia Perlman"),
        )
        .item(
            TimelineItem::new("Tests passed")
                .time("14:06")
                .icon(IconName::Check)
                .tone(Severity::Success)
                .child("412 tests in 3 min 12 s"),
        )
        .item(
            TimelineItem::new("Deployed to staging")
                .time("14:09")
                .icon(IconName::Rocket)
                .tone(Severity::Info),
        )
        .item(
            TimelineItem::new("Promote to production")
                .icon(IconName::Hourglass)
                .child("Waiting for approval"),
        );
    let project = Timeline::new()
        .item(TimelineItem::new("Kickoff").time("Jan 12"))
        .item(
            TimelineItem::new("Design review")
                .time("Feb 3")
                .child("Type scale settled; spacing loosened by one step."),
        )
        .item(
            TimelineItem::new("Beta")
                .time("Mar 18")
                .tone(Severity::Success),
        )
        .item(
            TimelineItem::new("Launch")
                .time("Apr 30")
                .tone(Severity::Warning)
                .child("Planned"),
        );
    section(
        "Timeline / TimelineItem",
        "Events down a rail, marked by an icon in a tinted ring or by a dot, with detail below.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_12()
            .child(div().flex_1().child(deploy))
            .child(div().flex_1().child(project)),
    )
}

/// The canned arrivals the demo cycles through: actor, verb, object.
const ARRIVALS: [(&str, &str, &str); 3] = [
    ("Sophie Wilson", "commented on", "Release notes"),
    ("Tim Berners-Lee", "linked", "Spec draft"),
    ("Radia Perlman", "approved", "Promote to production"),
];

pub fn feed(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let opened = *keep("feed-opened", Timestamp::now, window, cx).read(cx);
    let arrivals = keep("feed-arrivals", Vec::<Timestamp>::new, window, cx);
    let (now, post) = (arrivals.read(cx).clone(), arrivals.clone());
    let ago = |span: jiff::Span| opened.checked_sub(span).expect("a time in range");
    let feed = now
        .iter()
        .enumerate()
        .rev()
        .fold(ActivityFeed::new("feed"), |feed, (ix, at)| {
            let (actor, verb, object) = ARRIVALS[ix % ARRIVALS.len()];
            feed.entry(Activity::new(format!("arrival-{ix}"), actor, verb, *at).object(object))
        });
    let feed = feed
        .entry(
            Activity::new("comment", "Grace Hopper", "commented on", ago(3.minutes()))
                .object("Design review")
                .body("The cards finally breathe. Ship it."),
        )
        .entry(
            Activity::new("deploy", "Build bot", "deployed", ago(2.hours()))
                .object("v0.4.0")
                .icon(IconName::Rocket),
        )
        .entry(
            Activity::new("merge", "Alan Turing", "merged", ago(26.hours()))
                .object("Timeline component"),
        )
        .entry(
            Activity::new("create", "Ada Lovelace", "created", ago(72.hours()))
                .object("Q4 roadmap"),
        );
    section(
        "ActivityFeed",
        "Who did what, and when, newest first. A new entry folds in at the top; the times stay fresh.",
        cx,
    )
    .child(div().max_w(px(560.)).child(feed))
    .child(div().flex().child(probe(
        "feed-post",
        Button::new("feed-post", "Something happens")
            .variant(ButtonVariant::Ghost)
            .icon(IconName::Plus)
            .on_click(move |_, _, cx| {
                let mut next = post.read(cx).clone();
                next.push(Timestamp::now());
                set(&post, next, cx)
            }),
    )))
}

pub fn changelog(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Changelog",
        "Releases, newest first, each change under its kind.",
        cx,
    )
    .child(
        div().max_w(px(560.)).child(
            Changelog::new()
                .release(
                    Release::new("2.4.0", "Sep 25, 2026")
                        .summary("Faster search, and a calmer sidebar.")
                        .change(Change::Added, "Search inside attachments.")
                        .change(Change::Fixed, "Pasting a table keeps its borders.")
                        .change(Change::Added, "Pin a note to the sidebar.")
                        .change(Change::Changed, "The sidebar folds with a narrow window."),
                )
                .release(
                    Release::new("2.3.1", "Sep 9, 2026")
                        .change(Change::Fixed, "Sync resumes after sleep.")
                        .change(Change::Security, "Shared links expire when they should."),
                )
                .release(
                    Release::new("2.3.0", "Aug 28, 2026")
                        .change(Change::Added, "Templates for new notes.")
                        .change(Change::Deprecated, "The legacy export format.")
                        .change(Change::Removed, "The old toolbar."),
                ),
        ),
    )
}
