use ely_gpui_component::{
    forms::{Choice, Select},
    i18n::{I18n, LOCALES},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, Radius},
    typography::Ellipsis,
};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::{Timestamp, tz::TimeZone};

mod a11y;

use super::Page;
use crate::probe::probe;
use crate::step::Step;
use crate::ui::{section, specimen, specimens};

pub const PAGE: Page = Page {
    number: 40,
    slug: "i18n",
    title: "i18n & a11y",
    summary: "Messages, numbers and dates in the reader's locale. A right-to-left locale mirrors its stacks.",
    render,
    script: &[
        Step::Click("locale"),
        Step::Key("end"),
        Step::Key("enter"),
        Step::Wait(300),
        Step::Shot("right-to-left"),
        Step::Click("locale"),
        Step::Key("home"),
        Step::Key("enter"),
        Step::Wait(300),
        Step::Key("tab"),
        Step::HoverAt("skip-demo", 300.0, 70.0),
        Step::Wait(300),
        Step::Shot("skip-link"),
        Step::Key("enter"),
        Step::Key("tab"),
        Step::Wait(300),
        Step::Shot("skipped"),
        Step::Click("color-blind"),
        Step::Wait(300),
        Step::Shot("color-blind"),
        Step::Click("color-blind"),
        Step::Wait(300),
    ],
};

/// 1 September 2026, when the demo's folder was saved.
const SAVED: i64 = 1_788_220_800;

/// The gallery's messages, one catalog per listed locale.
pub fn install(cx: &mut App) {
    cx.set_global(
        I18n::new("en-US")
            .catalog(
                "en-US",
                &[
                    ("welcome", "Welcome back, {name}"),
                    ("files", "{count} files in {folder}"),
                    ("saved", "Last saved {date}"),
                    ("change", "{change}% since yesterday"),
                ],
            )
            .catalog(
                "en-GB",
                &[
                    ("welcome", "Welcome back, {name}"),
                    ("files", "{count} files in {folder}"),
                    ("saved", "Last saved {date}"),
                    ("change", "{change}% since yesterday"),
                ],
            )
            .catalog(
                "de-DE",
                &[
                    ("welcome", "Willkommen zurück, {name}"),
                    ("files", "{count} Dateien in {folder}"),
                    ("saved", "Zuletzt gespeichert am {date}"),
                    ("change", "{change} % seit gestern"),
                ],
            )
            .catalog(
                "fr-FR",
                &[
                    ("welcome", "Bon retour, {name}"),
                    ("files", "{count} fichiers dans {folder}"),
                    ("saved", "Enregistré le {date}"),
                    ("change", "{change}\u{202f}% depuis hier"),
                ],
            )
            .catalog(
                "ja-JP",
                &[
                    ("welcome", "おかえりなさい、{name}さん"),
                    ("files", "{folder}に{count}個のファイル"),
                    ("saved", "最終保存：{date}"),
                    ("change", "昨日から{change}%"),
                ],
            )
            .catalog(
                "zh-CN",
                &[
                    ("welcome", "欢迎回来，{name}"),
                    ("files", "{folder}中有{count}个文件"),
                    ("saved", "上次保存于{date}"),
                    ("change", "较昨日{change}%"),
                ],
            )
            .catalog(
                "he-IL",
                &[
                    ("welcome", "ברוך שובך, {name}"),
                    ("files", "{count} קבצים ב־{folder}"),
                    ("saved", "נשמר לאחרונה ב־{date}"),
                    ("change", "{change}% מאז אתמול"),
                ],
            ),
    );
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(messages(cx))
        .child(formats(cx))
        .child(mirrored(cx))
        .child(a11y::skip_link(window, cx))
        .child(a11y::color_blind(cx))
        .into_any_element()
}

fn messages(cx: &App) -> impl IntoElement + use<> {
    let i18n = cx.global::<I18n>();
    let locale = i18n.locale();
    let saved = Timestamp::from_second(SAVED).expect("a time");
    let lines = [
        i18n.text("welcome", &[("name", "Ada")]),
        i18n.text(
            "files",
            &[("count", &locale.number(1204.0, 0)), ("folder", "Docs")],
        ),
        i18n.text("saved", &[("date", &locale.date(saved, &TimeZone::UTC))]),
        i18n.text("change", &[("change", &locale.number(-3.5, 1))]),
    ];
    section(
        "I18nProvider · Trans / LocalizedText",
        "One locale for the app, a catalog of messages per locale. Every catalog holds the same keys; a missing key or argument fails.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "locale",
                probe(
                    "locale",
                    div().w(px(240.0)).child(
                        Select::new(
                            "locale",
                            LOCALES.map(|locale| Choice::new(locale.tag, locale.name)),
                        )
                        .selected(locale.tag)
                        .on_change(|tag, _, cx| I18n::set_locale(tag, cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "messages",
                locale.direction.align(
                    div()
                        .w(px(280.0))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(lines.map(Ellipsis::new)),
                ),
                cx,
            )),
    )
}

fn formats(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let saved = Timestamp::from_second(SAVED).expect("a time");
    let rows = LOCALES.map(|locale| {
        div()
            .flex()
            .flex_wrap()
            .gap_x_4()
            .child(
                div()
                    .w(px(96.0))
                    .text_color(theme.colors.fg_muted)
                    .child(locale.name),
            )
            .child(div().w(px(120.0)).child(locale.number(-1234567.891, 2)))
            .child(div().child(locale.date(saved, &TimeZone::UTC)))
    });
    section(
        "LocaleNumber / LocaleDate",
        "Each locale's separators and its short numeric date, from CLDR.",
        cx,
    )
    .child(div().flex().flex_col().gap_1().children(rows))
}

fn mirrored(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let direction = cx.global::<I18n>().locale().direction;
    let file = |name: &'static str, size: &'static str| {
        direction
            .row(div())
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .child(Icon::new(IconName::File).color(theme.colors.fg_muted))
            .child(direction.align(div().flex_1().min_w_0().child(Ellipsis::new(name))))
            .child(
                div()
                    .flex_none()
                    .text_color(theme.colors.fg_muted)
                    .child(size),
            )
    };
    section(
        "RTL layout",
        "A right-to-left locale starts its stacks and sets its text at the right; pick עברית above. Ely's other components stay left to right.",
        cx,
    )
    .child(
        div()
            .max_w(px(320.0))
            .flex()
            .flex_col()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(file("Docs", "12 KB"))
            .child(file("Photos", "3.1 MB")),
    )
}
