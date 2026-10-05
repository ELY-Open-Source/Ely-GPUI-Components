use std::rc::Rc;

use gpui::{
    AnyElement, App, DefiniteLength, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::*,
};
use smallvec::SmallVec;

use super::chrome::{OnClose, WindowControls, controls_style, drag_region, route_close};
use crate::{
    layout::on_axis,
    theme::{ActiveTheme, ControlSize, Platform, TextSize},
    typography::Ellipsis,
};

/// One column of a `ColumnShell`: a header that drags the window, then its body.
pub struct ShellColumn {
    id: ElementId,
    width: Option<DefiniteLength>,
    leading: SmallVec<[AnyElement; 2]>,
    title: Option<SharedString>,
    actions: SmallVec<[AnyElement; 2]>,
    body: SmallVec<[AnyElement; 2]>,
}

impl ShellColumn {
    /// Fills the width the other columns leave.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: None,
            leading: SmallVec::new(),
            title: None,
            actions: SmallVec::new(),
            body: SmallVec::new(),
        }
    }

    /// A set width instead of the rest; it yields before a header's fixed parts.
    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Header content before the title.
    pub fn leading(mut self, item: impl IntoElement) -> Self {
        self.leading.push(item.into_any_element());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Header content at the end.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl ParentElement for ShellColumn {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

/// Columns to the window's top, each under a header; controls lead the first or end the last.
#[derive(IntoElement)]
pub struct ColumnShell {
    id: ElementId,
    platform: Option<Platform>,
    on_close: Option<OnClose>,
    columns: Vec<ShellColumn>,
}

impl ColumnShell {
    /// On macOS the system draws the traffic lights over the first column.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            platform: None,
            on_close: None,
            columns: Vec::new(),
        }
    }

    pub fn column(mut self, column: ShellColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Draws this platform's controls instead of the system's.
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Runs instead of closing; without a forced platform, system closes run it too.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ColumnShell {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.columns.len();
        assert!(count > 0, "column shell {:?} needs a column", self.id);
        if self.platform.is_none() {
            route_close(&self.id, self.on_close.clone(), window, cx);
        }
        let (system, style) = controls_style(self.platform, cx);
        let controls = || {
            let controls = WindowControls::new((self.id.clone(), "controls")).platform(style);
            let controls = match self.on_close.clone() {
                Some(close) => controls.on_close(move |window, cx| close(window, cx)),
                None => controls,
            };
            div()
                .flex_none()
                .debug_selector(|| "column-shell-controls".into())
                .child(controls)
        };
        let room = system && !window.is_fullscreen();
        let mut columns = Vec::with_capacity(count);
        for (ix, column) in self.columns.into_iter().enumerate() {
            let (first, last) = (ix == 0, ix + 1 == count);
            let lead = (!system && first && style == Platform::Mac).then(controls);
            let trail = (!system && last && style != Platform::Mac).then(controls);
            let flush = trail.is_some() && style == Platform::Windows;
            let header = header(column.id.clone(), window, cx)
                .when(first && room, |header| {
                    header.pl(cx.theme().traffic_light_inset())
                })
                .when(!(first && room), |header| header.pl_3())
                .when(!flush, |header| header.pr_3())
                .children(lead)
                .child(
                    items(
                        column.id.clone(),
                        column.leading,
                        title(column.title, cx),
                        column.actions,
                    )
                    .min_w(cx.theme().control_height(ControlSize::Lg))
                    .debug_selector(move || format!("column-shell-items-{ix}")),
                )
                .children(trail);
            let colors = &cx.theme().colors;
            columns.push(
                div()
                    .flex()
                    .flex_col()
                    .h_full()
                    .map(|box_| match column.width {
                        Some(width) => box_.w(width),
                        None => box_.flex_1(),
                    })
                    .when(!last, |box_| box_.border_r_1().border_color(colors.border))
                    .debug_selector(move || format!("column-shell-{ix}"))
                    .child(header)
                    .child(
                        // Absolute, so only the header sets the least width.
                        div().flex_1().min_h_0().relative().child(
                            div()
                                .absolute()
                                .inset_0()
                                .flex()
                                .flex_col()
                                .children(column.body),
                        ),
                    ),
            );
        }
        let theme = cx.theme();
        div()
            .flex()
            .h_full()
            .debug_selector(|| "column-shell".into())
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .children(columns)
    }
}

/// A header strip at the title bar's height that drags the window.
fn header(id: ElementId, window: &mut Window, cx: &mut App) -> gpui::Stateful<gpui::Div> {
    drag_region((id, "header"), window, cx)
        .flex()
        .flex_none()
        .items_center()
        .gap_2()
        .h(cx.theme().titlebar_height())
}

/// The host's header items: one control wide at least, scrolling when short.
fn items(
    id: ElementId,
    leading: SmallVec<[AnyElement; 2]>,
    title: gpui::Div,
    actions: SmallVec<[AnyElement; 2]>,
) -> gpui::Div {
    let group = || div().flex_none().flex().items_center().gap_2();
    div().flex_1().min_w_0().h_full().relative().child(
        on_axis(div().id((id, "items")))
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .gap_2()
            .overflow_x_scroll()
            .child(group().children(leading))
            .child(title)
            .child(group().children(actions)),
    )
}

/// The title, adding nothing to the least width; a spacer without one.
fn title(title: Option<SharedString>, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div().flex_1().min_w_0().h_full().relative().child(
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .text_size(theme.text_size(TextSize::Sm))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.colors.fg_muted)
            .children(title.map(Ellipsis::new)),
    )
}
