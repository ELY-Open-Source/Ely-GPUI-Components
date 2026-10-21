use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, Point, RenderOnce,
    SharedString, Window, div,
};

use super::labels::hue_dot;
use crate::{
    data_display::CountBadge,
    forms::OnValue,
    lists::{ListItem, OnFold, Sections},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, IconSize},
};

/// What marks a mailbox: an icon, or a label's hue among the theme's chart colors.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mark {
    Icon(IconName),
    Hue(usize),
}

/// A mailbox, folder or label: its key, its name, its mark, and how many messages wait unread.
#[derive(Clone, Debug, PartialEq)]
pub struct Mailbox {
    pub key: SharedString,
    pub name: SharedString,
    mark: Mark,
    pub unread: u32,
}

impl Mailbox {
    pub fn new(
        key: impl Into<SharedString>,
        name: impl Into<SharedString>,
        icon: IconName,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            mark: Mark::Icon(icon),
            unread: 0,
        }
    }

    /// A label, marked by a dot in the theme's chart color at `hue`.
    pub fn label(key: impl Into<SharedString>, name: impl Into<SharedString>, hue: usize) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            mark: Mark::Hue(hue),
            unread: 0,
        }
    }

    pub fn unread(mut self, count: u32) -> Self {
        self.unread = count;
        self
    }

    fn item(&self, id: &ElementId, cx: &App) -> ListItem {
        let theme = cx.theme();
        let colors = &theme.colors;
        let mark = match self.mark {
            Mark::Icon(icon) => Icon::new(icon)
                .size(IconSize::Sm)
                .color(colors.fg_muted)
                .into_any_element(),
            Mark::Hue(hue) => hue_dot(&self.name, hue, cx).into_any_element(),
        };
        let row = ListItem::new((id.clone(), format!("box-{}", self.key)), self.name.clone())
            .leading(mark)
            .strong(self.unread > 0);
        match self.unread {
            0 => row,
            unread => row.trailing(
                CountBadge::new(
                    (id.clone(), format!("unread-{}", self.key)),
                    unread as usize,
                )
                .max(999),
            ),
        }
    }
}

type OnMenu = Rc<dyn Fn(&SharedString, Point<Pixels>, &mut Window, &mut App)>;

/// Mailboxes, folders and labels in sections down a column: a press, an arrow or Enter opens one, and the one open is lit. A box with mail unread stands out with its count.
#[derive(IntoElement)]
pub struct MailboxList {
    id: ElementId,
    sections: Vec<(SharedString, Vec<Mailbox>)>,
    open: Option<SharedString>,
    on_open: Option<OnValue>,
    folded: Vec<SharedString>,
    on_fold: Option<OnFold>,
    on_menu: Option<OnMenu>,
}

impl MailboxList {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sections: Vec::new(),
            open: None,
            on_open: None,
            folded: Vec::new(),
            on_fold: None,
            on_menu: None,
        }
    }

    /// A section under `title`; a title names one section, and a section holds a box.
    pub fn section(
        mut self,
        title: impl Into<SharedString>,
        boxes: impl IntoIterator<Item = Mailbox>,
    ) -> Self {
        let title = title.into();
        let boxes: Vec<Mailbox> = boxes.into_iter().collect();
        assert!(!boxes.is_empty(), "section {title} holds no mailbox");
        let named = self.sections.iter().any(|(other, _)| *other == title);
        assert!(!named, "section {title} twice");
        for (ix, mailbox) in boxes.iter().enumerate() {
            let seen = self
                .sections
                .iter()
                .flat_map(|(_, boxes)| boxes)
                .chain(&boxes[..ix])
                .any(|other| other.key == mailbox.key);
            assert!(!seen, "mailbox {} twice", mailbox.key);
        }
        self.sections.push((title, boxes));
        self
    }

    /// The box open now, by key.
    pub fn open(mut self, key: impl Into<SharedString>) -> Self {
        self.open = Some(key.into());
        self
    }

    /// The sections folded now, by title.
    pub fn folded(mut self, titles: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.folded = titles.into_iter().map(Into::into).collect();
        self
    }

    /// Titles fold; a folded section sums its unread.
    pub fn on_fold(
        mut self,
        handler: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_fold = Some(Rc::new(handler));
        self
    }

    /// Gets a box's key and where the right press fell.
    pub fn on_menu(
        mut self,
        handler: impl Fn(&SharedString, Point<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_menu = Some(Rc::new(handler));
        self
    }

    /// Gets the key of the box to open.
    pub fn on_open(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MailboxList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(open) = &self.open {
            let listed = self
                .sections
                .iter()
                .flat_map(|(_, boxes)| boxes)
                .any(|mailbox| mailbox.key == *open);
            if !listed {
                log::error!("mailbox list: no mailbox {open}; none open");
            }
        }
        let open: OnValue = {
            let (id, on_open) = (self.id.clone(), self.on_open.clone());
            Rc::new(move |key, window, cx| {
                log::info!("mailbox list {id}: open {key}");
                if let Some(on_open) = &on_open {
                    on_open(key, window, cx);
                }
            })
        };
        let sections = self.sections.iter().fold(
            Sections::new(self.id.clone()),
            |sections, (title, boxes)| {
                let rows = boxes
                    .iter()
                    .map(|mailbox| {
                        let item = mailbox.item(&self.id, cx);
                        let item = match self.on_menu.clone() {
                            Some(menu) => {
                                let key = mailbox.key.clone();
                                item.on_secondary(move |at, window, cx| menu(&key, at, window, cx))
                            }
                            None => item,
                        };
                        (mailbox.key.clone(), item)
                    })
                    .collect();
                let sections = sections.section(title.clone(), rows);
                let unread: u32 = boxes.iter().map(|mailbox| mailbox.unread).sum();
                match self.folded.contains(title) && unread > 0 {
                    true => {
                        let named = title.clone();
                        let badge = CountBadge::new(
                            (self.id.clone(), format!("folded-{title}")),
                            unread as usize,
                        )
                        .max(999);
                        let badge = div()
                            .debug_selector(move || format!("folded-unread {named}"))
                            .child(badge);
                        sections.badge(badge.into_any_element())
                    }
                    false => sections,
                }
            },
        );
        let sections = match self.on_fold.clone() {
            Some(on_fold) => sections.foldable(self.folded.clone(), on_fold),
            None => sections,
        };
        div().debug_selector(|| "mailbox-list".into()).child(
            sections
                .selected(self.open)
                .on_select(open.clone())
                .on_activate(open),
        )
    }
}
