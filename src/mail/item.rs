use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};
use jiff::{Timestamp, tz::TimeZone};

use crate::{
    forms::OnFlag,
    i18n,
    lists::ListItem,
    primitives::{FocusRing, Icon, IconName, Tooltip},
    theme::{ActiveTheme, IconSize, Radius, TextSize},
    typography::format,
};

/// A message as a list shows it: who sent it, its subject and first words, when it came, and its flags.
#[derive(Clone, Debug, PartialEq)]
pub struct Mail {
    pub key: SharedString,
    pub from: SharedString,
    pub subject: SharedString,
    pub snippet: Option<SharedString>,
    /// None when the message names no date.
    pub at: Option<Timestamp>,
    pub unread: bool,
    pub starred: bool,
    pub attachments: usize,
}

impl Mail {
    pub fn new(
        key: impl Into<SharedString>,
        from: impl Into<SharedString>,
        subject: impl Into<SharedString>,
        at: Timestamp,
    ) -> Self {
        Self {
            key: key.into(),
            from: from.into(),
            subject: subject.into(),
            snippet: None,
            at: Some(at),
            unread: false,
            starred: false,
            attachments: 0,
        }
    }

    /// A message that names no date; its row shows none.
    pub fn undated(
        key: impl Into<SharedString>,
        from: impl Into<SharedString>,
        subject: impl Into<SharedString>,
    ) -> Self {
        Self {
            key: key.into(),
            from: from.into(),
            subject: subject.into(),
            snippet: None,
            at: None,
            unread: false,
            starred: false,
            attachments: 0,
        }
    }

    /// Its first words.
    pub fn snippet(mut self, text: impl Into<SharedString>) -> Self {
        self.snippet = Some(text.into());
        self
    }

    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    pub fn starred(mut self, starred: bool) -> Self {
        self.starred = starred;
        self
    }

    /// How many files it carries.
    pub fn attachments(mut self, count: usize) -> Self {
        self.attachments = count;
        self
    }
}

/// How a list dates a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum When {
    /// Today: the time.
    Time(String),
    /// Within the week: Monday is 1.
    Weekday(i8),
    /// Within the year: month, day.
    Day(i8, i8),
    /// Before: year, month, day.
    Date(i16, i8, i8),
}

/// When a message came: the time today, the weekday within the week, the date within the year, and the year past that.
pub(super) fn when(at: Timestamp, now: Timestamp, zone: &TimeZone) -> When {
    let (at, now) = (at.to_zoned(zone.clone()), now.to_zoned(zone.clone()));
    let days = at
        .date()
        .until(now.date())
        .expect("two dates have a span")
        .get_days();
    match days {
        0 => When::Time(at.strftime("%H:%M").to_string()),
        1..=6 => When::Weekday(at.weekday().to_monday_one_offset()),
        _ if at.year() == now.year() => When::Day(at.month(), at.day()),
        _ => When::Date(at.year(), at.month(), at.day()),
    }
}

/// The date as the reader's language writes it.
pub(super) fn say(when: When, cx: &App) -> SharedString {
    let date = |key: &str, year: i16, month: i8, day: i8| {
        let name = i18n::text(cx, &format!("date.month.{month}"), &[]);
        let args = [
            ("year", year.to_string()),
            ("month", name.to_string()),
            ("m", month.to_string()),
            ("day", day.to_string()),
        ];
        let args: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
        i18n::text(cx, key, &args)
    };
    match when {
        When::Time(time) => time.into(),
        When::Weekday(day) => i18n::text(cx, &format!("mail.weekday.{day}"), &[]),
        When::Day(month, day) => date("mail.date.this_year", 0, month, day),
        When::Date(year, month, day) => date("date.day", year, month, day),
    }
}

/// A message as a row: its sender, strong beside a dot while unread, the subject and first words under it, when it came, a paperclip when it carries files, and its star. Every row is three lines high. With a handler the star toggles, its press kept off the row.
#[derive(IntoElement)]
pub struct MailItem {
    id: ElementId,
    mail: Mail,
    zone: Option<TimeZone>,
    on_star: Option<OnFlag>,
}

impl MailItem {
    pub fn new(id: impl Into<ElementId>, mail: Mail) -> Self {
        Self {
            id: id.into(),
            mail,
            zone: None,
            on_star: None,
        }
    }

    /// The zone its date reads in; the system's otherwise.
    pub fn zone(mut self, zone: TimeZone) -> Self {
        self.zone = Some(zone);
        self
    }

    /// Gets the star's new state.
    pub fn on_star(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_star = Some(Rc::new(handler));
        self
    }

    pub(super) fn item(self, cx: &App) -> ListItem {
        let theme = cx.theme();
        let colors = &theme.colors;
        let mail = self.mail;
        let zone = self
            .zone
            .unwrap_or_else(|| format::system_zone("mail item"));
        let date = mail.at.map(|at| say(when(at, Timestamp::now(), &zone), cx));
        let dot = div()
            .size(theme.icon_size(IconSize::Xs))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size(theme.status_dot())
                    .rounded_full()
                    .when(mail.unread, |dot| dot.bg(colors.accent)),
            );
        let starred = mail.starred;
        let tint = if starred {
            colors.warning
        } else {
            colors.fg_subtle
        };
        let mark = |icon| Icon::new(icon).size(IconSize::Sm).color(tint);
        let star: Option<AnyElement> = match self.on_star {
            Some(set) => {
                let key = mail.key.clone();
                let named = key.clone();
                let tip = i18n::text(cx, if starred { "mail.unstar" } else { "mail.star" }, &[]);
                Some(
                    div()
                        .id((self.id.clone(), "star"))
                        .debug_selector(move || format!("star {named}"))
                        .p_0p5()
                        .rounded(theme.radius(Radius::Sm))
                        .border_1()
                        .border_color(gpui::transparent_black())
                        .tab_index(0)
                        .focus_ring(cx)
                        .cursor_pointer()
                        .hover(|style| style.bg(colors.hover))
                        .tooltip(Tooltip::text(tip))
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_key_down(|event, _, cx| {
                            let key = event.keystroke.key.as_str();
                            if matches!(key, "space" | "enter")
                                && !event.keystroke.modifiers.modified()
                            {
                                cx.stop_propagation();
                            }
                        })
                        .on_click(move |_, window, cx| {
                            log::info!("mail {key}: star {}", !starred);
                            set(!starred, window, cx)
                        })
                        .child(mark(IconName::Star))
                        .into_any_element(),
                )
            }
            None => starred.then(|| mark(IconName::Star).into_any_element()),
        };
        let clip = (mail.attachments > 0).then(|| {
            Icon::new(IconName::Paperclip)
                .size(IconSize::Sm)
                .color(colors.fg_subtle)
        });
        let trail = div()
            .flex()
            .flex_col()
            .items_end()
            .gap_1()
            .children(date.map(|date| {
                let named = mail.key.clone();
                div()
                    .debug_selector(move || format!("date {named}"))
                    .text_size(theme.text_size(TextSize::Xs))
                    .child(date)
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .children(clip)
                    .children(star),
            );
        // A blank third line keeps rows one height.
        let snippet = mail.snippet.unwrap_or_else(|| "\u{a0}".into());
        ListItem::new(self.id, mail.from)
            .leading(dot)
            .strong(mail.unread)
            .description(mail.subject)
            .detail(snippet)
            .trailing(trail)
    }
}

impl RenderOnce for MailItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.item(cx)
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn a_date_reads_as_the_time_the_weekday_the_date_or_the_year() {
        let zone = TimeZone::UTC;
        let at = |y, m, d, h| {
            date(y, m, d)
                .at(h, 5, 0, 0)
                .to_zoned(zone.clone())
                .expect("a UTC time")
                .timestamp()
        };
        let now = at(2026, 9, 26, 12);
        assert_eq!(
            when(at(2026, 9, 26, 9), now, &zone),
            When::Time("09:05".into())
        );
        assert_eq!(when(at(2026, 9, 25, 23), now, &zone), When::Weekday(5));
        assert_eq!(when(at(2026, 9, 20, 9), now, &zone), When::Weekday(7));
        assert_eq!(when(at(2026, 9, 19, 9), now, &zone), When::Day(9, 19));
        assert_eq!(
            when(at(2025, 12, 31, 9), now, &zone),
            When::Date(2025, 12, 31)
        );
    }
}
