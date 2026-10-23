use ely_gpui_component::{
    finance::{
        BidAskBar, DomLadder, Level2Quotes, OrderBook, PriceChangeBadge, PriceText, QuoteCard,
        Side, SpreadIndicator, TickerTape, TimeAndSales, Trade,
    },
    theme::TextSize,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, tz::TimeZone};

use super::markets::day;
use crate::ui::{keep, noise, row, section, set};

/// Prices, each with the size waiting there.
type Levels = Vec<(f64, f64)>;

/// A book around a price: bids below it and asks above, a tick apart, sizes drawn steadily.
fn levels(seed: u64, middle: f64, tick: f64, count: usize) -> (Levels, Levels) {
    let mut rng = noise(seed);
    let bids = (0..count)
        .map(|ix| {
            (
                middle - tick * (ix as f64 + 1.0),
                (120.0 + rng() * 800.0 + ix as f64 * 60.0).round(),
            )
        })
        .collect();
    let asks = (0..count)
        .map(|ix| {
            (
                middle + tick * (ix as f64 + 1.0),
                (120.0 + rng() * 800.0 + ix as f64 * 55.0).round(),
            )
        })
        .collect();
    (bids, asks)
}

pub fn quotes(cx: &mut App) -> impl IntoElement + use<> {
    let tape = [
        ("ELY", 187.24, 0.0124),
        ("NOVA", 62.10, -0.0081),
        ("ORBIT", 131.52, 0.0203),
        ("PINE", 27.33, -0.0152),
        ("QUILL", 88.05, 0.0041),
        ("RIVER", 45.60, 0.0),
        ("SOLACE", 312.90, 0.0077),
        ("TERRA", 9.84, -0.0312),
    ]
    .iter()
    .fold(TickerTape::new("tape"), |tape, (symbol, now, share)| {
        tape.quote(*symbol, *now, *share)
    });
    let cards = [
        ("ELY", "Ely Systems", 41, 184.9),
        ("NOVA", "Nova Energy", 43, 62.6),
        ("ORBIT", "Orbit Freight", 47, 128.9),
    ]
    .map(|(symbol, name, seed, close)| {
        let line = day(seed, close);
        let now = f64::from(*line.last().expect("a day"));
        div().flex_1().child(QuoteCard::new(
            SharedString::from(format!("card-{symbol}")),
            (symbol, name),
            (now, close),
            line,
        ))
    });
    section(
        "Ticker / TickerTape / QuoteCard / PriceText / PriceChangeBadge",
        "Quotes running past in a band, and symbols at a glance: the price flashes green or red as it moves, and the badge reads the move since the last close.",
        cx,
    )
    .child(div().w_full().child(tape))
    .child(div().flex().gap_4().children(cards))
    .child(
        row()
            .child(PriceText::new("price-up", 187.26).from(187.24).size(TextSize::Lg))
            .child(PriceText::new("price-down", 62.08).from(62.10).size(TextSize::Lg))
            .child(PriceChangeBadge::new(184.9, 187.26))
            .child(PriceChangeBadge::new(62.6, 62.08))
            .child(PriceChangeBadge::new(187.26, 187.26)),
    )
}

pub fn book(cx: &mut App) -> impl IntoElement + use<> {
    let (bids, asks) = levels(51, 187.25, 0.05, 12);
    let (bid_size, ask_size) = (
        bids.iter().map(|level| level.1).sum::<f64>(),
        asks.iter().map(|level| level.1).sum::<f64>(),
    );
    let mut rng = noise(61);
    let venues = ["NYSE", "ARCA", "BATS", "EDGX", "IEX", "NSDQ"];
    let level2 = (0..10).fold(Level2Quotes::new(), |quotes, ix| {
        let venue = venues[ix % venues.len()];
        let step = (ix / 2) as f64 * 0.05;
        quotes
            .bid(venue, 187.20 - step, (100.0 + rng() * 900.0).round())
            .ask(
                venues[(ix + 3) % venues.len()],
                187.30 + step,
                (100.0 + rng() * 900.0).round(),
            )
    });
    section(
        "OrderBook / SpreadIndicator / BidAskBar / Level2 Quotes",
        "Orders waiting on each side, best in the middle, each level shaded by its running total; the spread and the balance between sides; and each venue's quotes side by side.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(div().w(px(320.)).child(OrderBook::new("book", bids.clone(), asks.clone())))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(SpreadIndicator::new(bids[0].0, asks[0].0))
                    .child(BidAskBar::new(bid_size, ask_size))
                    .child(level2),
            ),
    )
}

pub fn tape(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let mut rng = noise(71);
    let start = 1_790_342_400;
    let trades: Vec<Trade> = (0..40)
        .map(|ix| {
            let buy = rng() > 0.45;
            Trade {
                time: Timestamp::from_second(start + ix * 7).expect("a time"),
                price: 187.25 + (rng() - 0.5) * 0.4,
                size: (rng().powi(3) * 5_000.0 + 20.0).round(),
                side: if buy { Side::Buy } else { Side::Sell },
            }
        })
        .collect();
    let (bids, asks) = levels(81, 187.25, 0.05, 10);
    let traded = keep("finance-traded", || None::<SharedString>, window, cx);
    let (told, tell) = (traded.read(cx).clone(), traded.clone());
    section(
        "TimeAndSales / TradeTape / MarketDepthLadder / DOM",
        "Trades as they print, newest on top and large prints bold; and the book as a ladder around the last price. Press a size to trade at its price.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(div().w(px(320.)).child(TimeAndSales::new("prints", trades).large(2_000.0).zone(TimeZone::UTC)))
            .child(
                div().flex_1().flex().flex_col().gap_2().child(
                    DomLadder::new("ladder", bids, asks, 187.25, 0.05).on_trade(move |side, at, _, cx| {
                        let words = format!("{} at {at:.2}", if side == Side::Buy { "Bid" } else { "Offer" });
                        set(&tell, Some(words.into()), cx)
                    }),
                )
                .children(told.map(|words| Caption::new(format!("Placed: {words}.")))),
            ),
    )
}
