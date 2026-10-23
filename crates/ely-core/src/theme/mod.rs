mod contrast;
mod measures;
mod palette;
mod platform;
mod syntax;
mod tokens;

#[cfg(all(test, feature = "test-support"))]
mod tests;

use std::time::Duration;

use gpui::{App, Global, SharedString, WindowAppearance};
use web_time::Instant;

use crate::motion;
pub use contrast::{contrast, grades, luminance, over};
pub use measures::{
    CalendarSizes, CanvasSizes, ChartSizes, DashboardSizes, FileSizes, GenerativeSizes,
    InteractionSizes, MapSizes, MediaSizes, MessagingSizes, MiscSizes, OnboardingSizes,
    ProjectSizes, ToolingSizes,
};
pub use palette::{HUE_NAMES, Mix, Palette, Syntax};
pub use platform::Platform;
pub use syntax::{SyntaxTheme, syntax_themes};
pub use tokens::{
    AvatarSize, ContainerSize, ControlSize, Density, Elevation, IconSize, Radius, TextSize,
};

const FRAME: Duration = Duration::from_millis(8);

/// Light or dark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Dark text on light grounds.
    #[default]
    Light,
    /// Light text on dark grounds.
    Dark,
}

impl From<WindowAppearance> for Mode {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
        }
    }
}

/// Design tokens. One global per app.
pub struct Theme {
    mode: Mode,
    high_contrast: bool,
    color_blind_safe: bool,
    transition: u64,
    custom: [Option<Palette>; 2],
    /// The palette on screen, between two during a fade.
    pub colors: Palette,
    /// How much room controls take.
    pub density: Density,
    /// Scales every radius.
    pub radius_scale: f32,
    /// Scales every text and icon size.
    pub font_scale: f32,
    /// The face text sets in; Inter unless set.
    pub font_family: SharedString,
    /// The face code sets in; JetBrains Mono unless set.
    pub mono_family: SharedString,
    /// Holds repeating motion still and cuts the rest short.
    pub reduced_motion: bool,
    /// Whose window buttons, title bar and key caps components draw; the system's to start.
    pub platform: Platform,
}

impl Global for Theme {}

/// Reads the theme from the app.
pub trait ActiveTheme {
    /// The app's theme.
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl Theme {
    #[doc(hidden)]
    pub fn init(cx: &mut App) {
        cx.set_global(Theme {
            mode: Mode::Light,
            high_contrast: false,
            color_blind_safe: false,
            transition: 0,
            custom: [None, None],
            colors: Palette::light(false),
            density: Density::Standard,
            radius_scale: 1.0,
            font_scale: 1.0,
            font_family: "Inter".into(),
            mono_family: "JetBrains Mono".into(),
            reduced_motion: false,
            platform: Platform::current(),
        });
    }

    /// The mode shown, or the one a fade heads to.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Whether high contrast is on.
    pub fn high_contrast(&self) -> bool {
        self.high_contrast
    }

    /// Whether charts wear the color-blind safe hues.
    pub fn color_blind_safe(&self) -> bool {
        self.color_blind_safe
    }

    /// Whether the mode is dark.
    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// Cross-fades the palette to `mode`.
    pub fn set_mode(mode: Mode, cx: &mut App) {
        log::info!("theme: mode -> {mode:?}");
        cx.global_mut::<Theme>().mode = mode;
        Self::fade_to_target(cx);
    }

    /// Sets `mode` at once, stopping any fade under way.
    pub fn set_mode_now(mode: Mode, cx: &mut App) {
        log::info!("theme: mode -> {mode:?}, at once");
        let theme = cx.global_mut::<Theme>();
        theme.mode = mode;
        theme.transition += 1;
        theme.colors = theme.target();
        cx.refresh_windows();
    }

    /// Turns high contrast on or off with a fade; it strengthens Ely's own palettes alone.
    pub fn set_high_contrast(on: bool, cx: &mut App) {
        log::info!("theme: high contrast -> {on}");
        cx.global_mut::<Theme>().high_contrast = on;
        Self::fade_to_target(cx);
    }

    /// Chart hues that stay apart for color-blind eyes, in Ely's own palettes.
    pub fn set_color_blind_safe(on: bool, cx: &mut App) {
        log::info!("theme: color-blind safe charts -> {on}");
        cx.global_mut::<Theme>().color_blind_safe = on;
        Self::fade_to_target(cx);
    }

    /// The owner's palette for `mode`, as an edited or imported theme gives, or none to bring Ely's back. The colors fade to it when `mode` shows. High contrast and color-blind safe charts change Ely's own palettes alone.
    pub fn set_palette(mode: Mode, palette: Option<Palette>, cx: &mut App) {
        log::info!(
            "theme: {mode:?} palette -> {}",
            if palette.is_some() {
                "the owner's"
            } else {
                "Ely's"
            }
        );
        cx.global_mut::<Theme>().custom[mode as usize] = palette;
        Self::fade_to_target(cx);
    }

    /// The palette the shown mode fades to: the owner's, or Ely's.
    pub fn palette(&self) -> Palette {
        self.target()
    }

    /// Edits non-color tokens, then repaints.
    pub fn update(cx: &mut App, edit: impl FnOnce(&mut Theme)) {
        edit(cx.global_mut::<Theme>());
        cx.refresh_windows();
    }

    fn target(&self) -> Palette {
        if let Some(palette) = &self.custom[self.mode as usize] {
            return palette.clone();
        }
        let mut palette = match self.mode {
            Mode::Light => Palette::light(self.high_contrast),
            Mode::Dark => Palette::dark(self.high_contrast),
        };
        if self.color_blind_safe {
            palette.chart = palette::color_blind_chart(self.mode);
        }
        palette
    }

    fn fade_to_target(cx: &mut App) {
        let duration = motion::duration(motion::THEME, cx);
        let theme = cx.global_mut::<Theme>();
        theme.transition += 1;
        let (id, from, to) = (theme.transition, theme.colors.clone(), theme.target());
        let start = Instant::now();
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor().timer(FRAME).await;
                let t = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
                let running = cx.update(|cx| {
                    let theme = cx.global_mut::<Theme>();
                    if theme.transition != id {
                        return false;
                    }
                    theme.colors = from.mix(&to, motion::ease_in_out_cubic(t));
                    cx.refresh_windows();
                    t < 1.0
                });
                if !running {
                    return;
                }
            }
        })
        .detach();
    }
}
