use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    feedback::{
        Alert, Banner, Callout, EmptyState, InlineMessage, Notification, NotificationCenter,
        StatusMessage, Toast, ToastViewport, Toaster,
    },
    layout::Collapsible,
    primitives::{IconName, Severity},
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen, specimens},
};

pub fn toasts(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let toaster = keep("toaster", Toaster::default, window, cx);
    let fire = |key: &'static str, label: &'static str, make: fn() -> Toast| {
        let toaster = toaster.clone();
        probe(
            key,
            Button::new(key, label).on_click(move |_, _, cx| {
                toaster.update(cx, |toaster, cx| {
                    toaster.push(make(), cx);
                })
            }),
        )
    };
    section(
        "Toast / Snackbar / UndoToast",
        "Short messages stack in the corner, the newest nearest it. Each rises in as its room opens and folds away under the rest. The pointer on the stack holds their time.",
        cx,
    )
    .child(
        row()
            .child(fire("toast-saved", "Success", || {
                Toast::new("Changes saved").severity(Severity::Success)
            }))
            .child(fire("toast-error", "Error", || {
                Toast::new("Couldn't sync")
                    .body("Check your connection, then try again.")
                    .severity(Severity::Danger)
                    .action("Retry", |_, _| log::info!("gallery: retry"))
            }))
            .child(fire("toast-warning", "Warning", || {
                Toast::new("Storage almost full")
                    .body("46 of 50 GB used.")
                    .severity(Severity::Warning)
            }))
            .child(fire("toast-info", "Info", || {
                Toast::new("A new version is ready")
                    .severity(Severity::Info)
                    .action("Reload", |_, _| log::info!("gallery: reload"))
            }))
            .child(fire("toast-undo", "Undo", || {
                Toast::new("3 files moved to Trash").undo(|_, _| log::info!("gallery: undo"))
            }))
            .child(fire("toast-plain", "Plain", || Toast::new("Link copied"))),
    )
    .child(ToastViewport::new("toast-viewport", &toaster))
}

/// Which demo notifications are left, and which are still unread.
#[derive(Clone)]
struct Inbox {
    unread: [bool; 5],
    cleared: bool,
}

impl Inbox {
    fn fresh() -> Self {
        Self {
            unread: [true, true, false, false, false],
            cleared: false,
        }
    }
}

pub fn center(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let inbox = keep("inbox", Inbox::fresh, window, cx);
    let now = inbox.read(cx).clone();
    let (read, clear, reset) = (inbox.clone(), inbox.clone(), inbox.clone());
    let theme = cx.theme();
    let item =
        |ix: usize, icon: IconName, title: &'static str, body: &'static str, time: &'static str| {
            Notification::new(("note", ix), icon, title)
                .body(body)
                .time(time)
                .unread(now.unread[ix])
        };
    let (today, earlier) = if now.cleared {
        (Vec::new(), Vec::new())
    } else {
        (
            vec![
                item(
                    0,
                    IconName::MessageSquare,
                    "Mia Chen replied",
                    "“Looks right. Ship it after the copy pass.”",
                    "2m",
                ),
                item(
                    1,
                    IconName::GitPullRequest,
                    "Review requested",
                    "Hana asked you to review #482, toast motion.",
                    "18m",
                )
                .action(Button::new("note-review", "Review").primary())
                .action(Button::new("note-later", "Later").variant(ButtonVariant::Ghost)),
                item(
                    2,
                    IconName::CalendarDays,
                    "Design review moved",
                    "Now Thursday, 14:00 CET.",
                    "1h",
                ),
            ],
            vec![
                item(
                    3,
                    IconName::Download,
                    "Export ready",
                    "quarterly-report.pdf · 2.4 MB",
                    "Yesterday",
                ),
                item(
                    4,
                    IconName::Users,
                    "Léa Martin joined",
                    "Say hello in #studio.",
                    "Mon",
                ),
            ],
        )
    };
    section(
        "Notification / NotificationCenter",
        "Messages under day headings. Unread ones carry a dot; Mark all read and Clear show only while they have work.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_end()
            .gap_4()
            .child(probe(
                "center",
                div()
                    .w_96()
                    .h_112()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .child(
                        NotificationCenter::new("center")
                            .group("Today", today)
                            .group("Earlier", earlier)
                            .on_read_all(move |_, cx| {
                                let cleared = read.read(cx).cleared;
                                set(&read, Inbox { unread: [false; 5], cleared }, cx)
                            })
                            .on_clear(move |_, cx| {
                                set(&clear, Inbox { unread: [false; 5], cleared: true }, cx)
                            }),
                    ),
            ))
            .child(probe(
                "center-reset",
                Button::new("center-reset", "Reset")
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, _, cx| set(&reset, Inbox::fresh(), cx)),
            )),
    )
}

pub fn alerts(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("alert-shown", || true, window, cx);
    let (hide, show) = (shown.clone(), shown.clone());
    section(
        "Alert",
        "A message in the flow of the page, tinted by how much it matters.",
        cx,
    )
    .child(
        div()
            .w_128()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Alert::new("alert-info", Severity::Info, "Scheduled maintenance")
                    .body("Sync pauses Sunday, 02:00 to 03:00 UTC."),
            )
            .child(
                Alert::new("alert-success", Severity::Success, "Domain verified")
                    .body("ely.design now points to this workspace."),
            )
            .child(
                Alert::new(
                    "alert-warning",
                    Severity::Warning,
                    "Your trial ends in 3 days",
                )
                .body("Add a payment method to keep your projects.")
                .action(Button::new("alert-pay", "Add payment method")),
            )
            .child(if *shown.read(cx) {
                Alert::new("alert-danger", Severity::Danger, "Couldn't publish")
                    .body("Two pages link to files that no longer exist.")
                    .action(Button::new("alert-show", "Show pages"))
                    .on_dismiss(move |_, cx| set(&hide, false, cx))
                    .into_any_element()
            } else {
                Button::new("alert-again", "Show the closed alert")
                    .variant(ButtonVariant::Link)
                    .on_click(move |_, _, cx| set(&show, true, cx))
                    .into_any_element()
            }),
    )
}

pub fn banner(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("banner-open", || true, window, cx);
    let shown = *open.read(cx);
    let (close, again) = (open.clone(), open.clone());
    let theme = cx.theme();
    section(
        "Banner",
        "A strip across the top of a window. Closing it folds it away.",
        cx,
    )
    .child(
        div()
            .w_128()
            .h_40()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.bg)
            .child(
                Collapsible::new("banner-fold", shown).child(
                    Banner::new(
                        "banner",
                        Severity::Warning,
                        "You're offline. Edits sync when you reconnect.",
                    )
                    .action("Retry", |_, _| log::info!("gallery: retry"))
                    .on_dismiss(move |_, cx| set(&close, false, cx)),
                ),
            )
            .child(
                div()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Caption::new("The window's content sits under it."))
                    .when(!shown, |content| {
                        content.child(
                            Button::new("banner-again", "Show the banner")
                                .variant(ButtonVariant::Link)
                                .on_click(move |_, _, cx| set(&again, true, cx)),
                        )
                    }),
            ),
    )
}

pub fn callouts(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    section(
        "Callout / Admonition",
        "A note set apart in reading: note, tip, warning and danger.",
        cx,
    )
    .child(
        div()
            .w_128()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(theme.colors.fg)
            .child(
                Callout::new(Severity::Info)
                    .child("Themes cross-fade in 280 ms. The theme's reduced motion settles entrances, folds and fades at once; an undo line still counts real time."),
            )
            .child(
                Callout::new(Severity::Success)
                    .child("Tab walks every control on a page; Shift-Tab walks back."),
            )
            .child(
                Callout::new(Severity::Warning)
                    .child("Layouts saved by a newer version are refused, not guessed."),
            )
            .child(
                Callout::new(Severity::Danger)
                    .title("Irreversible")
                    .child("Deleting a workspace removes its files for everyone."),
            ),
    )
}

pub fn lines(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const STEPS: [&str; 3] = ["Ready to upload", "Uploading 3 files…", "Uploaded 3 files"];
    let at = keep("status-at", || 0usize, window, cx);
    let now = *at.read(cx);
    let step = at.clone();
    section(
        "InlineMessage / StatusMessage",
        "One line beside what it is about. A status line rises in each time it changes.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "success",
                InlineMessage::new(Severity::Success, "Saved"),
                cx,
            ))
            .child(specimen(
                "danger",
                InlineMessage::new(Severity::Danger, "That name is taken"),
                cx,
            ))
            .child(specimen(
                "warning",
                InlineMessage::new(Severity::Warning, "Unsaved changes"),
                cx,
            ))
            .child(specimen(
                "info",
                InlineMessage::new(Severity::Info, "Updated 2 minutes ago"),
                cx,
            )),
    )
    .child(
        row()
            .child(probe(
                "status",
                Button::new("status-step", "Next status")
                    .on_click(move |_, _, cx| set(&step, (now + 1) % STEPS.len(), cx)),
            ))
            .child(StatusMessage::new("status", STEPS[now]).icon(IconName::Upload)),
    )
}

pub fn empties(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let frame = || {
        div()
            .flex_1()
            .min_w_0()
            .h_80()
            .flex()
            .flex_col()
            .justify_center()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
    };
    section(
        "EmptyState",
        "What a view shows with nothing in it: no data yet, no results, or a first start.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_3()
            .child(
                frame().child(
                    EmptyState::new("empty-inbox", IconName::Inbox, "No messages yet")
                        .body("When someone writes to you, it lands here, newest first, until you archive it.")
                        .action(Button::new("empty-compose", "Write a message")),
                ),
            )
            .child(
                frame().child(
                    EmptyState::new(
                        "empty-search",
                        IconName::SearchX,
                        "No results for “quarterly”",
                    )
                    .body("Try fewer words, or check the spelling.")
                    .action(Button::new("empty-clear", "Clear search")),
                ),
            )
            .child(
                frame().child(
                    EmptyState::new(
                        "empty-first",
                        IconName::Sparkles,
                        "Start your first project",
                    )
                    .body("Projects keep files, people and tasks together.")
                    .action(Button::new("empty-new", "New project").primary()),
                ),
            ),
    )
}
