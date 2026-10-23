use ely_gpui_component::{
    buttons::{ButtonVariant, IconButton},
    feedback::Alert,
    layout::on_axis,
    primitives::{FocusScope, IconName, Severity},
    theme::{ActiveTheme, ControlSize, Mode, Radius, TextSize, Theme},
};
use gpui::{
    AnyElement, App, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Subscription, Window, div, point, prelude::*, px,
};

use crate::{Start, pages, ui::Story};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Choice {
    #[default]
    Light,
    Dark,
    System,
}

pub struct Gallery {
    page: usize,
    /// The one section drawn alone, full-bleed, if any.
    story: Option<SharedString>,
    /// Why nothing can be shown, in place of the page.
    failure: Option<SharedString>,
    /// Counts fresh starts; the page's state lives under it.
    pass: usize,
    choice: Choice,
    focus: FocusHandle,
    scroll: ScrollHandle,
    /// Narrow check width, if any.
    narrow: Option<Pixels>,
    _appearance: Subscription,
}

impl Gallery {
    pub fn new(start: Start, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let appearance = cx.observe_window_appearance(window, |gallery, window, cx| {
            if gallery.choice == Choice::System {
                Theme::set_mode(window.appearance().into(), cx);
            }
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            page: start.page,
            story: start.story,
            failure: start.failure,
            pass: 0,
            choice: start.choice,
            focus,
            scroll: ScrollHandle::new(),
            narrow: start.narrow.map(px),
            _appearance: appearance,
        }
    }

    pub fn select(&mut self, page: usize, cx: &mut Context<Self>) {
        log::info!("gallery: page -> {}", pages::ALL[page].slug);
        self.page = page;
        self.scroll.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    }

    /// Drops the page's state, so it draws as new.
    #[cfg(not(target_family = "wasm"))]
    pub fn fresh(&mut self, cx: &mut Context<Self>) {
        self.pass += 1;
        cx.notify();
    }

    /// Viewport height and furthest scroll, in pixels.
    #[cfg(not(target_family = "wasm"))]
    pub fn scroll_extent(&self) -> (Pixels, Pixels) {
        (self.scroll.bounds().size.height, self.scroll.max_offset().y)
    }

    /// Scrolls until `target` sits inside the page. True if it moved.
    #[cfg(not(target_family = "wasm"))]
    pub fn reveal(&mut self, target: gpui::Bounds<Pixels>, cx: &mut Context<Self>) -> bool {
        let (view, margin) = (self.scroll.bounds(), px(96.0));
        let shift = if target.top() < view.top() + margin {
            target.top() - view.top() - margin
        } else if target.bottom() > view.bottom() - margin {
            target.bottom() - view.bottom() + margin
        } else {
            return false;
        };
        let y = (-self.scroll.offset().y + shift).clamp(px(0.0), self.scroll.max_offset().y);
        self.scroll_to(y, cx);
        true
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn scroll_to(&mut self, y: Pixels, cx: &mut Context<Self>) {
        self.scroll.set_offset(point(px(0.0), -y));
        cx.notify();
    }

    pub fn choose(&mut self, choice: Choice, window: &mut Window, cx: &mut Context<Self>) {
        self.choice = choice;
        let mode = match choice {
            Choice::Light => Mode::Light,
            Choice::Dark => Mode::Dark,
            Choice::System => window.appearance().into(),
        };
        Theme::set_mode(mode, cx);
        cx.notify();
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = cx.theme();
        let colors = &theme.colors;
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(248.0))
            .h_full()
            .pt(px(56.0))
            .px_3()
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .px_3()
                    .pb_6()
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Lg))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.fg)
                            .child("Ely"),
                    )
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Sm))
                            .text_color(colors.fg_subtle)
                            .child("GPUI Component"),
                    ),
            )
            .child(
                on_axis(div().id("nav"))
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .overflow_y_scroll()
                    .children(pages::ALL.iter().enumerate().map(|(ix, page)| {
                        let selected = ix == self.page;
                        div()
                            .id(page.slug)
                            .flex()
                            .items_center()
                            .gap_3()
                            .h(px(30.0))
                            .px_3()
                            .rounded(theme.radius(Radius::Md))
                            .cursor_pointer()
                            .text_size(theme.text_size(TextSize::Base))
                            .text_color(if selected { colors.fg } else { colors.fg_muted })
                            .when(selected, |el| el.bg(colors.hover))
                            .hover(|style| style.bg(colors.hover).text_color(colors.fg))
                            .child(
                                div()
                                    .w(px(18.0))
                                    .font_family(theme.mono_family.clone())
                                    .text_size(theme.text_size(TextSize::Xs))
                                    .text_color(colors.fg_subtle)
                                    .child(SharedString::from(format!("{:02}", page.number))),
                            )
                            .child(page.title)
                            .on_click(cx.listener(move |gallery, _, _, cx| gallery.select(ix, cx)))
                    })),
            )
    }

    fn switcher(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let options = [
            (Choice::Light, IconName::Sun, "theme-light"),
            (Choice::Dark, IconName::Moon, "theme-dark"),
            (Choice::System, IconName::Monitor, "theme-system"),
        ];
        let colors = &cx.theme().colors;
        div()
            .flex_none()
            .flex()
            .gap_0p5()
            .p_0p5()
            .rounded(cx.theme().radius(Radius::Lg))
            .border_1()
            .border_color(colors.border)
            .children(options.map(|(choice, icon, id)| {
                let variant = if choice == self.choice {
                    ButtonVariant::Subtle
                } else {
                    ButtonVariant::Ghost
                };
                IconButton::new(id, icon)
                    .variant(variant)
                    .size(ControlSize::Sm)
                    .on_click(
                        cx.listener(move |gallery, _, window, cx| {
                            gallery.choose(choice, window, cx)
                        }),
                    )
            }))
    }
}

impl Gallery {
    /// Header and body; a padded card when narrow.
    fn card(&self, header: impl IntoElement, body: AnyElement, cx: &App) -> AnyElement {
        let content = div().child(header).child(body);
        match self.narrow {
            // Width plus padding and hairlines.
            Some(width) => div()
                .w(width + px(34.0))
                .p_4()
                .border_1()
                .border_color(cx.theme().colors.border)
                .child(content)
                .into_any_element(),
            None => content.into_any_element(),
        }
    }

    /// The window's root: the focus scope over Ely's ground.
    fn root(&self, cx: &App) -> FocusScope {
        let theme = cx.theme();
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .font_family(theme.font_family.clone())
            .text_size(theme.text_size(TextSize::Base))
    }

    /// The page's demos, the story's alone, or why not.
    fn body(
        &mut self,
        page: &pages::Page,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<AnyElement, SharedString> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        if let Some(story) = &self.story {
            cx.set_global(Story::new(story.clone()));
        }
        let body = window.with_id(("pass", self.pass), |window| (page.render)(window, cx));
        if self.story.is_none() {
            return Ok(body);
        }
        match cx.remove_global::<Story>().missing(page.slug) {
            None => Ok(body),
            Some(missing) => {
                log::error!("gallery: {missing}");
                let failure = SharedString::from(missing);
                self.failure = Some(failure.clone());
                Err(failure)
            }
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = &pages::ALL[self.page];
        let body = match self.body(page, window, cx) {
            Ok(body) => body,
            Err(failure) => {
                return self.root(cx).child(
                    div().w_full().p_6().child(
                        Alert::new("gallery-failure", Severity::Danger, "Nothing to show")
                            .body(failure),
                    ),
                );
            }
        };
        if self.story.is_some() {
            return self.root(cx).child(
                on_axis(div().id(("story", self.pass)))
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&self.scroll)
                    // The widest demo, 900, plus padding; narrower, it scrolls sideways.
                    .child(
                        div()
                            .min_w(px(948.0))
                            .p_6()
                            // Natively the traffic lights take the top 56.
                            .when(cfg!(not(target_family = "wasm")), |story| {
                                story.pt(px(56.0))
                            })
                            .child(body),
                    ),
            );
        }
        let sidebar = self.sidebar(cx);
        let switcher = self.switcher(cx);
        let theme = cx.theme();
        let colors = &theme.colors;

        self.root(cx).child(sidebar).child(
            on_axis(div().id(("page", self.pass)))
                .flex_1()
                .h_full()
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(
                    div()
                        .max_w(px(960.0))
                        .px(px(56.0))
                        .pt(px(56.0))
                        .pb(px(96.0))
                        .child(
                            self.card(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_start()
                                    .justify_between()
                                    .gap_6()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(200.0))
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(theme.text_size(TextSize::Xxl))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(page.title),
                                            )
                                            .child(
                                                div()
                                                    .text_size(theme.text_size(TextSize::Md))
                                                    .text_color(colors.fg_muted)
                                                    .child(page.summary),
                                            ),
                                    )
                                    .child(switcher),
                                body,
                                cx,
                            ),
                        ),
                ),
        )
    }
}
