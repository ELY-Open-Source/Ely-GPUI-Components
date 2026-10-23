use gpui::{Context, IntoElement, Render, TestAppContext, Window, black};

use super::{FoundIcons, Icon, IconName, Source, found};
use crate::theme::{ActiveTheme, Theme};

#[test]
fn every_icon_carries_its_own_lucide_svg() {
    for icon in IconName::ALL {
        let svg = std::str::from_utf8(icon.svg()).expect("an SVG is UTF-8");
        assert!(
            svg.contains(&format!("class=\"lucide lucide-{}\"", icon.name())),
            "{icon:?} carries another icon's file"
        );
    }
}

#[test]
fn an_app_path_draws_as_given_and_a_name_as_its_bundled_svg() {
    let custom = Icon::from_path("app-icons/mark.svg");
    assert_eq!(
        custom.source,
        Source::App {
            path: "app-icons/mark.svg".into(),
            broken: IconName::ImageOff
        }
    );
    let named = Icon::new(IconName::Check);
    assert_eq!(named.source, Source::Bundled(IconName::Check));
}

#[test]
fn the_source_answers_whether_it_holds_a_path() {
    assert!(found(
        &crate::Assets,
        "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"
    ));
    assert!(!found(&crate::Assets, "app-icons/missing.svg"));
}

struct Missing;

impl Render for Missing {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Icon::from_path("app-icons/missing.svg")
    }
}

#[gpui::test]
fn a_missing_app_icon_draws_its_mark(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Missing);
    cx.run_until_parked();
    let seen = cx.update(|_, cx| cx.global::<FoundIcons>().0.clone());
    assert_eq!(seen.get("app-icons/missing.svg"), Some(&false));
}

#[gpui::test]
fn a_broken_icon_shows_a_broken_picture_in_danger(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        let icon = Icon::from_path("app-icons/missing.svg")
            .color(black())
            .group_hover_color("row", black())
            .broken(IconName::ImageOff, cx);
        assert_eq!(icon.source, Source::Bundled(IconName::ImageOff));
        assert_eq!(icon.color, Some(cx.theme().colors.danger));
        assert!(icon.hover.is_none(), "hover keeps the danger color");
    });
}

#[gpui::test]
fn init_without_ely_assets_returns_an_error(cx: &mut TestAppContext) {
    let error = cx
        .update(crate::init)
        .expect_err("the test source holds no Ely assets");
    assert!(format!("{error:#}").contains("pass `ely_core::Assets`"));
}
