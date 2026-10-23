use ely_gpui_component::{
    buttons::Button,
    misc::{ConsentDialog, CookieBanner, LicenseViewer, Package},
    theme::ActiveTheme,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::probe::probe;
use crate::ui::section;

const TERMS: &str = "These terms cover the gallery's sample app. You may use it to try the components, read their code, and share what you find. You may not use it to harm anyone. The app keeps no data about you; it forgets your choices when it closes.";

/// What the product ships and under which license, read from the repository's own files.
fn packages() -> [Package; 4] {
    [
        Package::new(
            "Ely GPUI Component",
            "0.1.0",
            "MIT",
            include_str!("../../../../../LICENSE"),
        ),
        Package::new(
            "Lucide",
            "1.48.0",
            "ISC",
            include_str!("../../../../ely-core/icons/LICENSE"),
        ),
        Package::new(
            "Inter",
            "4.1",
            "OFL-1.1",
            include_str!("../../../../ely-core/assets/fonts/Inter-LICENSE.txt"),
        ),
        Package::new(
            "JetBrains Mono",
            "2.304",
            "OFL-1.1",
            include_str!("../../../../ely-core/assets/fonts/JetBrainsMono-LICENSE.txt"),
        ),
    ]
}

pub fn render(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = window.use_keyed_state("consent-open", cx, |_, _| false);
    let said = window.use_keyed_state("notices-said", cx, |_, _| SharedString::default());
    let answer = |word: &'static str| {
        let (open, said) = (open.clone(), said.clone());
        move |_: &mut Window, cx: &mut App| {
            open.update(cx, |open, cx| {
                *open = false;
                cx.notify();
            });
            said.update(cx, |said, cx| {
                *said = word.into();
                cx.notify();
            });
        }
    };
    let dialog = open.read(cx).then(|| {
        ConsentDialog::new(
            "consent",
            "Terms of use",
            TERMS,
            answer("Terms accepted."),
            answer("Terms declined."),
        )
    });
    let opener = open.clone();
    let chose = said.clone();
    div()
        .child(section(
            "ChangeLog Viewer",
            "ChangeLog Viewer is data_display::Changelog, shown on the Data Display page.",
            cx,
        ))
        .child(
            section(
                "LicenseViewer",
                "The packages a product ships, each with its license, beside the chosen one's text. These are Ely's own.",
                cx,
            )
            .child(div().w(px(560.0)).child(LicenseViewer::new("licenses", packages()))),
        )
        .child(
            section(
                "Terms / Consent Dialog / CookieBanner",
                "Terms to agree to before going on, and cookies to choose when the app shows web content. Each hands its answer to the owner.",
                cx,
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        probe(
                            "consent-open",
                            div().child(Button::new("consent-button", "Review the terms").on_click(
                                move |_, _, cx| {
                                    opener.update(cx, |open, cx| {
                                        *open = true;
                                        cx.notify();
                                    })
                                },
                            )),
                        ),
                    )
                    .child(probe(
                        "cookies",
                        div().w(px(360.0)).child(
                            CookieBanner::new(
                                "cookies",
                                "The help pages set cookies. Necessary ones keep them working; the rest are up to you.", move |kinds, _, cx| {
                                let words = match kinds.len() {
                                    0 => "Only necessary cookies.".to_string(),
                                    _ => format!("Cookies on: {}.", kinds.join(", ")),
                                };
                                chose.update(cx, |said, cx| {
                                    *said = words.into();
                                    cx.notify();
                                })
                            })
                            .kind("Analytics", "Counts which pages help, without names.")
                            .kind("Video", "Lets embedded tutorials play and remember where you stopped."),
                        ),
                    ))
                    .child(
                        div()
                            .text_color(cx.theme().colors.fg_muted)
                            .child(said.read(cx).clone()),
                    ),
            ),
        )
        .children(dialog)
}
