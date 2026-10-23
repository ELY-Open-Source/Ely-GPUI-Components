use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use gpui::{Hsla, Rgba};

use crate::{forms::parse_hex, theme::Mode};

pub(super) const HUE_NAMES: [&str; 6] = ["red", "green", "yellow", "blue", "magenta", "cyan"];

const HUE_KEYS: [&[&str]; 6] = [
    &["red", "color1"],
    &["green", "color2"],
    &["yellow", "color3"],
    &["blue", "color4"],
    &["magenta", "color5", "purple"],
    &["cyan", "color6"],
];

const BRIGHT_KEYS: [&[&str]; 6] = [
    &["bright_red", "color9"],
    &["bright_green", "color10"],
    &["bright_yellow", "color11"],
    &["bright_blue", "color12"],
    &["bright_magenta", "color13", "bright_purple"],
    &["bright_cyan", "color14"],
];

/// A theme's colors, resolved as `omarchy-theme-color` does.
pub(super) struct Colors {
    pub mode: Mode,
    pub bg: Hsla,
    pub fg: Hsla,
    pub bright_fg: Hsla,
    pub muted: Hsla,
    pub accent: Option<Hsla>,
    pub selection: Hsla,
    /// In `HUE_NAMES` order; none where the theme lacks it.
    pub hues: [Option<Hsla>; 6],
    pub bright: [Option<Hsla>; 6],
    pub orange: Option<Hsla>,
}

struct Raw(BTreeMap<String, String>);

impl Raw {
    /// Reads lines as Omarchy's `read` loop does: loose quotes, comments, its key and value characters, and no last line without its newline.
    fn parse(text: &str) -> Self {
        let mut pairs = BTreeMap::new();
        for line in text
            .split_inclusive('\n')
            .filter_map(|line| line.strip_suffix('\n'))
        {
            let (key, value) = line.split_once('=').unwrap_or((line, ""));
            let key = key.replace(['"', '\'', ' '], "");
            if key.is_empty() || key.starts_with('#') {
                continue;
            }
            if !key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                log::warn!("omarchy: colors.toml skips {key:?}, its name has other characters");
                continue;
            }
            let value = match value.find(['"', '\'']) {
                Some(open) => {
                    let rest = &value[open + 1..];
                    rest[..rest.find(['"', '\'']).unwrap_or(rest.len())].to_string()
                }
                None => value.trim().to_string(),
            };
            if !value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "#(),._+/% -".contains(c))
            {
                log::warn!("omarchy: colors.toml skips {key:?}, its value has other characters");
                continue;
            }
            pairs.insert(key, value);
        }
        Self(pairs)
    }

    fn text(&self, key: &str) -> Option<&str> {
        self.0
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    /// The first of `keys` the theme sets, as a color.
    fn pick(&self, keys: &[&str]) -> Result<Option<Hsla>> {
        let Some((key, value)) = keys
            .iter()
            .find_map(|key| self.text(key).map(|value| (key, value)))
        else {
            return Ok(None);
        };
        hex(value).map(Some).with_context(|| format!("key {key:?}"))
    }

    fn need(&self, keys: &[&str]) -> Result<Hsla> {
        self.pick(keys)?
            .with_context(|| format!("{:?} is missing", keys[0]))
    }
}

/// `light_file`: the theme's folder holds a `light.mode` file.
pub(super) fn read(text: &str, light_file: bool) -> Result<Colors> {
    let raw = Raw::parse(text);
    let bg = raw.need(&["background", "bg", "color0"])?;
    let fg = raw.need(&["foreground", "fg", "color7"])?;
    let mut hues = [None; 6];
    let mut bright = [None; 6];
    for i in 0..6 {
        hues[i] = raw.pick(HUE_KEYS[i])?;
        bright[i] = match raw.pick(BRIGHT_KEYS[i])? {
            Some(color) => Some(color),
            None => hues[i].map(|hue| toward_white(hue, 0.2)),
        };
    }
    Ok(Colors {
        mode: mode(&raw, bg, light_file)?,
        bg,
        fg,
        bright_fg: raw
            .pick(&["bright_foreground", "bright_fg", "color15"])?
            .unwrap_or(fg),
        muted: raw
            .pick(&["muted", "color8", "dark_foreground", "dark_fg"])?
            .unwrap_or(fg),
        accent: raw.pick(&["accent"])?,
        selection: raw
            .pick(&["selection", "selection_background", "color8"])?
            .unwrap_or(bg),
        orange: raw.pick(&["orange"])?.or(hues[2]),
        hues,
        bright,
    })
}

/// `mode`, then `theme_type`, then `light.mode`, then the background's brightness.
fn mode(raw: &Raw, bg: Hsla, light_file: bool) -> Result<Mode> {
    match raw.text("mode").or(raw.text("theme_type")) {
        Some("dark") => Ok(Mode::Dark),
        Some("light") => Ok(Mode::Light),
        Some(other) => bail!("mode is {other:?}, not dark or light"),
        None if light_file => Ok(Mode::Light),
        None => {
            let sum: u32 = bytes(bg).iter().map(|&byte| u32::from(byte)).sum();
            Ok(if sum > 382 { Mode::Light } else { Mode::Dark })
        }
    }
}

/// `#rrggbb`, the one form Omarchy's themes write.
pub(super) fn hex(value: &str) -> Result<Hsla> {
    if value.len() != 7 || !value.starts_with('#') {
        bail!("{value:?} is not #rrggbb");
    }
    parse_hex(value)
        .map(Hsla::from)
        .map_err(|error| anyhow::anyhow!(error))
}

fn bytes(color: Hsla) -> [u8; 3] {
    let rgb = Rgba::from(color);
    [rgb.r, rgb.g, rgb.b].map(|channel| (channel * 255.0).round() as u8)
}

/// `color` a share of the way to white, in whole bytes as `mix_color` rounds.
fn toward_white(color: Hsla, share: f32) -> Hsla {
    let [r, g, b] = bytes(color).map(|byte| {
        let byte = f32::from(byte);
        ((byte + (255.0 - byte) * share + 0.5).floor() / 255.0).min(1.0)
    });
    Rgba { r, g, b, a: 1.0 }.into()
}
