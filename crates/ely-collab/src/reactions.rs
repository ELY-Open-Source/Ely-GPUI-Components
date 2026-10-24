use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};

use crate::{
    buttons::{ButtonVariant, IconButton},
    forms::{EmojiPicker, OnValue, Run},
    i18n,
    overlays::Popover,
    primitives::{FocusRing, IconName},
    theme::{ActiveTheme, ControlSize, Radius, TextSize},
    typography::tabular,
};

/// Emoji most reactions use, offered before the full picker.
const QUICK: [&str; 6] = ["👍", "❤️", "🎉", "😄", "👀", "🙏"];

/// A reaction on a comment: its emoji, how many gave it, and whether you did.
#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// Its emoji.
    pub emoji: SharedString,
    /// How many gave it.
    pub count: usize,
    /// Whether you did.
    pub mine: bool,
}

/// `reactions` after you toggle `emoji`: yours comes off, or goes on, a new one last.
pub fn toggled(reactions: &[Reaction], emoji: &str) -> Vec<Reaction> {
    let mut next: Vec<Reaction> = reactions.to_vec();
    match next
        .iter()
        .position(|reaction| reaction.emoji.as_ref() == emoji)
    {
        Some(ix) if next[ix].mine => {
            if next[ix].count == 0 {
                log::error!("reactions: {emoji} counts none yet is yours; it comes off");
            }
            next[ix].count = next[ix].count.saturating_sub(1);
            next[ix].mine = false;
            if next[ix].count == 0 {
                next.remove(ix);
            }
        }
        Some(ix) => {
            next[ix].count += 1;
            next[ix].mine = true;
        }
        None => next.push(Reaction {
            emoji: emoji.to_string().into(),
            count: 1,
            mine: true,
        }),
    }
    next
}

/// Picks an emoji to react with: a few most used, then every one.
///
/// ```
/// use ely_collab::ReactionPicker;
///
/// let pick = ReactionPicker::new("pick", |_emoji, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ReactionPicker {
    id: ElementId,
    on_pick: OnValue,
}

impl ReactionPicker {
    /// A picker under `id`; `on_pick` takes the emoji picked.
    pub fn new(
        id: impl Into<ElementId>,
        on_pick: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            on_pick: Rc::new(on_pick),
        }
    }
}

impl RenderOnce for ReactionPicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (id, on_pick) = (self.id.clone(), self.on_pick);
        let (trigger, tip) = (
            (self.id.clone(), "trigger"),
            i18n::text(cx, "reactions.add", &[]),
        );
        let opener = move |toggle: Run| {
            IconButton::new(trigger, IconName::SmilePlus)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip(tip)
                .on_click(move |_, window, cx| toggle(window, cx))
        };
        Popover::with_opener(self.id, opener, move |_, _, cx| {
            let theme = cx.theme();
            let colors = theme.colors.clone();
            let (quick, all) = (on_pick.clone(), on_pick.clone());
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().flex().gap_1().children(QUICK.iter().map(|emoji| {
                    let (pick, emoji) = (quick.clone(), SharedString::from(*emoji));
                    let label = emoji.clone();
                    div()
                        .id((id.clone(), format!("quick-{emoji}")))
                        .p_1()
                        .rounded(theme.radius(Radius::Sm))
                        .cursor_pointer()
                        .text_size(theme.text_size(TextSize::Lg))
                        .hover(|face| face.bg(colors.hover))
                        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                        .on_click(move |_, window, cx| pick(&emoji, window, cx))
                        .child(label)
                })))
                .child(
                    EmojiPicker::new((id.clone(), "all")).on_change(move |glyph, window, cx| {
                        all(&SharedString::from(glyph.to_string()), window, cx)
                    }),
                )
        })
    }
}

/// Reactions under a comment: each emoji with its count, yours outlined; a press toggles yours, and the picker adds another.
///
/// ```
/// use ely_collab::{Reaction, Reactions};
///
/// let reactions = Reactions::new("reactions", [Reaction { emoji: "👍".into(), count: 3, mine: true }])
///     .on_toggle(|_emoji, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct Reactions {
    id: ElementId,
    reactions: Vec<Reaction>,
    on_toggle: Option<OnValue>,
}

impl Reactions {
    /// Reactions under `id`, in order.
    pub fn new(id: impl Into<ElementId>, reactions: impl IntoIterator<Item = Reaction>) -> Self {
        Self {
            id: id.into(),
            reactions: reactions.into_iter().collect(),
            on_toggle: None,
        }
    }

    /// Gets the emoji to toggle, from a chip or the picker.
    pub fn on_toggle(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Reactions {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .children(self.reactions.iter().map(|reaction| {
                let toggle = self.on_toggle.clone();
                let emoji = reaction.emoji.clone();
                div()
                    .id((self.id.clone(), format!("reaction-{emoji}")))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .py_0p5()
                    .rounded_full()
                    .border_1()
                    .border_color(if reaction.mine {
                        colors.focus
                    } else {
                        colors.border
                    })
                    .when(reaction.mine, |chip| chip.bg(colors.focus.opacity(0.1)))
                    .role(Role::Button)
                    .aria_label(format!("{} {}", reaction.emoji, reaction.count))
                    .aria_toggled(reaction.mine.into())
                    .tab_index(0)
                    .focus_ring(cx)
                    .cursor_pointer()
                    .text_size(theme.text_size(TextSize::Xs))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .when_some(toggle, |chip, toggle| {
                        chip.on_click(move |_, window, cx| {
                            log::info!("reactions: toggle {emoji}");
                            toggle(&emoji, window, cx)
                        })
                    })
                    .child(reaction.emoji.clone())
                    .child(
                        tabular(div().text_color(colors.fg_muted))
                            .child(format!("{}", reaction.count)),
                    )
            }))
            .children(self.on_toggle.map(|toggle| {
                ReactionPicker::new((self.id.clone(), "add"), move |emoji, window, cx| {
                    toggle(emoji, window, cx)
                })
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yours_at_no_count_comes_off() {
        assert_eq!(toggled(&[reaction("👍", 0, true)], "👍"), []);
    }

    fn reaction(emoji: &str, count: usize, mine: bool) -> Reaction {
        Reaction {
            emoji: emoji.to_string().into(),
            count,
            mine,
        }
    }

    #[test]
    fn a_toggle_adds_yours_or_takes_it_off() {
        let start = [reaction("👍", 2, false), reaction("🎉", 1, true)];
        assert_eq!(
            toggled(&start, "👍"),
            [reaction("👍", 3, true), reaction("🎉", 1, true)]
        );
        assert_eq!(
            toggled(&start, "🎉"),
            [reaction("👍", 2, false)],
            "the last one goes"
        );
        assert_eq!(toggled(&start, "👀").last(), Some(&reaction("👀", 1, true)));
    }
}
