mod calendar;
mod cron;
mod period;
mod picker;
mod relative;
mod rule;
mod time;
mod week;
mod zone;

use gpui::App;
use jiff::{
    Timestamp, Zoned,
    civil::{Date, Time},
};

use crate::{i18n::text, typography::format::system_zone};

pub use calendar::Calendar;
pub(crate) use calendar::{month_grid, week_start};
pub use cron::CronEditor;
pub use period::{MonthPicker, QuarterPicker, YearPicker};
pub use picker::{DatePicker, DateRangePicker};
pub(crate) use picker::{Face, dropdown, picker_field};
pub use relative::{RelativeDatePicker, RelativeRange};
pub use rule::CronRule;
pub(crate) use rule::weekday_words;
pub use time::{DateTimePicker, DurationPicker, TimePicker};
pub use week::WeekPicker;
pub use zone::TimezoneSelect;

/// The system's clock in its own zone, UTC when that zone is unknown.
pub(crate) fn zoned_now() -> Zoned {
    Timestamp::now().to_zoned(system_zone("date pickers"))
}

/// A day as the reader's language writes it.
pub(crate) fn show_date(date: Date, cx: &App) -> String {
    say("date.day", date, &[], cx)
}

/// A month's title, "September 2026".
pub(crate) fn show_month(date: Date, cx: &App) -> String {
    let long = text(cx, &format!("date.month_long.{}", date.month()), &[]);
    say("date.title", date, &[("month", &long)], cx)
}

/// `key` filled with `more`, then `date`'s parts.
fn say(key: &str, date: Date, more: &[(&str, &str)], cx: &App) -> String {
    let month = text(cx, &format!("date.month.{}", date.month()), &[]);
    let (year, m, day) = (
        date.year().to_string(),
        date.month().to_string(),
        date.day().to_string(),
    );
    let mut args = more.to_vec();
    args.extend([
        ("year", year.as_str()),
        ("month", month.as_ref()),
        ("m", m.as_str()),
        ("day", day.as_str()),
    ]);
    text(cx, key, &args).to_string()
}

/// A time as "09:30".
pub(crate) fn show_time(time: Time) -> String {
    time.strftime("%H:%M").to_string()
}

/// A span as "Sep 3 – 17, 2026", naming only what differs.
pub(crate) fn show_span(start: Date, end: Date, cx: &App) -> String {
    let day = end.day().to_string();
    if start == end {
        show_date(start, cx)
    } else if start.year() != end.year() {
        let (from, to) = (show_date(start, cx), show_date(end, cx));
        text(cx, "date.span", &[("start", &from), ("end", &to)]).to_string()
    } else if start.month() != end.month() {
        let name = text(cx, &format!("date.month.{}", end.month()), &[]);
        let m = end.month().to_string();
        let more = [("end_month", name.as_ref()), ("end_m", &m), ("end", &day)];
        say("date.span.year", start, &more, cx)
    } else {
        say("date.span.month", start, &[("end", &day)], cx)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{App, TestAppContext};
    use jiff::civil::{date, time};

    use super::{show_date, show_month, show_span, show_time};
    use crate::i18n::I18n;

    fn spans(cx: &App) -> [String; 3] {
        [
            show_span(date(2026, 9, 3), date(2026, 9, 17), cx),
            show_span(date(2026, 9, 28), date(2026, 10, 2), cx),
            show_span(date(2026, 12, 30), date(2027, 1, 2), cx),
        ]
    }

    #[gpui::test]
    fn dates_times_and_spans_read_short(cx: &mut TestAppContext) {
        cx.update(|cx| {
            assert_eq!(show_date(date(2026, 9, 5), cx), "Sep 5, 2026");
            assert_eq!(show_month(date(2026, 9, 5), cx), "September 2026");
            assert_eq!(show_time(time(9, 5, 0, 0)), "09:05");
            assert_eq!(
                spans(cx),
                [
                    "Sep 3 \u{2013} 17, 2026",
                    "Sep 28 \u{2013} Oct 2, 2026",
                    "Dec 30, 2026 \u{2013} Jan 2, 2027",
                ]
            );
        });
    }

    #[gpui::test]
    fn chinese_dates_read_as_chinese(cx: &mut TestAppContext) {
        cx.update(|cx| {
            cx.set_global(I18n::new("zh-CN"));
            assert_eq!(show_date(date(2026, 9, 5), cx), "2026年9月5日");
            assert_eq!(show_month(date(2026, 9, 5), cx), "2026年9月");
            assert_eq!(
                spans(cx),
                [
                    "2026年9月3日–17日",
                    "2026年9月28日–10月2日",
                    "2026年12月30日–2027年1月2日",
                ]
            );
        });
    }
}
