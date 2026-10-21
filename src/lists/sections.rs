use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::{ListItem, SelectableList};
use crate::{
    forms::OnValue,
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme, IconSize, Radius, TextSize},
};

/// Gets a section's title and whether it is now folded.
pub(crate) type OnFold = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;

struct Section {
    title: SharedString,
    rows: Vec<(SharedString, ListItem)>,
    badge: Option<AnyElement>,
}

/// Lists under titles down a column, each a `SelectableList` keyed by its title under the id: Tab moves between them, and the chosen key is lit wherever it sits. A press or an arrow selects, Enter or a double press activates. Owners check their titles and keys.
#[derive(IntoElement)]
pub(crate) struct Sections {
    id: ElementId,
    sections: Vec<Section>,
    counted: bool,
    selected: Option<SharedString>,
    on_select: Option<OnValue>,
    on_activate: Option<OnValue>,
    fold: Option<(Vec<SharedString>, OnFold)>,
}

impl Sections {
    pub(crate) fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sections: Vec::new(),
            counted: false,
            selected: None,
            on_select: None,
            on_activate: None,
            fold: None,
        }
    }

    pub(crate) fn section(
        mut self,
        title: SharedString,
        rows: Vec<(SharedString, ListItem)>,
    ) -> Self {
        self.sections.push(Section {
            title,
            rows,
            badge: None,
        });
        self
    }

    /// Something beside the last section's title.
    pub(crate) fn badge(mut self, badge: AnyElement) -> Self {
        let last = self.sections.last_mut().expect("a badge follows a section");
        last.badge = Some(badge);
        self
    }

    /// Shows how many rows each section holds beside its title.
    pub(crate) fn counted(mut self) -> Self {
        self.counted = true;
        self
    }

    /// Titles fold; `folded` names those folded now.
    pub(crate) fn foldable(mut self, folded: Vec<SharedString>, on_fold: OnFold) -> Self {
        self.fold = Some((folded, on_fold));
        self
    }

    pub(crate) fn selected(mut self, key: Option<SharedString>) -> Self {
        self.selected = key;
        self
    }

    pub(crate) fn on_select(mut self, handler: OnValue) -> Self {
        self.on_select = Some(handler);
        self
    }

    pub(crate) fn on_activate(mut self, handler: OnValue) -> Self {
        self.on_activate = Some(handler);
        self
    }
}

impl RenderOnce for Sections {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let sections = self.sections.into_iter().map(|section| {
            let Section { title, rows, badge } = section;
            let count = rows.len();
            let folded = (self.fold.as_ref()).is_some_and(|(folded, _)| folded.contains(&title));
            let words = div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.fg_muted)
                .child(title.clone());
            let head = div()
                .px_3()
                .flex()
                .items_center()
                .gap_2()
                .text_size(theme.text_size(TextSize::Xs));
            let head = match &self.fold {
                None => head.child(words).into_any_element(),
                Some((_, on_fold)) => {
                    let (on_fold, named, folding) = (on_fold.clone(), title.clone(), title.clone());
                    let chevron = if folded {
                        IconName::ChevronRight
                    } else {
                        IconName::ChevronDown
                    };
                    head.id((self.id.clone(), format!("fold-{title}")))
                        .debug_selector(move || format!("fold {named}"))
                        .role(Role::Button)
                        .aria_label(title.clone())
                        .aria_expanded(!folded)
                        .gap_1()
                        .rounded(theme.radius(Radius::Sm))
                        .border_1()
                        .border_color(gpui::transparent_black())
                        .tab_index(0)
                        .focus_ring(cx)
                        .cursor_pointer()
                        .hover(|style| style.bg(colors.hover))
                        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                        .on_click(move |_, window, cx| on_fold(&folding, !folded, window, cx))
                        .child(
                            Icon::new(chevron)
                                .size(IconSize::Xs)
                                .color(colors.fg_subtle),
                        )
                        .child(words)
                        .into_any_element()
                }
            };
            let head = div()
                .flex()
                .items_center()
                .justify_between()
                .pr_3()
                .child(div().flex().items_baseline().gap_2().child(head).children(
                    self.counted.then(|| {
                        div()
                            .text_size(theme.text_size(TextSize::Xs))
                            .text_color(colors.fg_subtle)
                            .child(count.to_string())
                    }),
                ))
                .children(badge);
            let lit = self
                .selected
                .clone()
                .filter(|key| rows.iter().any(|(each, _)| each == key));
            let list = (!folded).then(|| {
                let (select, activate) = (self.on_select.clone(), self.on_activate.clone());
                rows.into_iter()
                    .fold(
                        SelectableList::new((self.id.clone(), format!("section-{title}"))),
                        |list, (key, row)| list.row(key, row),
                    )
                    .selected(lit)
                    .on_change(move |keys, window, cx| {
                        let key = keys.first().expect("a pick names a row");
                        if let Some(select) = &select {
                            select(key, window, cx);
                        }
                    })
                    .on_activate(move |key, window, cx| {
                        if let Some(activate) = &activate {
                            activate(key, window, cx);
                        }
                    })
            });
            div().flex().flex_col().gap_1().child(head).children(list)
        });
        div().flex().flex_col().gap_4().children(sections)
    }
}
