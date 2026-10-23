use std::{collections::HashSet, rc::Rc};

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    ScrollStrategy, SharedString, StatefulInteractiveElement, Styled, UniformListScrollHandle,
    Window, div, uniform_list,
};
use jiff::tz::TimeZone;

use super::item::{Mail, MailItem};
use crate::{
    forms::OnValues,
    i18n,
    lists::Pick,
    primitives::tab_stop,
    theme::{ActiveTheme, TextSize},
    typography::format,
};

type OnStar = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;
type Picker = Rc<dyn Fn(usize, Pick, &mut Window, &mut App)>;

/// The row the keyboard is on and where a Shift range starts, by index and by key so they follow their rows; the scroll; the owner's last selection and the one the list last sent; the list last checked and its keys.
#[derive(Default)]
struct Cursor {
    at: usize,
    anchor: usize,
    keys: Option<(SharedString, SharedString)>,
    scroll: UniformListScrollHandle,
    seen: Option<Vec<SharedString>>,
    sent: Option<Vec<SharedString>>,
    checked: usize,
    listed: HashSet<SharedString>,
}

/// The selection after picking row `at`, a range reaching back to `anchor`; in list order.
fn picked(
    mails: &[Mail],
    selected: &HashSet<SharedString>,
    anchor: usize,
    at: usize,
    pick: Pick,
) -> Vec<SharedString> {
    let (low, high) = (anchor.min(at), anchor.max(at));
    mails
        .iter()
        .enumerate()
        .filter(|(ix, mail)| match pick {
            Pick::One => *ix == at,
            Pick::Toggle => selected.contains(&mail.key) != (*ix == at),
            Pick::Range => (low..=high).contains(ix),
            Pick::All => true,
        })
        .map(|(_, mail)| mail.key.clone())
        .collect()
}

/// Where `key` is now; `hint` first, since rows rarely move.
fn place(mails: &[Mail], key: Option<&SharedString>, hint: usize) -> usize {
    let last = mails.len().saturating_sub(1);
    match key {
        Some(key) if mails.get(hint).is_some_and(|mail| mail.key == *key) => hint,
        Some(key) => mails
            .iter()
            .position(|mail| mail.key == *key)
            .unwrap_or(hint.min(last)),
        None => hint.min(last),
    }
}

/// Messages in a list, in the order given, drawing only the rows in view; it fills its box. A press or an arrow selects one, and Command or Shift with a press takes several; Shift with an arrow extends, Space toggles, Command-A takes all. Unread ones stand out.
#[derive(IntoElement)]
pub struct MailList {
    id: ElementId,
    mails: Rc<Vec<Mail>>,
    selected: Vec<SharedString>,
    zone: Option<TimeZone>,
    on_select: Option<OnValues>,
    on_star: Option<OnStar>,
}

impl MailList {
    /// Each key once; a list is checked when it first draws.
    pub fn new(id: impl Into<ElementId>, mails: Rc<Vec<Mail>>) -> Self {
        Self {
            id: id.into(),
            mails,
            selected: Vec::new(),
            zone: None,
            on_select: None,
            on_star: None,
        }
    }

    /// The messages selected now, by key.
    pub fn selected(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = keys.into_iter().map(Into::into).collect();
        self
    }

    /// The zone dates read in; the system's otherwise.
    pub fn zone(mut self, zone: TimeZone) -> Self {
        self.zone = Some(zone);
        self
    }

    /// Gets the keys selected, in list order.
    pub fn on_select(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Gets a message's key and its star's new state.
    pub fn on_star(
        mut self,
        handler: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_star = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MailList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mails = self.mails;
        let count = mails.len();
        if count == 0 {
            let theme = cx.theme();
            return div()
                .debug_selector(|| "mail-none".into())
                .px_3()
                .py_4()
                .text_size(theme.text_size(TextSize::Sm))
                .text_color(theme.colors.fg_muted)
                .child(i18n::text(cx, "mail.none", &[]))
                .into_any_element();
        }
        let cursor =
            window.use_keyed_state((self.id.clone(), "cursor"), cx, |_, _| Cursor::default());
        let list = Rc::as_ptr(&mails) as usize;
        if cursor.read(cx).checked != list {
            let mut listed = HashSet::with_capacity(count);
            for mail in mails.iter() {
                assert!(listed.insert(mail.key.clone()), "mail {} twice", mail.key);
            }
            cursor.update(cx, |cursor, _| {
                (cursor.checked, cursor.listed) = (list, listed)
            });
        }
        if cursor.read(cx).seen.as_ref() != Some(&self.selected) {
            cursor.update(cx, |cursor, _| {
                for key in self
                    .selected
                    .iter()
                    .filter(|key| !cursor.listed.contains(*key))
                {
                    log::error!("mail list: no mail {key}; it is not selected");
                }
                let echo = cursor.sent.take().as_ref() == Some(&self.selected);
                if let Some(first) = self.selected.first().filter(|_| !echo) {
                    cursor.keys = Some((first.clone(), first.clone()));
                }
                cursor.seen = Some(self.selected.clone());
            });
        }
        let (at, anchor, scroll) = {
            let cursor = cursor.read(cx);
            let held = cursor.keys.as_ref();
            (
                place(&mails, held.map(|keys| &keys.0), cursor.at),
                place(&mails, held.map(|keys| &keys.1), cursor.anchor),
                cursor.scroll.clone(),
            )
        };
        if (at, anchor) != (cursor.read(cx).at, cursor.read(cx).anchor) {
            cursor.update(cx, |cursor, _| (cursor.at, cursor.anchor) = (at, anchor));
        }
        let focus = tab_stop((self.id.clone(), "focus").into(), true, window, cx);
        let focused = focus.is_focused(window);
        let selected: Rc<HashSet<SharedString>> = Rc::new(self.selected.into_iter().collect());
        let pick: Picker = {
            let (id, mails, selected, cursor, on_select) = (
                self.id.clone(),
                mails.clone(),
                selected.clone(),
                cursor.clone(),
                self.on_select,
            );
            Rc::new(move |ix, pick, window, cx| {
                let next = picked(&mails, &selected, cursor.read(cx).anchor, ix, pick);
                cursor.update(cx, |cursor, cx| {
                    cursor.at = ix;
                    if pick != Pick::Range {
                        cursor.anchor = ix;
                    }
                    cursor.keys = Some((mails[ix].key.clone(), mails[cursor.anchor].key.clone()));
                    cursor.scroll.scroll_to_item(ix, ScrollStrategy::Nearest);
                    cursor.sent = Some(next.clone());
                    cx.notify();
                });
                log::info!("mail list {id:?}: {} selected", next.len());
                if let Some(on_select) = &on_select {
                    on_select(&next, window, cx);
                }
            })
        };
        let zone = self
            .zone
            .unwrap_or_else(|| format::system_zone("mail list"));
        let rows = {
            let (id, mails, pick, focus, on_star) = (
                self.id.clone(),
                mails.clone(),
                pick.clone(),
                focus.clone(),
                self.on_star,
            );
            uniform_list((self.id.clone(), "rows"), count, move |range, _, cx| {
                range
                    .map(|ix| {
                        let mail = mails[ix].clone();
                        let key = mail.key.clone();
                        let item = MailItem::new((id.clone(), format!("mail-{key}")), mail)
                            .zone(zone.clone());
                        let item = match on_star.clone() {
                            Some(star) => {
                                let starred = key.clone();
                                item.on_star(move |on, window, cx| star(&starred, on, window, cx))
                            }
                            None => item,
                        };
                        let (pick, focus) = (pick.clone(), focus.clone());
                        item.item(cx)
                            .selected(selected.contains(&key))
                            .place(ix + 1, count)
                            .current(focused && ix == at)
                            .on_click(move |event, window, cx| {
                                let held = event.modifiers();
                                let how = match (held.shift, held.platform) {
                                    (true, _) => Pick::Range,
                                    (false, true) => Pick::Toggle,
                                    _ => Pick::One,
                                };
                                window.focus(&focus, cx);
                                pick(ix, how, window, cx);
                            })
                    })
                    .collect()
            })
            .track_scroll(&scroll)
            .size_full()
        };
        div()
            .id(self.id)
            .debug_selector(|| "mail-list".into())
            .role(Role::List)
            .aria_label(i18n::text(cx, "mail.list", &[]))
            .track_focus(&focus)
            .size_full()
            .on_key_down(move |event, window, cx| {
                let held = &event.keystroke.modifiers;
                let at = cursor.read(cx).at.min(count - 1);
                let extend = if held.shift { Pick::Range } else { Pick::One };
                let (to, how) = match event.keystroke.key.as_str() {
                    "down" => (Some((at + 1).min(count - 1)), extend),
                    "up" => (Some(at.saturating_sub(1)), extend),
                    "home" => (Some(0), extend),
                    "end" => (Some(count - 1), extend),
                    "space" if !held.modified() => (Some(at), Pick::Toggle),
                    "a" if held.platform => (Some(at), Pick::All),
                    _ => return,
                };
                cx.stop_propagation();
                if let Some(to) = to {
                    pick(to, how, window, cx);
                }
            })
            .child(rows)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;

    fn mails(keys: &[&'static str]) -> Vec<Mail> {
        keys.iter()
            .map(|key| Mail::new(*key, "Ana", "Hi", Timestamp::UNIX_EPOCH))
            .collect()
    }

    fn set(keys: &[&'static str]) -> HashSet<SharedString> {
        keys.iter().map(|key| SharedString::from(*key)).collect()
    }

    #[test]
    fn picks_one_toggle_a_range_or_all_in_list_order() {
        let rows = mails(&["a", "b", "c", "d"]);
        let keys = |list: &[&'static str]| -> Vec<SharedString> {
            list.iter().map(|key| SharedString::from(*key)).collect()
        };
        assert_eq!(picked(&rows, &set(&["a"]), 0, 2, Pick::One), keys(&["c"]));
        assert_eq!(
            picked(&rows, &set(&["c", "a"]), 0, 1, Pick::Toggle),
            keys(&["a", "b", "c"])
        );
        assert_eq!(
            picked(&rows, &set(&["a", "c"]), 0, 0, Pick::Toggle),
            keys(&["c"])
        );
        assert_eq!(
            picked(&rows, &set(&[]), 3, 1, Pick::Range),
            keys(&["b", "c", "d"])
        );
        assert_eq!(picked(&rows, &set(&["b"]), 0, 0, Pick::All).len(), 4);
    }

    #[test]
    fn a_key_is_found_where_it_moved() {
        let rows = mails(&["a", "b", "c"]);
        assert_eq!(place(&rows, Some(&"b".into()), 1), 1);
        assert_eq!(place(&rows, Some(&"c".into()), 0), 2);
        assert_eq!(place(&rows, Some(&"gone".into()), 9), 2);
        assert_eq!(place(&rows, None, 1), 1);
    }
}
