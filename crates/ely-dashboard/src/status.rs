use gpui::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};
use jiff::{Timestamp, civil::Date};

use crate::{
    data_display::{Badge, Tone},
    primitives::Tooltip,
    theme::{ActiveTheme, Palette, Radius, TextSize},
    typography::{Ellipsis, format},
};

/// The tone a day's uptime wears: whole, a little down, or down.
fn day_tone(uptime: f32) -> Tone {
    match uptime {
        up if up >= 0.999 => Tone::Success,
        up if up >= 0.99 => Tone::Warning,
        _ => Tone::Danger,
    }
}

fn tone_color(tone: Tone, colors: &Palette) -> Hsla {
    match tone {
        Tone::Success => colors.success,
        Tone::Warning => colors.warning,
        Tone::Danger => colors.danger,
        Tone::Info => colors.info,
        Tone::Accent => colors.accent,
        Tone::Neutral => colors.border_strong,
    }
}

/// The share of time up across days with readings.
pub fn uptime(days: &[Option<f32>]) -> Option<f32> {
    let read: Vec<f32> = days.iter().flatten().copied().collect();
    (!read.is_empty()).then(|| read.iter().sum::<f32>() / read.len() as f32)
}

/// A service's days as bars, oldest first, each in the tone of its uptime, a quiet bar where there is no reading, with each day's date and share in its tip; the share up across all of them heads it.
///
/// ```
/// use ely_dashboard::UptimeBar;
/// use jiff::civil::date;
///
/// let bar = UptimeBar::new("api", "API", date(2026, 10, 20), [Some(1.0), Some(0.98), None, Some(0.999)]);
/// ```
#[derive(IntoElement)]
pub struct UptimeBar {
    id: ElementId,
    name: SharedString,
    first: Date,
    days: Vec<Option<f32>>,
}

impl UptimeBar {
    /// `days` are shares up from 0 to 1, oldest first from `first`.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        first: Date,
        days: impl IntoIterator<Item = Option<f32>>,
    ) -> Self {
        let days: Vec<Option<f32>> = days.into_iter().collect();
        for share in days.iter().flatten() {
            assert!((0.0..=1.0).contains(share), "an uptime of {share}");
        }
        Self {
            id: id.into(),
            name: name.into(),
            first,
            days,
        }
    }
}

impl RenderOnce for UptimeBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let whole = uptime(&self.days).map_or("No readings".to_string(), |share| {
            format::percent(share as f64, 2, false)
        });
        let bars = self.days.iter().enumerate().map(|(ix, day)| {
            let date = self.first.saturating_add(jiff::Span::new().days(ix as i64));
            let tip = match day {
                Some(share) => format!(
                    "{} · {}",
                    date.strftime("%b %-d"),
                    format::percent(*share as f64, 2, false)
                ),
                None => format!("{} · no reading", date.strftime("%b %-d")),
            };
            let color = day.map_or(colors.border, |share| tone_color(day_tone(share), &colors));
            div()
                .id((self.id.clone(), format!("day-{ix}")))
                .flex_1()
                .min_w_0()
                .h_full()
                .rounded(theme.radius(Radius::Sm))
                .bg(color)
                .tooltip(Tooltip::text(tip))
        });
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .gap_2()
                    .text_size(theme.text_size(TextSize::Sm))
                    .child(div().flex_1().min_w_0().child(Ellipsis::new(self.name)))
                    .child(
                        div()
                            .flex_none()
                            .text_color(theme.colors.fg_muted)
                            .child(whole),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_0p5()
                    .h(theme.control_height(crate::theme::ControlSize::Sm))
                    .children(bars),
            )
    }
}

/// Where a service stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Health {
    /// Working.
    Operational,
    /// Down for planned work.
    Maintenance,
    /// Slower or worse than it should be.
    Degraded,
    /// Down in part.
    PartialOutage,
    /// Down.
    MajorOutage,
}

impl Health {
    /// Its key, the value its choice carries.
    pub fn key(self) -> &'static str {
        match self {
            Health::Operational => "operational",
            Health::Maintenance => "maintenance",
            Health::Degraded => "degraded",
            Health::PartialOutage => "partial-outage",
            Health::MajorOutage => "major-outage",
        }
    }

    /// Its words in the shown locale.
    pub fn words(self, cx: &App) -> SharedString {
        crate::i18n::text(cx, &format!("dashboard.health.{}", self.key()), &[])
    }

    /// Its tone.
    pub fn tone(self) -> Tone {
        match self {
            Health::Operational => Tone::Success,
            Health::Maintenance => Tone::Info,
            Health::Degraded => Tone::Warning,
            Health::PartialOutage | Health::MajorOutage => Tone::Danger,
        }
    }
}

/// A service: its name, where it stands, and its uptime share when known.
#[derive(Clone, Debug, PartialEq)]
pub struct Service {
    /// What it is called.
    pub name: SharedString,
    /// Where it stands.
    pub health: Health,
    /// Its share of time up, from 0 to 1.
    pub uptime: Option<f32>,
}

/// The line over a set of services: the worst of them.
pub fn overall(services: &[Service]) -> Health {
    services
        .iter()
        .map(|service| service.health)
        .max()
        .expect("a status lists a service")
}

/// Services under a line for the whole system, each with where it stands and its uptime.
///
/// ```
/// use ely_dashboard::{Health, Service, ServiceStatus};
///
/// let status = ServiceStatus::new([
///     Service { name: "API".into(), health: Health::Operational, uptime: Some(0.999) },
///     Service { name: "Search".into(), health: Health::Degraded, uptime: None },
/// ]);
/// ```
#[derive(IntoElement)]
pub struct ServiceStatus {
    services: Vec<Service>,
}

impl ServiceStatus {
    /// The status of `services`; panics on none.
    pub fn new(services: impl IntoIterator<Item = Service>) -> Self {
        let services: Vec<_> = services.into_iter().collect();
        assert!(!services.is_empty(), "service status: no service");
        Self { services }
    }
}

impl RenderOnce for ServiceStatus {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let whole = overall(&self.services);
        let headline = crate::i18n::text(cx, &format!("dashboard.headline.{}", whole.key()), &[]);
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pb_3()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(Badge::new(whole.words(cx)).tone(whole.tone()).dot())
                    .child(div().min_w_0().child(Ellipsis::new(headline))),
            )
            .children(self.services.into_iter().map(|service| {
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .py_2()
                    .border_t_1()
                    .border_color(theme.colors.border)
                    .text_size(theme.text_size(TextSize::Sm))
                    .child(div().flex_1().min_w_0().child(Ellipsis::new(service.name)))
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap_2()
                            .children(service.uptime.map(|share| {
                                div()
                                    .text_color(theme.colors.fg_muted)
                                    .child(format::percent(share as f64, 2, false))
                            }))
                            .child(
                                Badge::new(service.health.words(cx))
                                    .tone(service.health.tone())
                                    .dot(),
                            ),
                    )
            }))
    }
}

/// What a health check found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// It passed.
    Passing,
    /// It warned.
    Warning,
    /// It failed.
    Failing,
    /// Waiting on a result.
    Pending,
}

/// A health check's state as a dot and a word, with when it last ran and how long it took.
///
/// ```
/// use ely_dashboard::{Check, HealthCheck};
///
/// let check = HealthCheck::new("Database", Check::Passing).took(12);
/// ```
#[derive(IntoElement)]
pub struct HealthCheck {
    name: SharedString,
    state: Check,
    ran: Option<(Timestamp, Timestamp)>,
    took_ms: Option<u32>,
}

impl HealthCheck {
    /// A check named `name` in `state`, its last run and time unknown.
    pub fn new(name: impl Into<SharedString>, state: Check) -> Self {
        Self {
            name: name.into(),
            state,
            ran: None,
            took_ms: None,
        }
    }

    /// When it last ran, dated against `now`.
    pub fn ran(mut self, at: Timestamp, now: Timestamp) -> Self {
        self.ran = Some((at, now));
        self
    }

    /// How long its last run took, in milliseconds.
    pub fn took(mut self, ms: u32) -> Self {
        self.took_ms = Some(ms);
        self
    }
}

impl RenderOnce for HealthCheck {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (tone, words) = match self.state {
            Check::Passing => (Tone::Success, "Passing"),
            Check::Warning => (Tone::Warning, "Warning"),
            Check::Failing => (Tone::Danger, "Failing"),
            Check::Pending => (Tone::Neutral, "Pending"),
        };
        let mut detail = Vec::new();
        if let Some((at, now)) = self.ran {
            detail.push(format!("ran {}", format::relative(at, now)));
        }
        if let Some(ms) = self.took_ms {
            detail.push(format!("{ms} ms"));
        }
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x_2()
            .text_size(theme.text_size(TextSize::Sm))
            .child(
                div()
                    .flex_1()
                    .min_w(theme.label_width())
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_none()
                            .size(theme.dashboard().dot)
                            .rounded_full()
                            .bg(tone_color(tone, &theme.colors)),
                    )
                    .child(div().flex_1().min_w_0().child(Ellipsis::new(self.name))),
            )
            .child(div().flex_none().text_color(theme.colors.fg_muted).child(
                if detail.is_empty() {
                    words.to_string()
                } else {
                    format!("{words} · {}", detail.join(" · "))
                },
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Health, Service, overall, uptime};

    #[test]
    fn uptime_averages_the_days_with_readings_and_the_worst_service_leads() {
        assert_eq!(uptime(&[Some(1.0), None, Some(0.98)]), Some(0.99));
        assert_eq!(uptime(&[None]), None);
        let service = |health| Service {
            name: "s".into(),
            health,
            uptime: None,
        };
        assert_eq!(
            overall(&[
                service(Health::Operational),
                service(Health::Degraded),
                service(Health::Maintenance)
            ]),
            Health::Degraded
        );
    }

    #[test]
    fn a_days_tone_turns_at_three_nines_and_two() {
        use crate::data_display::Tone;
        assert_eq!(super::day_tone(0.9995), Tone::Success);
        assert_eq!(super::day_tone(0.995), Tone::Warning);
        assert_eq!(super::day_tone(0.985), Tone::Danger);
    }
}
