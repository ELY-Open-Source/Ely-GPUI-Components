use ely_gpui_component::{
    Assets,
    buttons::Button,
    motion::{self, Entrance, Flash, Spinner, Transition},
    primitives::FocusScope,
    theme::{ActiveTheme, Radius, Theme},
};
use gpui::{
    Animation, AnimationExt, App, AppContext, Context, FocusHandle, IntoElement, ParentElement,
    Render, Styled, Window, WindowOptions, div,
};

struct Moves {
    focus: FocusHandle,
    shown: bool,
    count: usize,
}

impl Render for Moves {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let still = theme.reduced_motion;
        let (toggle, add) = (cx.entity(), cx.entity());
        let dot = div()
            .size_4()
            .rounded(theme.radius(Radius::Xl))
            .bg(theme.colors.accent);
        let dot = if still {
            dot.into_any_element()
        } else {
            dot.with_animation(
                "breath",
                Animation::new(motion::duration(motion::SLOW * 4, cx))
                    .repeat()
                    .with_easing(motion::ease_in_out_cubic),
                |dot, t| dot.opacity(0.4 + 0.6 * (1.0 - (2.0 * t - 1.0).abs())),
            )
            .into_any_element()
        };
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("show", if self.shown { "Hide" } else { "Show" }).on_click(
                            move |_, _, cx| {
                                toggle.update(cx, |moves, cx| {
                                    moves.shown = !moves.shown;
                                    cx.notify();
                                })
                            },
                        ),
                    )
                    .child(Button::new("add", "Add one").on_click(move |_, _, cx| {
                        add.update(cx, |moves, cx| {
                            moves.count += 1;
                            cx.notify();
                        })
                    }))
                    .child(
                        Button::new(
                            "still",
                            if still {
                                "Allow motion"
                            } else {
                                "Reduce motion"
                            },
                        )
                        .on_click(move |_, _, cx| {
                            Theme::update(cx, |theme| theme.reduced_motion = !still)
                        }),
                    ),
            )
            .child(
                Transition::new("panel", self.shown)
                    .entrance(Entrance::Rise)
                    .child("A panel that rises in and fades out."),
            )
            .child(
                Flash::new("count", self.count)
                    .p_2()
                    .child(format!("Count: {}", self.count)),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(Spinner::new("spinner"))
                    .child(dot),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Moves {
                        focus,
                        shown: true,
                        count: 0,
                    }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
