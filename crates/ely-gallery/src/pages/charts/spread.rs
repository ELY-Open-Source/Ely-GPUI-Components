use ely_gpui_component::charts::{BoxPlot, Histogram, ViolinPlot, WaterfallChart};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};

use crate::ui::{noise, section};

pub fn distributions(cx: &mut App) -> impl IntoElement + use<> {
    let mut rng = noise(13);
    let mut normal =
        |mean: f64, spread: f64| -> f64 { mean + spread * ((rng() + rng() + rng() + rng()) - 2.0) };
    let times: Vec<f64> = (0..240).map(|_| normal(320.0, 120.0).max(40.0)).collect();
    let groups: Vec<(&str, Vec<f64>)> = [("Europe", 220.0), ("Americas", 310.0), ("Asia", 270.0)]
        .into_iter()
        .map(|(name, mean)| (name, (0..60).map(|_| normal(mean, 90.0)).collect()))
        .collect();
    let boxes = groups
        .iter()
        .fold(BoxPlot::new("boxes"), |plot, (name, values)| {
            plot.group(*name, values.clone())
        });
    let violins = groups
        .iter()
        .fold(ViolinPlot::new("violins"), |plot, (name, values)| {
            plot.group(*name, values.clone())
        });
    section(
        "Histogram / BoxPlot / ViolinPlot",
        "Response times by count in equal bins, and by region as boxes and as violins that widen where values gather.",
        cx,
    )
    .child(div().w(px(720.)).child(Histogram::new("times", times).bins(12)))
    .child(div().flex().gap_6().child(div().w(px(360.)).child(boxes)).child(div().w(px(360.)).child(violins)))
}

pub fn waterfall(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "WaterfallChart",
        "How a year's revenue became its profit: each step floats from where the last left off, and the totals stand on zero.",
        cx,
    )
    .child(
        div().w(px(720.)).child(
            WaterfallChart::new("profit")
                .step("Revenue", 420.0)
                .step("Refunds", -36.0)
                .total("Net sales")
                .step("Cost of sales", -148.0)
                .step("Operating", -121.0)
                .step("Other income", 18.0)
                .total("Profit"),
        ),
    )
}
