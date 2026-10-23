//! A typed number read, rounded and clamped.

use crate::typography::format;

/// Reads a typed number: groups and the typographic minus are allowed.
#[doc(hidden)]
pub fn parse(text: &str) -> Option<f64> {
    let plain: String = text
        .chars()
        .filter(|ch| *ch != ',')
        .map(|ch| if ch == format::MINUS { '-' } else { ch })
        .collect();
    plain
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// Rounds to `precision` places, then clamps to the rounded numbers in `min..=max`.
pub(super) fn settle(value: f64, min: f64, max: f64, precision: usize) -> f64 {
    let scale = 10f64.powi(precision as i32);
    let (low, high) = (
        on_grid(min, scale, f64::ceil),
        on_grid(max, scale, f64::floor),
    );
    assert!(
        low <= high,
        "no {precision}-place number lies in {min}..={max}"
    );
    ((value * scale).round() / scale).clamp(low, high)
}

/// `bound` on the `1 / scale` grid: kept if float error alone put it off, else moved `inward`.
fn on_grid(bound: f64, scale: f64, inward: fn(f64) -> f64) -> f64 {
    let scaled = bound * scale;
    let near = scaled.round();
    if (scaled - near).abs() <= 4.0 * f64::EPSILON * near.abs().max(1.0) {
        near / scale
    } else {
        inward(scaled) / scale
    }
}
