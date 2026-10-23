use std::path::PathBuf;

use gpui::{
    App, ElementId, Entity, IntoElement, PathPromptOptions, RenderOnce, SharedString, Window,
};

use super::{Input, TextInput};
use crate::{
    buttons::{Button, ButtonVariant},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize},
};

/// Opens the system's file dialog; `then` gets the chosen paths back in the window.
#[doc(hidden)]
pub fn choose(
    what: SharedString,
    options: PathPromptOptions,
    window: &mut Window,
    cx: &mut App,
    then: impl FnOnce(Vec<PathBuf>, &mut Window, &mut App) + 'static,
) {
    let chosen = cx.prompt_for_paths(options);
    log::info!("{what}: dialog opened");
    window
        .spawn(cx, async move |cx| {
            let paths = match chosen.await {
                Ok(Ok(Some(paths))) => paths,
                Ok(Ok(None)) => {
                    log::info!("{what}: dialog cancelled");
                    return;
                }
                Ok(Err(error)) => {
                    log::error!("{what}: dialog failed: {error:#}");
                    return;
                }
                Err(_) => {
                    log::error!("{what}: dialog closed without an answer");
                    return;
                }
            };
            if paths.is_empty() {
                log::error!("{what}: dialog chose nothing");
                return;
            }
            log::info!("{what}: chose {} paths", paths.len());
            if let Err(error) = cx.update(|window, cx| then(paths, window, cx)) {
                log::error!("{what}: window closed first: {error:#}");
            }
        })
        .detach();
}

/// A path field with Browse, which opens the system's file dialog.
///
/// ```
/// use ely_core::forms::{PathInput, TextInput};
/// use gpui::{App, AppContext, Window};
///
/// fn folder(window: &mut Window, cx: &mut App) -> PathInput {
///     let state = cx.new(|cx| TextInput::new(window, cx));
///     PathInput::new("folder", &state).directories()
/// }
/// ```
#[derive(IntoElement)]
pub struct PathInput {
    id: ElementId,
    state: Entity<TextInput>,
    directories: bool,
}

impl PathInput {
    /// A field under `id` over `state`, with Browse for files.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextInput>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            directories: false,
        }
    }

    /// Chooses folders instead of files.
    pub fn directories(mut self) -> Self {
        self.directories = true;
        self
    }
}

impl RenderOnce for PathInput {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let subtle = cx.theme().colors.fg_subtle;
        let (state, directories) = (self.state.clone(), self.directories);
        let icon = if directories {
            IconName::Folder
        } else {
            IconName::File
        };
        Input::new(&self.state)
            .prefix(Icon::new(icon).size(IconSize::Sm).color(subtle))
            .suffix(
                Button::new(self.id, "Browse…")
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, window, cx| {
                        let state = state.clone();
                        let options = PathPromptOptions {
                            files: !directories,
                            directories,
                            multiple: false,
                            prompt: Some(SharedString::from("Choose")),
                        };
                        choose(
                            "path input".into(),
                            options,
                            window,
                            cx,
                            move |paths, _, cx| {
                                let shown = paths[0].display().to_string();
                                state.update(cx, |input, cx| input.set_text(shown, cx));
                            },
                        );
                    }),
            )
    }
}
