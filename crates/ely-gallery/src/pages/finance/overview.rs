use ely_gpui_component::{
    data_display::Tone,
    finance::{
        Earnings, EarningsCalendar, EconomicCalendar, MarketHeatmap, MarketOverview, MarketStatus,
        NewsFeed, Release, Screener, Session, Story, SymbolBadge, SymbolSearch,
        TradingSessionClock, Watch, Watchlist,
    },
    forms::TextInput,
    tables::{Cell, Column, FilterRule, Row, Test},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, civil::date, tz::TimeZone};

use super::markets::day;
use crate::{
    probe::probe,
    ui::{keep, row, section, set},
};

/// The demo's symbols: letters, name, where each trades, last close, and a seed for its day.
const SYMBOLS: [(&str, &str, &str, f64, u64); 8] = [
    ("ELY", "Ely Systems", "NASDAQ", 184.9, 41),
    ("NOVA", "Nova Energy", "NYSE", 62.6, 43),
    ("ORBIT", "Orbit Freight", "NYSE", 128.9, 47),
    ("PINE", "Pine Foods", "NASDAQ", 27.7, 53),
    ("QUILL", "Quill Media", "NASDAQ", 87.7, 59),
    ("RIVER", "River Health", "NYSE", 45.2, 67),
    ("SOLACE", "Solace Labs", "NASDAQ", 310.5, 73),
    ("TERRA", "Terra Mining", "LSE", 10.1, 79),
];

fn watches() -> Vec<Watch> {
    SYMBOLS
        .iter()
        .map(|(symbol, name, _, close, seed)| {
            let day = day(*seed, *close);
            Watch {
                symbol: (*symbol).into(),
                name: (*name).into(),
                last: f64::from(*day.last().expect("a day")),
                close: *close,
                day,
            }
        })
        .collect()
}

pub fn watch(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep("finance-watched", || SharedString::from("ELY"), window, cx);
    let field = window.use_keyed_state("finance-symbol", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Search symbols")
    });
    let (now, pick, find) = (chosen.read(cx).clone(), chosen.clone(), chosen.clone());
    let symbols = SYMBOLS.iter().map(|(symbol, name, venue, ..)| {
        (
            SharedString::from(*symbol),
            SharedString::from(*name),
            SharedString::from(*venue),
        )
    });
    let badges = SYMBOLS.iter().take(4).map(|(symbol, _, venue, ..)| {
        SymbolBadge::new(SharedString::from(format!("badge-{symbol}")), *symbol).venue(*venue)
    });
    section(
        "Watchlist / SymbolBadge / SymbolSearch",
        "Symbols to follow, each with its day as a line, its price and its move; a symbol as a tile of its letters; and a search by letters or name.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(520.)).child(
                    Watchlist::new("watchlist", watches())
                        .selected(now.clone())
                        .on_select(move |symbol, _, cx| set(&pick, symbol.clone(), cx)),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        SymbolSearch::new("symbol-search", &field, symbols)
                            .on_pick(move |symbol, _, cx| set(&find, symbol.clone(), cx)),
                    )
                    .child(row().children(badges))
                    .child(Caption::new(format!("Watching {now}."))),
            ),
    )
}

pub fn heatmap(cx: &mut App) -> impl IntoElement + use<> {
    let tiles = [
        ("SOLACE", 920.0, 0.0212),
        ("ELY", 780.0, 0.0124),
        ("ORBIT", 410.0, -0.0203),
        ("NOVA", 380.0, -0.0081),
        ("QUILL", 260.0, 0.0041),
        ("RIVER", 240.0, 0.0),
        ("PINE", 150.0, -0.0152),
        ("TERRA", 120.0, -0.0312),
        ("ATLAS", 110.0, 0.0088),
        ("BIRCH", 90.0, 0.0263),
        ("CEDAR", 70.0, -0.0047),
        ("DUNE", 60.0, 0.0151),
    ];
    let map = tiles.iter().fold(
        MarketHeatmap::new("heatmap"),
        |map, (symbol, weight, change)| map.tile(*symbol, *weight, *change),
    );
    section(
        "MarketHeatmap / SectorTreemap",
        "Each symbol a tile sized by its market value and tinted by its move: deeper green as it rises, deeper red as it falls. Hover a tile to read it.",
        cx,
    )
    .child(probe("heatmap", div().w(px(840.)).child(map.h(px(320.)))))
}

pub fn screener(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rules = keep(
        "finance-screen",
        || {
            (
                vec![FilterRule {
                    column: "change".into(),
                    test: Test::Above,
                    value: "0".into(),
                }],
                false,
            )
        },
        window,
        cx,
    );
    let (now, any) = rules.read(cx).clone();
    let store = rules.clone();
    let columns = [
        Column::new("symbol", "Symbol"),
        Column::new("sector", "Sector"),
        Column::new("last", "Last").end().decimals(2),
        Column::new("change", "Change")
            .end()
            .decimals(2)
            .suffix("%"),
        Column::new("value", "Value").end().decimals(1).suffix(" B"),
        Column::new("earnings", "P/E").end().decimals(1),
    ];
    let sectors = [
        "Software",
        "Energy",
        "Transport",
        "Food",
        "Media",
        "Health",
        "Software",
        "Materials",
    ];
    let rows = SYMBOLS.iter().zip(watches()).zip(sectors).enumerate().map(
        |(ix, (((symbol, ..), watch), sector))| {
            let change = (watch.last / watch.close - 1.0) * 100.0;
            Row::new(
                *symbol,
                [
                    Cell::Text((*symbol).into()),
                    Cell::Tag(sector.into(), Tone::Neutral),
                    Cell::Number(watch.last),
                    Cell::Number(change),
                    Cell::Number(12.0 + ix as f64 * 17.5),
                    Cell::Number(9.0 + (ix * 7 % 30) as f64),
                ],
            )
        },
    );
    section(
        "Screener / FilterPanel",
        "Securities narrowed by rules: build the rules above, and the table keeps only what passes them.",
        cx,
    )
    .child(
        Screener::new("screener", columns, rows)
            .rules(now, any)
            .on_rules(move |rules, any, _, cx| set(&store, (rules.to_vec(), any), cx)),
    )
}

pub fn overview(cx: &mut App) -> impl IntoElement + use<> {
    let indexes = [
        ("EGX 500", "Broad market", 5_812.4, 41),
        ("EGX Tech", "Technology", 19_440.1, 43),
        ("EGX Small", "Small companies", 2_231.8, 47),
    ];
    let movers = [
        ("BIRCH", 18.92, 18.43),
        ("SOLACE", 317.08, 310.5),
        ("TERRA", 9.78, 10.1),
        ("ORBIT", 126.28, 128.9),
    ];
    let board = indexes.iter().fold(
        MarketOverview::new("overview")
            .status(MarketStatus::new("New York", Session::Open).next("Closes in 2 h 14 min")),
        |board, (symbol, name, close, seed)| {
            let day = day(*seed, *close);
            let last = f64::from(*day.last().expect("a day"));
            board.index((*symbol, *name), (last, *close), day)
        },
    );
    let board = movers.iter().fold(board, |board, (symbol, last, close)| {
        board.mover(*symbol, *last, *close)
    });
    let clock = [
        ("Sydney", 23.0, 5.0),
        ("Tokyo", 0.0, 6.0),
        ("Hong Kong", 1.5, 8.0),
        ("Frankfurt", 7.0, 15.5),
        ("London", 8.0, 16.5),
        ("New York", 13.5, 20.0),
    ]
    .iter()
    .fold(
        TradingSessionClock::new(15.75),
        |clock, (name, open, close)| clock.market(*name, *open, *close),
    );
    section(
        "MarketOverview / IndexCard / MarketStatus / TradingSessionClock",
        "A market at a glance: its session, its indexes as cards, and the day's movers. Below, each market's hours across one day in UTC, those open now lit.",
        cx,
    )
    .child(board)
    .child(
        row()
            .gap_6()
            .child(MarketStatus::new("London", Session::After).next("Opens in 15 h 15 min"))
            .child(MarketStatus::new("Tokyo", Session::Closed).next("Opens in 8 h 15 min"))
            .child(MarketStatus::new("Frankfurt", Session::Before).next("Opens in 30 min")),
    )
    .child(clock)
}

fn stamp(hour: i64, minute: i64) -> Timestamp {
    Timestamp::from_second(1_790_294_400 + hour * 3_600 + minute * 60).expect("a time")
}

pub fn calendars(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let release = |(hour, minute),
                   region: &str,
                   event: &str,
                   weight,
                   figures: (Option<f64>, Option<f64>, Option<f64>),
                   unit: &str| Release {
        time: stamp(hour, minute),
        region: SharedString::from(region.to_string()),
        event: SharedString::from(event.to_string()),
        weight,
        actual: figures.0,
        forecast: figures.1,
        previous: figures.2,
        unit: SharedString::from(unit.to_string()),
    };
    let releases = [
        release(
            (1, 30),
            "JP",
            "Tokyo core consumer prices",
            2,
            (Some(2.4), Some(2.3), Some(2.2)),
            "%",
        ),
        release(
            (8, 0),
            "DE",
            "Ifo business climate",
            2,
            (Some(87.1), Some(87.6), Some(87.3)),
            "",
        ),
        release(
            (12, 30),
            "US",
            "Durable goods orders",
            2,
            (Some(0.8), Some(0.5), Some(-0.3)),
            "%",
        ),
        release(
            (12, 30),
            "US",
            "Initial jobless claims",
            3,
            (None, Some(224.0), Some(231.0)),
            "K",
        ),
        release(
            (14, 0),
            "US",
            "New home sales",
            1,
            (None, Some(668.0), Some(652.0)),
            "K",
        ),
    ];
    let earnings = [
        (
            date(2026, 9, 28),
            "ELY",
            "Ely Systems",
            false,
            1.84,
            Some(1.97),
        ),
        (
            date(2026, 9, 29),
            "NOVA",
            "Nova Energy",
            true,
            0.62,
            Some(0.55),
        ),
        (
            date(2026, 9, 30),
            "SOLACE",
            "Solace Labs",
            false,
            3.10,
            None,
        ),
        (date(2026, 10, 1), "QUILL", "Quill Media", true, 0.48, None),
    ]
    .map(
        |(date, symbol, name, before_open, estimate, reported)| Earnings {
            date,
            symbol: symbol.into(),
            name: name.into(),
            before_open,
            estimate,
            reported,
        },
    );
    let story = |(hour, minute), source: &str, headline: &str, symbols: &[&str], lean| Story {
        source: SharedString::from(source.to_string()),
        time: stamp(hour, minute),
        headline: SharedString::from(headline.to_string()),
        symbols: symbols
            .iter()
            .map(|symbol| SharedString::from(symbol.to_string()))
            .collect(),
        lean,
    };
    let stories = [
        story(
            (15, 42),
            "Wire",
            "Ely Systems lifts its outlook as cloud orders climb",
            &["ELY"],
            Some(true),
        ),
        story(
            (15, 10),
            "Markets Desk",
            "Freight rates slide for a third week",
            &["ORBIT", "TERRA"],
            Some(false),
        ),
        story(
            (14, 36),
            "Wire",
            "Nova Energy names a new chief financial officer",
            &["NOVA"],
            None,
        ),
        story(
            (13, 58),
            "Daily Brief",
            "Solace Labs trial meets its main goal",
            &["SOLACE"],
            Some(true),
        ),
    ];
    let opened = keep("finance-story", || None::<usize>, window, cx);
    let (read, open) = (*opened.read(cx), opened.clone());
    section(
        "EconomicCalendar / EarningsCalendar / NewsFeed",
        "The day's releases with how much each moves markets and its figure beside the forecast; reports due by date; and stories newest first, each dot leaning green or red. Press a story to open it.",
        cx,
    )
    .child(EconomicCalendar::new(releases).zone(TimeZone::UTC))
    .child(EarningsCalendar::new("earnings", earnings))
    .child(
        div()
            .w(px(640.))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                NewsFeed::new("news", stories)
                    .zone(TimeZone::UTC)
                    .on_open(move |ix, _, cx| set(&open, Some(ix), cx)),
            )
            .children(read.map(|ix| Caption::new(format!("Opened story {}.", ix + 1)))),
    )
}
