use std::rc::Rc;

use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::{
    motion::Spinner,
    primitives::{Icon, IconName},
    shell::StatusBarItem,
    theme::{ActiveTheme, IconSize, TextSize},
};

type OnPress = Rc<dyn Fn(&mut Window, &mut App)>;

fn item(
    id: ElementId,
    label: impl Into<SharedString>,
    tooltip: &'static str,
    press: Option<OnPress>,
) -> StatusBarItem {
    let item = StatusBarItem::new(id).label(label).tooltip(tooltip);
    match press {
        Some(press) => item.on_click(move |_, window, cx| press(window, cx)),
        None => item,
    }
}

macro_rules! pressable {
    ($name:ident) => {
        impl $name {
            /// Runs when it is pressed.
            pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
                self.press = Some(Rc::new(handler));
                self
            }
        }
    };
}

/// The cursor's line and column, and how many characters are selected when some are.
///
/// ```
/// use ely_editor::CursorPosition;
///
/// let place = CursorPosition::new("place", 12, 4).selected(18);
/// ```
#[derive(IntoElement)]
pub struct CursorPosition {
    id: ElementId,
    line: usize,
    column: usize,
    selected: usize,
    press: Option<OnPress>,
}

impl CursorPosition {
    /// `line` and `column` count from one.
    pub fn new(id: impl Into<ElementId>, line: usize, column: usize) -> Self {
        assert!(line > 0 && column > 0, "lines and columns count from one");
        Self {
            id: id.into(),
            line,
            column,
            selected: 0,
            press: None,
        }
    }

    /// How many characters are selected.
    pub fn selected(mut self, characters: usize) -> Self {
        self.selected = characters;
        self
    }
}

pressable!(CursorPosition);

impl RenderOnce for CursorPosition {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut label = format!("Ln {}, Col {}", self.line, self.column);
        if self.selected > 0 {
            label.push_str(&format!(" ({} selected)", self.selected));
        }
        item(self.id, label, "Go to line", self.press)
    }
}

/// The language a file is read as.
///
/// ```
/// use ely_editor::LanguageMode;
///
/// let language = LanguageMode::new("language", "Rust").on_click(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct LanguageMode {
    id: ElementId,
    language: SharedString,
    press: Option<OnPress>,
}

impl LanguageMode {
    /// The item under `id` naming `language`.
    pub fn new(id: impl Into<ElementId>, language: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            language: language.into(),
            press: None,
        }
    }
}

pressable!(LanguageMode);

impl RenderOnce for LanguageMode {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        item(
            self.id,
            self.language,
            "Select the language mode",
            self.press,
        )
    }
}

/// The text encoding a file is saved in.
///
/// ```
/// use ely_editor::Encoding;
///
/// let encoding = Encoding::new("encoding", "UTF-8");
/// ```
#[derive(IntoElement)]
pub struct Encoding {
    id: ElementId,
    name: SharedString,
    press: Option<OnPress>,
}

impl Encoding {
    /// The item under `id` naming the encoding `name`, such as UTF-8.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            press: None,
        }
    }
}

pressable!(Encoding);

impl RenderOnce for Encoding {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        item(
            self.id,
            self.name,
            "Reopen or save with an encoding",
            self.press,
        )
    }
}

/// How a file ends its lines.
///
/// ```
/// use ely_editor::LineEnding;
///
/// let ending = LineEnding::new("ending", false);
/// ```
#[derive(IntoElement)]
pub struct LineEnding {
    id: ElementId,
    crlf: bool,
    press: Option<OnPress>,
}

impl LineEnding {
    /// `crlf` for carriage return and line feed, else line feed alone.
    pub fn new(id: impl Into<ElementId>, crlf: bool) -> Self {
        Self {
            id: id.into(),
            crlf,
            press: None,
        }
    }
}

pressable!(LineEnding);

impl RenderOnce for LineEnding {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        item(
            self.id,
            if self.crlf { "CRLF" } else { "LF" },
            "Change the line ending",
            self.press,
        )
    }
}

/// How a file indents: spaces or tabs, and how wide.
///
/// ```
/// use ely_editor::IndentSettings;
///
/// let indent = IndentSettings::new("indent", true, 4);
/// ```
#[derive(IntoElement)]
pub struct IndentSettings {
    id: ElementId,
    spaces: bool,
    size: usize,
    press: Option<OnPress>,
}

impl IndentSettings {
    /// The item under `id`: `spaces` or tabs, `size` columns wide. Panics on zero.
    pub fn new(id: impl Into<ElementId>, spaces: bool, size: usize) -> Self {
        assert!(size > 0, "an indent is at least one column");
        Self {
            id: id.into(),
            spaces,
            size,
            press: None,
        }
    }
}

pressable!(IndentSettings);

impl RenderOnce for IndentSettings {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let label = if self.spaces {
            format!("Spaces: {}", self.size)
        } else {
            format!("Tab Size: {}", self.size)
        };
        item(self.id, label, "Change the indentation", self.press)
    }
}

/// The branch checked out, a mark when the tree has changes, and how far it is ahead of and behind its upstream.
///
/// ```
/// use ely_editor::BranchIndicator;
///
/// let branch = BranchIndicator::new("branch", "main").dirty(true).sync(2, 0);
/// ```
#[derive(IntoElement)]
pub struct BranchIndicator {
    id: ElementId,
    branch: SharedString,
    dirty: bool,
    ahead: usize,
    behind: usize,
    press: Option<OnPress>,
}

impl BranchIndicator {
    /// The item under `id` naming `branch`.
    pub fn new(id: impl Into<ElementId>, branch: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            branch: branch.into(),
            dirty: false,
            ahead: 0,
            behind: 0,
            press: None,
        }
    }

    /// Marks changes in the tree.
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Commits not yet pushed, and commits not yet pulled.
    pub fn sync(mut self, ahead: usize, behind: usize) -> Self {
        self.ahead = ahead;
        self.behind = behind;
        self
    }
}

pressable!(BranchIndicator);

impl RenderOnce for BranchIndicator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut label = format!("{}{}", self.branch, if self.dirty { "*" } else { "" });
        if self.ahead > 0 || self.behind > 0 {
            label.push_str(&format!("  ↑{} ↓{}", self.ahead, self.behind));
        }
        item(self.id, label, "Checkout a branch", self.press).icon(IconName::GitBranch)
    }
}

/// Where a language server stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LspState {
    /// Starting up.
    Starting,
    /// Ready.
    Ready,
    /// Working on something it names.
    Busy(SharedString),
    /// Stopped, and why.
    Failed(SharedString),
}

/// A language server's name and state: a spinner while it starts or works, a check when ready, a mark when it failed.
///
/// ```
/// use ely_editor::{LspState, LspStatus};
///
/// let server = LspStatus::new("server", "rust-analyzer", LspState::Busy("indexing".into()));
/// ```
#[derive(IntoElement)]
pub struct LspStatus {
    id: ElementId,
    server: SharedString,
    state: LspState,
    press: Option<OnPress>,
}

impl LspStatus {
    /// The item under `id` naming `server` in `state`.
    pub fn new(id: impl Into<ElementId>, server: impl Into<SharedString>, state: LspState) -> Self {
        Self {
            id: id.into(),
            server: server.into(),
            state,
            press: None,
        }
    }
}

pressable!(LspStatus);

impl RenderOnce for LspStatus {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let (lead, label, tooltip) = match &self.state {
            LspState::Starting => (
                Spinner::new((self.id.clone(), "spin"))
                    .size(IconSize::Xs)
                    .into_any_element(),
                format!("{} starting", self.server),
                "Starting",
            ),
            LspState::Busy(task) => (
                Spinner::new((self.id.clone(), "spin"))
                    .size(IconSize::Xs)
                    .into_any_element(),
                format!("{} · {task}", self.server),
                "Working",
            ),
            LspState::Ready => (
                Icon::new(IconName::CircleCheck)
                    .size(IconSize::Xs)
                    .color(colors.fg_muted)
                    .into_any_element(),
                self.server.to_string(),
                "Ready",
            ),
            LspState::Failed(why) => (
                Icon::new(IconName::CircleAlert)
                    .size(IconSize::Xs)
                    .color(colors.danger)
                    .into_any_element(),
                format!("{} · {why}", self.server),
                "Failed",
            ),
        };
        item(self.id, label, tooltip, self.press).leading(lead)
    }
}

/// A bell with how many notifications wait, quiet when there are none.
///
/// ```
/// use ely_editor::NotificationBell;
///
/// let bell = NotificationBell::new("bell", 3).on_click(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct NotificationBell {
    id: ElementId,
    count: usize,
    press: Option<OnPress>,
}

impl NotificationBell {
    /// The bell under `id` with `count` waiting.
    pub fn new(id: impl Into<ElementId>, count: usize) -> Self {
        Self {
            id: id.into(),
            count,
            press: None,
        }
    }
}

pressable!(NotificationBell);

impl RenderOnce for NotificationBell {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let bell = div()
            .relative()
            .child(
                Icon::new(IconName::Bell)
                    .size(IconSize::Xs)
                    .color(colors.fg_muted),
            )
            .children((self.count > 0).then(|| {
                div()
                    .absolute()
                    .top_neg_1()
                    .right_neg_1p5()
                    .px_0p5()
                    .rounded_full()
                    .bg(colors.accent)
                    .text_color(colors.on_accent)
                    .text_size(theme.text_size(TextSize::Xs) * 0.75)
                    .line_height(theme.text_size(TextSize::Xs) * 0.9)
                    .child(self.count.min(99).to_string())
            }));
        let tooltip = if self.count == 0 {
            "No new notifications"
        } else {
            "Notifications"
        };
        let item = StatusBarItem::new(self.id).tooltip(tooltip).leading(bell);
        match self.press {
            Some(press) => item.on_click(move |_, window, cx| press(window, cx)),
            None => item,
        }
    }
}
