mod measures;
mod media;
mod people;
mod records;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 12,
    slug: "data",
    title: "Data Display",
    summary: "People, numbers and records, shown plainly.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("badge-more"),
    Step::Wait(200),
    Step::Shot("count-rolling"),
    Step::Hover("avatar-upload"),
    Step::Wait(300),
    Step::Shot("avatar-hover"),
    Step::Click("stats-next"),
    Step::Wait(300),
    Step::Shot("stats-next"),
    Step::Click("feed-post"),
    Step::Wait(300),
    Step::Shot("feed-arrival"),
    Step::Hover("carousel"),
    Step::Wait(300),
    Step::Shot("carousel-hover"),
    Step::DownAt("carousel", 530.0, 133.0),
    Step::UpAt("carousel", 530.0, 133.0),
    Step::Wait(600),
    Step::Shot("carousel-next"),
    Step::DownAt("gallery", 60.0, 60.0),
    Step::UpAt("gallery", 60.0, 60.0),
    Step::Wait(400),
    Step::Shot("gallery-lightbox"),
    Step::Key("escape"),
    Step::Wait(300),
    Step::DownAt("before-after", 240.0, 160.0),
    Step::DragTo("before-after", 390.0, 160.0),
    Step::UpAt("before-after", 390.0, 160.0),
    Step::Wait(200),
    Step::Shot("before-after-drag"),
    Step::Click("measures-next"),
    Step::Wait(400),
    Step::Shot("measures-next"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(people::badges(window, cx))
        .child(people::tags(window, cx))
        .child(people::avatars(window, cx))
        .child(people::groups(cx))
        .child(people::numbers(window, cx))
        .child(records::descriptions(cx))
        .child(records::properties(window, cx))
        .child(records::timelines(cx))
        .child(records::feed(window, cx))
        .child(records::changelog(cx))
        .child(media::carousel(cx))
        .child(media::gallery(cx))
        .child(media::before_after(cx))
        .child(media::watermark(window, cx))
        .child(measures::codes(cx))
        .child(measures::measures(window, cx))
        .child(measures::stars(cx))
        .child(measures::comparison(cx))
        .into_any_element()
}
