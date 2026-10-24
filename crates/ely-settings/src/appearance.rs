use std::rc::Rc;

use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, div,
};

use crate::{
    buttons::{IconButton, SegmentedControl},
    canvas::color_well,
    forms::{ColorPalette, Slider},
    i18n,
    primitives::IconName,
    theme::{ActiveTheme, Density, Mode, TextSize},
};

/// How an app picks light or dark: always one, or as the system does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    /// Always light.
    Light,
    /// Always dark.
    Dark,
    /// As the system is.
    System,
}

impl Appearance {
    /// Each with its value, catalog key and icon.
    const ALL: [(Appearance, &'static str, IconName); 3] = [
        (Appearance::Light, "theme.light", IconName::Sun),
        (Appearance::Dark, "theme.dark", IconName::Moon),
        (Appearance::System, "theme.system", IconName::Monitor),
    ];

    /// The mode it shows when the system is in `system`.
    pub fn mode(self, system: Mode) -> Mode {
        match self {
            Appearance::Light => Mode::Light,
            Appearance::Dark => Mode::Dark,
            Appearance::System => system,
        }
    }
}

type OnAppearance = Rc<dyn Fn(Appearance, &mut Window, &mut App)>;

/// Light, dark, or as the system does, as three segments with their icons. The owner applies the choice, as through `Theme::set_mode`.
///
/// ```
/// use ely_settings::{Appearance, ThemeSelector};
///
/// let selector = ThemeSelector::new("theme", Appearance::System, |_appearance, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ThemeSelector {
    id: ElementId,
    appearance: Appearance,
    on_change: OnAppearance,
}

impl ThemeSelector {
    /// A selector under `id` showing `appearance`; `on_change` gets the one picked.
    pub fn new(
        id: impl Into<ElementId>,
        appearance: Appearance,
        on_change: impl Fn(Appearance, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            appearance,
            on_change: Rc::new(on_change),
        }
    }
}

impl RenderOnce for ThemeSelector {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_change = self.on_change;
        let (_, words, _) = Appearance::ALL
            .iter()
            .find(|(each, _, _)| *each == self.appearance)
            .expect("every appearance is offered");
        Appearance::ALL
            .iter()
            .fold(
                SegmentedControl::new(self.id, *words),
                |control, (_, words, icon)| {
                    control.segment(*words, i18n::text(cx, words, &[]), Some(*icon))
                },
            )
            .on_change(move |value, window, cx| {
                let (chosen, _, _) = Appearance::ALL
                    .iter()
                    .find(|(_, words, _)| *words == value.as_ref())
                    .expect("an appearance the control offers");
                log::info!("theme selector: {chosen:?}");
                on_change(*chosen, window, cx);
            })
    }
}

type OnColor = Rc<dyn Fn(Hsla, &mut Window, &mut App)>;

/// An accent picked from named presets, or any color from the well beside them.
///
/// ```
/// use ely_settings::AccentColorPicker;
/// use gpui::rgb;
///
/// let picker = AccentColorPicker::new("accent", rgb(0x3b82f6), [("Blue", rgb(0x3b82f6)), ("Green", rgb(0x16a34a))], |_color, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct AccentColorPicker {
    id: ElementId,
    color: Hsla,
    presets: Vec<(SharedString, Hsla)>,
    on_change: OnColor,
}

impl AccentColorPicker {
    /// A picker under `id` showing `color`, offering `presets` by name; `on_change` gets each color picked.
    pub fn new(
        id: impl Into<ElementId>,
        color: impl Into<Hsla>,
        presets: impl IntoIterator<Item = (impl Into<SharedString>, impl Into<Hsla>)>,
        on_change: impl Fn(Hsla, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            color: color.into(),
            presets: presets
                .into_iter()
                .map(|(name, color)| (name.into(), color.into()))
                .collect(),
            on_change: Rc::new(on_change),
        }
    }
}

impl RenderOnce for AccentColorPicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_change = self.on_change;
        let (picked, custom) = (on_change.clone(), on_change);
        let report = |on_change: OnColor| {
            move |color: Hsla, window: &mut Window, cx: &mut App| {
                log::info!("accent: {color:?}");
                on_change(color, window, cx);
            }
        };
        let columns = u16::try_from(self.presets.len()).expect("a handful of presets");
        div()
            .debug_selector(|| "accent-color-picker".into())
            .flex()
            .items_start()
            .gap_2()
            .child(
                div().flex_1().min_w_0().child(
                    ColorPalette::new((self.id.clone(), "presets"), self.presets)
                        .columns(columns)
                        .selected(self.color)
                        .on_change(report(picked)),
                ),
            )
            .child(color_well(
                cx,
                (self.id, "custom").into(),
                self.color,
                report(custom),
            ))
    }
}

type OnSize = Rc<dyn Fn(f32, &mut Window, &mut App)>;

/// A text size on a slider, in pixels, over a line set at it, with a way back to the default size.
///
/// ```
/// use ely_settings::FontSizeControl;
///
/// let control = FontSizeControl::new("font", 15.0, 14.0, (11.0, 22.0), |_size, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct FontSizeControl {
    id: ElementId,
    size: f32,
    default: f32,
    range: (f32, f32),
    on_change: OnSize,
}

impl FontSizeControl {
    /// `size` and `default` lie within `range`, in pixels.
    pub fn new(
        id: impl Into<ElementId>,
        size: f32,
        default: f32,
        range: (f32, f32),
        on_change: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        assert!(range.0 < range.1, "font sizes {range:?} run backwards");
        let held = |what: &str, value: f32| {
            if !(range.0..=range.1).contains(&value) {
                log::error!("font size control: {what} {value} outside {range:?}; pegged");
            }
            value.clamp(range.0, range.1)
        };
        Self {
            id: id.into(),
            size: held("size", size),
            default: held("default", default),
            range,
            on_change: Rc::new(on_change),
        }
    }
}

impl RenderOnce for FontSizeControl {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_change = self.on_change;
        let theme = cx.theme();
        let (slid, reset, default) = (on_change.clone(), on_change, self.default);
        let (low, high) = self.range;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div().flex_none().w(theme.label_width()).child(
                            Slider::new((self.id.clone(), "slider"), self.size as f64)
                                .range(low as f64, high as f64)
                                .label(i18n::text(cx, "settings.font_size", &[]))
                                .step(1.0)
                                .on_change(move |size, window, cx| {
                                    slid(size as f32, window, cx);
                                }),
                        ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(theme.text_size(TextSize::Sm))
                            .child(format!("{} px", self.size)),
                    )
                    .child(
                        IconButton::new((self.id, "reset"), IconName::RotateCcw)
                            .tooltip(i18n::text(cx, "theme.size.reset", &[]))
                            .disabled(self.size == default)
                            .on_click(move |_, window, cx| {
                                reset(default, window, cx);
                            }),
                    ),
            )
            .child(
                div().flex().child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(Pixels::from(self.size))
                        .text_color(theme.colors.fg)
                        .child(i18n::text(cx, "theme.size.sample", &[])),
                ),
            )
    }
}

type OnDensity = Rc<dyn Fn(Density, &mut Window, &mut App)>;

/// How close things sit: compact, standard or comfortable, as three segments.
///
/// ```
/// use ely_core::theme::Density;
/// use ely_settings::DensitySelector;
///
/// let selector = DensitySelector::new("density", Density::Standard, |_density, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct DensitySelector {
    id: ElementId,
    density: Density,
    on_change: OnDensity,
}

const DENSITIES: [(Density, &str); 3] = [
    (Density::Compact, "Compact"),
    (Density::Standard, "Standard"),
    (Density::Comfortable, "Comfortable"),
];

impl DensitySelector {
    /// A selector under `id` showing `density`; `on_change` gets the one picked.
    pub fn new(
        id: impl Into<ElementId>,
        density: Density,
        on_change: impl Fn(Density, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            density,
            on_change: Rc::new(on_change),
        }
    }
}

impl RenderOnce for DensitySelector {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let on_change = self.on_change;
        let (_, words) = DENSITIES
            .iter()
            .find(|(each, _)| *each == self.density)
            .expect("every density is offered");
        DENSITIES
            .iter()
            .fold(
                SegmentedControl::new(self.id, *words),
                |control, (_, words)| control.segment(*words, *words, None),
            )
            .on_change(move |value, window, cx| {
                let (chosen, _) = DENSITIES
                    .iter()
                    .find(|(_, words)| *words == value.as_ref())
                    .expect("a density the control offers");
                log::info!("density: {chosen:?}");
                on_change(*chosen, window, cx);
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Appearance;
    use crate::theme::Mode;

    #[test]
    fn system_follows_the_system_and_the_rest_hold() {
        assert_eq!(Appearance::System.mode(Mode::Dark), Mode::Dark);
        assert_eq!(Appearance::Light.mode(Mode::Dark), Mode::Light);
        assert_eq!(Appearance::Dark.mode(Mode::Light), Mode::Dark);
    }
}
