mod colors;
#[cfg(not(target_family = "wasm"))]
mod follow;
mod palette;
#[cfg(test)]
mod tests;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::theme::{Mode, Palette};
#[cfg(not(target_family = "wasm"))]
pub use follow::{follow_omarchy, stop_omarchy};

/// Omarchy's current theme, read: its name and mode, the palette it gives, and the hues it lacks, which Ely's colors fill.
#[derive(Clone, Debug, PartialEq)]
pub struct OmarchyTheme {
    /// Its name, when the folder gives one.
    pub name: Option<String>,
    /// Light or dark.
    pub mode: Mode,
    /// The palette it gives.
    pub colors: Palette,
    /// The hues it lacks, which Ely's colors fill.
    pub missing: Vec<&'static str>,
}

/// `~/.local/state/omarchy/current`, where Omarchy keeps it whatever `XDG_STATE_HOME` says; none without Omarchy.
pub fn omarchy_current_dir() -> Option<PathBuf> {
    let current = std::env::home_dir()?.join(".local/state/omarchy/current");
    current.is_dir().then_some(current)
}

/// Reads `current/theme/colors.toml` as `omarchy-theme-color` resolves it, its text lifted until Ely's contrast audit passes; none while Omarchy has written no colors. Fails on a file that cannot be read, a color that is not `#rrggbb`, a mode that is neither dark nor light, or a palette no lift makes pass.
pub fn read_omarchy_theme(current: &Path) -> Result<Option<OmarchyTheme>> {
    let theme = current.join("theme");
    let file = theme.join("colors.toml");
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("omarchy: reading {}", file.display()));
        }
    };
    let name_file = current.join("theme.name");
    let name = match fs::read_to_string(&name_file) {
        Ok(name) => Some(name.trim().to_string()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| format!("omarchy: reading {}", name_file.display()));
        }
    };
    let read = parse(&text, name, theme.join("light.mode").is_file())
        .with_context(|| format!("omarchy: {}", file.display()))?;
    let shown = read.name.as_deref().unwrap_or("unnamed");
    log::info!("omarchy: theme {shown} read, {:?}", read.mode);
    if !read.missing.is_empty() {
        log::warn!(
            "omarchy: theme {shown} lacks {}, Ely's colors stand in",
            read.missing.join(", ")
        );
    }
    Ok(Some(read))
}

fn parse(text: &str, name: Option<String>, light_file: bool) -> Result<OmarchyTheme> {
    let colors = colors::read(text, light_file)?;
    let (palette, missing) = palette::build(&colors)?;
    Ok(OmarchyTheme {
        name,
        mode: colors.mode,
        colors: palette,
        missing,
    })
}
