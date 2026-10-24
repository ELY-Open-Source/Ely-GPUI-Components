use std::rc::Rc;

use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, Role, SharedString, Window, prelude::*,
};

use super::{Shown, TreeNode, rows};
use crate::{
    forms::{Choice, Listing, field_button, field_text, listing},
    primitives::{Icon, IconName, tab_stop},
    theme::{ActiveTheme, ControlSize, IconSize},
};

type OnPick = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Every node of a tree as a choice, depth first, each indented under its parent.
fn choices(nodes: &[TreeNode]) -> Vec<Choice> {
    fn keys(nodes: &[TreeNode], out: &mut Vec<SharedString>) {
        for node in nodes {
            out.push(node.key.clone());
            if let super::Children::Known(children) = &node.children {
                keys(children, out);
            }
        }
    }
    let mut every = Vec::new();
    keys(nodes, &mut every);
    rows(nodes, &every.into_iter().collect())
        .into_iter()
        .filter_map(|row| match row.shown {
            Shown::Node {
                key, label, icon, ..
            } => {
                let choice = Choice::new(key, label).depth(row.depth);
                Some(match icon {
                    Some(icon) => choice.icon(icon),
                    None => choice,
                })
            }
            Shown::Loading => None,
        })
        .collect()
}

/// A select whose choices are a tree's nodes, indented under their parents; any node can be picked.
///
/// ```
/// use ely_core::lists::{TreeNode, TreeSelect};
///
/// let folder = TreeSelect::new("folder", [TreeNode::new("work", "Work").child(TreeNode::new("q4", "Q4"))])
///     .selected("q4")
///     .on_change(|_key, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct TreeSelect {
    id: ElementId,
    nodes: Vec<TreeNode>,
    selected: Option<SharedString>,
    placeholder: SharedString,
    on_change: Option<OnPick>,
}

impl TreeSelect {
    /// A select under `id` over the tree `nodes`.
    pub fn new(id: impl Into<ElementId>, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        Self {
            id: id.into(),
            nodes: nodes.into_iter().collect(),
            selected: None,
            placeholder: "Choose".into(),
            on_change: None,
        }
    }

    /// The key of the node chosen.
    pub fn selected(mut self, key: impl Into<SharedString>) -> Self {
        self.selected = Some(key.into());
        self
    }

    /// What it shows with nothing chosen; `Choose` unless set.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Runs with the key of the node picked.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TreeSelect {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rows = Rc::new(choices(&self.nodes));
        assert!(!rows.is_empty(), "tree select {:?} has no nodes", self.id);
        let focus = tab_stop((self.id.clone(), "focus").into(), true, window, cx);
        let shown = self
            .selected
            .as_ref()
            .and_then(|key| rows.iter().find(|row| row.value == *key))
            .cloned();
        let colors = &cx.theme().colors;
        let trigger = field_button(self.id.clone(), &focus, ControlSize::Md, false, window, cx)
            .role(Role::ComboBox)
            .aria_label(self.placeholder.clone())
            .when_some(shown.as_ref(), |trigger, row| {
                trigger.aria_value(row.label.clone())
            })
            .when_some(shown.as_ref().and_then(|row| row.icon), |trigger, icon| {
                trigger.child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_muted))
            })
            .child(field_text(
                shown.map(|row| row.label),
                self.placeholder,
                false,
                cx,
            ))
            .child(
                Icon::new(IconName::ChevronsUpDown)
                    .size(IconSize::Xs)
                    .color(colors.fg_subtle),
            );
        let (id, on_change) = (self.id.clone(), self.on_change);
        let list = Listing {
            id: &self.id,
            rows,
            selected: self.selected.as_ref(),
            focused: focus.is_focused(window),
        };
        listing(
            list,
            trigger,
            move |key, window, cx| {
                log::info!("tree select {id:?}: {key}");
                if let Some(on_change) = &on_change {
                    on_change(key, window, cx);
                }
            },
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{TreeNode, choices};

    #[test]
    fn every_node_becomes_a_choice_at_its_depth() {
        let nodes = [
            TreeNode::new("design", "Design")
                .children([TreeNode::new("brand", "Brand"), TreeNode::new("web", "Web")]),
            TreeNode::new("ops", "Operations").pending(),
        ];
        let shown: Vec<_> = choices(&nodes)
            .into_iter()
            .map(|choice| (choice.value.to_string(), choice.depth))
            .collect();
        assert_eq!(
            shown,
            [
                ("design".into(), 0),
                ("brand".into(), 1),
                ("web".into(), 1),
                ("ops".into(), 0)
            ]
        );
    }
}

crate::capable!(TreeSelect: change(&SharedString));
