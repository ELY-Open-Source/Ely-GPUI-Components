use std::collections::HashMap;

use gpui::{
    App, AssetSource, Global, Hsla, IntoElement, Radians, RenderOnce, SharedString, Transformation,
    Window, prelude::*, size, svg,
};

use crate::theme::{ActiveTheme, IconSize};

mod names;

pub use names::IconName;

/// A bundled icon, or an app's path with the icon a miss shows.
#[derive(Clone, Debug, PartialEq)]
enum Source {
    Bundled(IconName),
    App {
        path: SharedString,
        broken: IconName,
    },
}

/// A Lucide icon, or an app's own SVG, in the theme's sizes and colors.
///
/// ```
/// use ely_core::{
///     primitives::{Icon, IconName},
///     theme::IconSize,
/// };
///
/// let check = Icon::new(IconName::Check).size(IconSize::Sm);
/// ```
#[derive(IntoElement)]
pub struct Icon {
    source: Source,
    size: IconSize,
    color: Option<Hsla>,
    hover: Option<(SharedString, Hsla)>,
    rotation: Option<Radians>,
    scale: f32,
}

impl Icon {
    /// A bundled icon at the medium size.
    pub fn new(name: IconName) -> Self {
        Self::drawing(Source::Bundled(name))
    }

    /// Draws an SVG supplied by the application's asset source; a missing one logs and shows `broken`.
    pub fn from_path(path: impl Into<SharedString>) -> Self {
        Self::drawing(Source::App {
            path: path.into(),
            broken: IconName::ImageOff,
        })
    }

    fn drawing(source: Source) -> Self {
        Self {
            source,
            size: IconSize::Md,
            color: None,
            hover: None,
            rotation: None,
            scale: 1.0,
        }
    }

    /// Its box size.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Defaults to `colors.fg`.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Color while the pointer is over `group`.
    pub fn group_hover_color(
        mut self,
        group: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        self.hover = Some((group.into(), color.into()));
        self
    }

    /// Turns it by `angle` about its center.
    pub fn rotate(mut self, angle: impl Into<Radians>) -> Self {
        self.rotation = Some(angle.into());
        self
    }

    /// Draws the glyph larger or smaller inside the same box.
    pub fn scale(mut self, factor: f32) -> Self {
        self.scale = factor;
        self
    }
}

impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Icon::new(name)
    }
}

/// App paths already looked up, and whether the source held them.
#[derive(Default)]
struct FoundIcons(HashMap<SharedString, bool>);

impl Global for FoundIcons {}

fn found(source: &dyn AssetSource, path: &str) -> bool {
    match source.load(path) {
        Ok(Some(_)) => true,
        Ok(None) => {
            log::error!("ely: icon {path} is not in the app's asset source");
            false
        }
        Err(error) => {
            log::error!("ely: icon {path} failed to load: {error:#}");
            false
        }
    }
}

impl Icon {
    /// What a missing app icon shows: a broken picture in danger.
    fn broken(self, mark: IconName, cx: &App) -> Self {
        Self {
            source: Source::Bundled(mark),
            color: Some(cx.theme().colors.danger),
            hover: None,
            ..self
        }
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let assets = cx.asset_source().clone();
        let icon = match self.source.clone() {
            Source::App { path, broken }
                if !*cx
                    .default_global::<FoundIcons>()
                    .0
                    .entry(path.clone())
                    .or_insert_with(|| found(assets.as_ref(), &path)) =>
            {
                self.broken(broken, cx)
            }
            _ => self,
        };
        let theme = cx.theme();
        let color = icon.color.unwrap_or(theme.colors.fg);
        let drawn = match icon.source {
            Source::Bundled(name) => svg().data(name.svg()),
            Source::App { path, .. } => svg().path(path),
        };
        drawn
            .size(theme.icon_size(icon.size))
            .flex_none()
            .text_color(color)
            .when_some(icon.hover, |svg, (group, hover)| {
                svg.group_hover(group, |style| style.text_color(hover))
            })
            .when(icon.rotation.is_some() || icon.scale != 1.0, |svg| {
                svg.with_transformation(
                    Transformation::rotate(icon.rotation.unwrap_or(Radians(0.0)))
                        .with_scaling(size(icon.scale, icon.scale)),
                )
            })
    }
}

#[cfg(test)]
mod tests;
