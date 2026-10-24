use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};

use super::OnIndex;
use crate::{
    buttons::{ButtonVariant, IconButton, shortcut_text},
    forms::Checkbox,
    primitives::IconName,
    theme::{ActiveTheme, ControlSize, Elevation, Radius, TextSize},
    typography::{Ellipsis, tabular},
};

/// Whether the program runs or waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugState {
    /// The program runs.
    Running,
    /// The program waits.
    Paused,
}

/// What the debugger is asked to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugCommand {
    /// Run on.
    Continue,
    /// Stop and wait.
    Pause,
    /// Run the next line, over calls.
    StepOver,
    /// Step into the call on the line.
    StepInto,
    /// Run to the end of this call.
    StepOut,
    /// Start again.
    Restart,
    /// End the session.
    Stop,
}

type OnCommand = Rc<dyn Fn(DebugCommand, &mut Window, &mut App)>;
type OnSwitch = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;

/// The floating row that drives a debug session: continue or pause, the three steps while paused, restart and stop. Each tooltip names the key debuggers use for it; the app binds the keys.
///
/// ```
/// use ely_debug::{DebugState, DebugToolbar};
///
/// let controls = DebugToolbar::new("controls", DebugState::Paused).on_command(|_command, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct DebugToolbar {
    id: ElementId,
    state: DebugState,
    on_command: Option<OnCommand>,
}

impl DebugToolbar {
    /// A toolbar under `id` for a program in `state`.
    pub fn new(id: impl Into<ElementId>, state: DebugState) -> Self {
        Self {
            id: id.into(),
            state,
            on_command: None,
        }
    }

    /// Runs with the command pressed.
    pub fn on_command(
        mut self,
        handler: impl Fn(DebugCommand, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_command = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DebugToolbar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let paused = self.state == DebugState::Paused;
        let platform = theme.platform;
        let button =
            |command: DebugCommand, icon: IconName, words: &str, key: &str, enabled: bool| {
                let run = self.on_command.clone();
                IconButton::new((self.id.clone(), format!("{command:?}")), icon)
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .tooltip(format!("{words}  {}", shortcut_text(key, platform)))
                    .disabled(!enabled)
                    .when_some(run, |button, run| {
                        button.on_click(move |_, window, cx| {
                            log::info!("debug toolbar: {command:?}");
                            run(command, window, cx)
                        })
                    })
            };
        let rule = || div().w_0().h_4().border_l_1().border_color(colors.border);
        div()
            .flex()
            .items_center()
            .gap_0p5()
            .p_1()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(colors.border)
            .bg(colors.overlay)
            .shadow(theme.elevation(Elevation::Floating))
            .child(if paused {
                button(
                    DebugCommand::Continue,
                    IconName::Play,
                    "Continue",
                    "f5",
                    true,
                )
            } else {
                button(DebugCommand::Pause, IconName::Pause, "Pause", "f6", true)
            })
            .child(button(
                DebugCommand::StepOver,
                IconName::RedoDot,
                "Step over",
                "f10",
                paused,
            ))
            .child(button(
                DebugCommand::StepInto,
                IconName::ArrowDownToDot,
                "Step into",
                "f11",
                paused,
            ))
            .child(button(
                DebugCommand::StepOut,
                IconName::ArrowUpFromDot,
                "Step out",
                "shift-f11",
                paused,
            ))
            .child(rule())
            .child(button(
                DebugCommand::Restart,
                IconName::RotateCw,
                "Restart",
                "secondary-shift-f5",
                true,
            ))
            .child(button(
                DebugCommand::Stop,
                IconName::Square,
                "Stop",
                "shift-f5",
                true,
            ))
    }
}

/// A breakpoint: where, whether it is on, the condition that guards it, and how often it has hit.
#[derive(Clone, Debug, PartialEq)]
pub struct Breakpoint {
    /// The file.
    pub path: SharedString,
    /// From one.
    pub line: usize,
    /// Whether it is on.
    pub enabled: bool,
    /// The condition that guards it.
    pub condition: Option<SharedString>,
    /// How often it has hit.
    pub hits: usize,
}

/// Every breakpoint: turn each on or off, see its condition and hits, go to it, or remove it.
///
/// ```
/// use ely_debug::{Breakpoint, BreakpointList};
///
/// let stops = BreakpointList::new(
///     "stops",
///     [Breakpoint { path: "src/main.rs".into(), line: 12, enabled: true, condition: Some("n > 3".into()), hits: 2 }],
/// )
/// .on_toggle(|_ix, _on, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct BreakpointList {
    id: ElementId,
    breakpoints: Vec<Breakpoint>,
    on_toggle: Option<OnSwitch>,
    on_open: Option<OnIndex>,
    on_remove: Option<OnIndex>,
}

impl BreakpointList {
    /// A list under `id` of `breakpoints`.
    pub fn new(
        id: impl Into<ElementId>,
        breakpoints: impl IntoIterator<Item = Breakpoint>,
    ) -> Self {
        Self {
            id: id.into(),
            breakpoints: breakpoints.into_iter().collect(),
            on_toggle: None,
            on_open: None,
            on_remove: None,
        }
    }

    /// Runs with a breakpoint's place and whether it should be on.
    pub fn on_toggle(
        mut self,
        handler: impl Fn(usize, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }

    /// Runs with the breakpoint to go to, by place.
    pub fn on_open(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    /// Runs with the breakpoint to remove, by place.
    pub fn on_remove(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for BreakpointList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let rows: Vec<AnyElement> = self
            .breakpoints
            .iter()
            .enumerate()
            .map(|(ix, point)| {
                let (folder, name) = point.path.rsplit_once('/').unwrap_or(("", &point.path));
                let group = SharedString::from(format!("breakpoint-{ix}"));
                let toggle = self.on_toggle.clone();
                let (open, remove) = (self.on_open.clone(), self.on_remove.clone());
                let check = Checkbox::new((self.id.clone(), format!("on-{ix}")), point.enabled)
                    .name(format!("{}:{}", point.path, point.line));
                let check = match toggle {
                    Some(toggle) => check.on_change(move |on, window, cx| {
                        cx.stop_propagation();
                        toggle(ix, on, window, cx)
                    }),
                    None => check,
                };
                div()
                    .id((self.id.clone(), format!("row-{ix}")))
                    .group(group.clone())
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded(theme.radius(Radius::Sm))
                    .hover(|row| row.bg(colors.hover))
                    .when_some(open, |row, open| {
                        row.cursor_pointer()
                            .on_click(move |_, window, cx| open(ix, window, cx))
                    })
                    .child(check)
                    .child(
                        div()
                            .flex_none()
                            .size_2()
                            .rounded_full()
                            .border_1()
                            .border_color(colors.danger)
                            .when(point.enabled, |dot| dot.bg(colors.danger)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.fg)
                            .child(format!("{name}:{}", point.line)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_color(colors.fg_subtle)
                            .child(Ellipsis::new(folder.to_string())),
                    )
                    .children(point.condition.clone().map(|condition| {
                        div()
                            .flex_none()
                            .font_family(theme.mono_family.clone())
                            .text_size(theme.text_size(TextSize::Xs))
                            .text_color(colors.fg_muted)
                            .child(format!("when {condition}"))
                    }))
                    .when(point.hits > 0, |row| {
                        row.child(
                            tabular(div().flex_none().text_color(colors.fg_subtle))
                                .child(format!("×{}", point.hits)),
                        )
                    })
                    .child(
                        div()
                            .opacity(0.)
                            .group_hover(group, |actions| actions.opacity(1.))
                            .child(
                                IconButton::new(
                                    (self.id.clone(), format!("remove-{ix}")),
                                    IconName::X,
                                )
                                .variant(ButtonVariant::Ghost)
                                .size(ControlSize::Sm)
                                .tooltip("Remove breakpoint")
                                .when_some(
                                    remove,
                                    |button, remove| {
                                        button.on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            remove(ix, window, cx)
                                        })
                                    },
                                ),
                            ),
                    )
                    .into_any_element()
            })
            .collect();
        div()
            .flex()
            .flex_col()
            .text_size(theme.text_size(TextSize::Sm))
            .children(rows)
    }
}
