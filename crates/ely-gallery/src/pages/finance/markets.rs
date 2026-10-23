use std::rc::Rc;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    finance::{
        Arrangement, Candle, CandlestickChart, ChartKind, ChartSync, ChartTypeSwitcher, DepthChart,
        Drawing, DrawingToolbar, IndicatorSelector, IntervalSelector, MultiChartLayout, Overlay,
        PointFigureChart, RenkoChart, Study, TimeRangeSelector, Tool,
    },
    theme::ActiveTheme,
};
use gpui::{
    App, AppContext as _, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::Timestamp;

use crate::{
    probe::probe,
    ui::{keep, noise, row, section, set},
};

/// A demo chart's width, inside the page's column.
pub(super) const WIDE: f32 = 840.0;

/// Today's prices every five minutes from the last close, moving as a quiet stock does.
pub(super) fn day(seed: u64, close: f64) -> Vec<f32> {
    let mut rng = noise(seed);
    let mut price = close;
    (0..78)
        .map(|_| {
            price *= 1.0 + (rng() - 0.5) * 0.004;
            price as f32
        })
        .collect()
}

/// A steady walk of candles `step` seconds apart from `start`, so captures repeat.
pub(super) fn walk(seed: u64, count: usize, start: f64, step: i64) -> Vec<Candle> {
    let mut rng = noise(seed);
    let first = 1_767_364_200;
    let mut close = start;
    (0..count)
        .map(|ix| {
            let open = close;
            let drift = (rng() - 0.47) * 0.034;
            close = (open * (1.0 + drift)).max(1.0);
            let high = open.max(close) * (1.0 + rng() * 0.011);
            let low = open.min(close) * (1.0 - rng() * 0.011);
            let volume = 1.0e6 * (0.5 + rng()) * (1.0 + drift.abs() * 24.0);
            let time = Timestamp::from_second(first + ix as i64 * step).expect("a time in range");
            Candle::new(time, (open, high, low, close), volume)
        })
        .collect()
}

/// How many candles of an interval a range holds, and each candle's seconds.
fn span(range: &str, interval: &str) -> (usize, i64) {
    let seconds = match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "1h" => 3_600,
        "4h" => 14_400,
        "1W" => 604_800,
        "1M" => 2_592_000,
        _ => 86_400,
    };
    let days = match range {
        "1D" => 1.0,
        "5D" => 5.0,
        "1M" => 22.0,
        "3M" => 63.0,
        "6M" => 126.0,
        "YTD" => 180.0,
        "1Y" => 252.0,
        "5Y" => 1_260.0,
        _ => 2_520.0,
    };
    let per_day = (23_400.0 / seconds as f64).max(1.0 / (seconds as f64 / 86_400.0));
    (((days * per_day) as usize).clamp(12, 2_400), seconds)
}

pub fn market(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kind = keep("finance-kind", || ChartKind::Candles, window, cx);
    let range = keep("finance-range", || SharedString::from("6M"), window, cx);
    let interval = keep("finance-interval", || SharedString::from("1D"), window, cx);
    let (now_kind, now_range, now_interval) = (
        *kind.read(cx),
        range.read(cx).clone(),
        interval.read(cx).clone(),
    );
    let (count, step) = span(&now_range, &now_interval);
    let seed = 7 + now_interval.len() as u64 * 13 + step as u64 % 97;
    let candles = walk(seed, count, 184.0, step);
    let alert = (candles.last().expect("a range holds candles").close * 1.04).round();
    let (pick_kind, pick_range, pick_interval) = (kind.clone(), range.clone(), interval.clone());
    section(
        "CandlestickChart / OHLCChart / HeikinAshiChart / LineQuoteChart / VolumeChart / ChartTypeSwitcher / TimeRangeSelector / IntervalSelector",
        "Prices as candles, bars, smoothed candles, a line or an area, with volume along the bottom and averages over the top. Drag or scroll sideways to move through time, hold Cmd and scroll to zoom, double-press to return to the newest.",
        cx,
    )
    .child(
        row()
            .justify_between()
            .w_full()
            .child(
                row()
                    .child(ChartTypeSwitcher::new("finance-kind", now_kind).on_change(move |kind, _, cx| set(&pick_kind, kind, cx)))
                    .child(IntervalSelector::new("finance-interval", now_interval.clone()).on_change(move |key, _, cx| set(&pick_interval, key.clone(), cx))),
            )
            .child(TimeRangeSelector::new("finance-range", now_range).on_change(move |key, _, cx| set(&pick_range, key.clone(), cx))),
    )
    .child(probe(
        "market",
        div().w(px(WIDE)).child(
            CandlestickChart::new("market", candles)
                .kind(now_kind)
                .title(format!("ELY · {now_interval}"))
                .volume()
                .overlay(Overlay::Sma(20))
                .overlay(Overlay::Ema(50))
                .alert(alert, "Alert"),
        ),
    ))
}

pub fn indicators(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep(
        "finance-indicators",
        || {
            (
                vec![Overlay::Bollinger(20, 2.0)],
                vec![Study::Macd, Study::Rsi],
            )
        },
        window,
        cx,
    );
    let (overlays, studies) = chosen.read(cx).clone();
    let candles = walk(11, 160, 92.0, 86_400);
    let peer = walk(19, 160, 61.0, 86_400);
    let chart = CandlestickChart::new("studies", candles)
        .title("ELY · 1D")
        .compare("PEER", peer);
    let chart = overlays
        .iter()
        .fold(chart, |chart, overlay| chart.overlay(*overlay));
    let chart = studies
        .iter()
        .fold(chart, |chart, study| chart.study(*study));
    let picked = chosen.clone();
    section(
        "IndicatorOverlay / IndicatorPane / IndicatorSelector / CompareSymbol / PriceAlertLine",
        "Bollinger bands over the prices, MACD and RSI in panes below, and a second symbol drawn from the same start so their moves compare. Choose indicators from the menu.",
        cx,
    )
    .child(row().child(IndicatorSelector::new("finance-choose", overlays, studies).on_change(move |overlays, studies, _, cx| set(&picked, (overlays, studies), cx))))
    .child(div().w_full().child(chart.alert(98.0, "Take profit")))
}

pub fn drawings(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tool = keep("finance-tool", || Some(Tool::Trend), window, cx);
    let marks = keep(
        "finance-marks",
        || vec![Drawing::Fib((96.0, 118.0), (128.0, 142.0))],
        window,
        cx,
    );
    let (now_tool, now_marks) = (*tool.read(cx), marks.read(cx).clone());
    let (pick, kept, cleared) = (tool.clone(), marks.clone(), marks.clone());
    section(
        "DrawingTools / DrawingToolbar",
        "Pick a tool and drag over the prices: a trend line, a level, a box, or retracement levels between a swing's two ends. The pointer moves through time again.",
        cx,
    )
    .child(
        row()
            .child(DrawingToolbar::new("finance-tools", now_tool).on_change(move |tool, _, cx| set(&pick, tool, cx)))
            .child(Button::new("finance-clear", "Clear").variant(ButtonVariant::Ghost).on_click(move |_, _, cx| set(&cleared, Vec::new(), cx))),
    )
    .child(probe(
        "drawings",
        div().w(px(WIDE)).child(
            CandlestickChart::new("drawn", walk(23, 140, 120.0, 86_400))
                .title("ELY · 1D")
                .drawings(now_marks)
                .tool(now_tool)
                .on_draw(move |drawing, _, cx| {
                    kept.update(cx, |marks, cx| {
                        marks.push(drawing);
                        cx.notify();
                    })
                }),
        ),
    ))
}

pub fn depth(cx: &mut App) -> impl IntoElement + use<> {
    let mut rng = noise(29);
    let bids: Vec<(f64, f64)> = (0..40)
        .map(|ix| {
            (
                187.20 - ix as f64 * 0.05,
                200.0 + rng() * 900.0 + ix as f64 * 40.0,
            )
        })
        .collect();
    let asks: Vec<(f64, f64)> = (0..40)
        .map(|ix| {
            (
                187.26 + ix as f64 * 0.05,
                200.0 + rng() * 900.0 + ix as f64 * 35.0,
            )
        })
        .collect();
    section("DepthChart", "Every bid and ask as a running total out from the spread. Hover to read the total to any price.", cx)
        .child(probe("depth", div().w(px(WIDE)).child(DepthChart::new("depth", bids, asks))))
}

pub fn profile(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "MarketProfile / VolumeProfile",
        "How much traded at each price in view, as bars from the right edge; the busiest price is the darkest.",
        cx,
    )
    .child(div().w_full().child(CandlestickChart::new("profile", walk(31, 120, 54.0, 86_400)).title("ELY · 1D").profile()))
}

pub fn bricks(cx: &mut App) -> impl IntoElement + use<> {
    let candles = Rc::new(walk(37, 260, 74.0, 86_400));
    section(
        "RenkoChart / PointAndFigure",
        "Time drops out: a brick for each whole move of two dollars, and columns of Xs and Os that turn after three boxes the other way.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(div().flex_1().child(RenkoChart::new("renko", candles.clone(), 2.0)))
            .child(div().flex_1().child(PointFigureChart::new("figure", candles, 2.0, 3))),
    )
}

pub fn together(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let arrangement = keep("finance-arrangement", || Arrangement::Four, window, cx);
    let sync = keep(
        "finance-sync",
        || None::<gpui::Entity<ChartSync>>,
        window,
        cx,
    );
    if sync.read(cx).is_none() {
        let made = cx.new(|_| ChartSync::default());
        set(&sync, Some(made), cx);
    }
    let shared = sync.read(cx).clone().expect("made above");
    let now = *arrangement.read(cx);
    let pick = arrangement.clone();
    let names = [
        ("ELY", 3, 184.0),
        ("NOVA", 5, 62.0),
        ("ORBIT", 8, 131.0),
        ("PINE", 13, 27.0),
    ];
    let layout = names.iter().fold(
        MultiChartLayout::new("finance-grid", now),
        |layout, (name, seed, start)| {
            let chart = CandlestickChart::new(
                SharedString::from(format!("sync-{name}")),
                walk(*seed, 120, *start, 86_400),
            )
            .title(*name)
            .sync(&shared);
            layout.cell(chart.h(cx.theme().chart().height * 0.8))
        },
    );
    section(
        "MultiChartLayout / ChartSync",
        "Four symbols moving together: pan, zoom or point at one and the others follow. Switch between one chart, two across, two down or four.",
        cx,
    )
    .child(layout.on_arrange(move |arrangement, _, cx| set(&pick, arrangement, cx)))
}
