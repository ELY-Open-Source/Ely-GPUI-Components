use ely_gpui_component::{
    finance::{
        Fill, GreeksTable, Leg, LeverageSlider, MarginIndicator, OptionChain, OptionQuote, Order,
        OrderConfirmDialog, OrderEntry, OrderKind, OrderTable, PayoffDiagram, PnLDisplay, Position,
        PositionTable, QuickTradeButtons, RiskMeter, Side, Strike, TradeHistoryTable, Working,
    },
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, tz::TimeZone};

use super::markets::WIDE;
use crate::{
    probe::probe,
    ui::{keep, row, section, set},
};

/// The demo symbol's last trade.
const LAST: f64 = 187.25;

fn at(minutes: i64) -> Timestamp {
    Timestamp::from_second(1_790_343_000 + minutes * 60).expect("a time")
}

pub fn ticket(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let order = keep(
        "finance-order",
        || Order {
            symbol: "ELY".into(),
            side: Side::Buy,
            kind: OrderKind::Limit,
            quantity: 100.0,
            price: 187.20,
        },
        window,
        cx,
    );
    let asking = keep("finance-asking", || false, window, cx);
    let told = keep("finance-told", || None::<SharedString>, window, cx);
    let now = order.read(cx).clone();
    let dialog = (*asking.read(cx)).then(|| {
        let (shut, placed) = (asking.clone(), told.clone());
        OrderConfirmDialog::new("finance-confirm", now.clone(), LAST, move |_, cx| {
            set(&shut, false, cx)
        })
        .on_confirm(move |order, _, cx| {
            set(
                &placed,
                Some(format!("Placed: {}.", order.words(2)).into()),
                cx,
            )
        })
    });
    let caption = told
        .read(cx)
        .clone()
        .unwrap_or("Nothing placed yet.".into());
    let (store, ask, quick) = (order.clone(), asking.clone(), told.clone());
    let entry = OrderEntry::new("ticket", now, LAST)
        .on_change(move |order, _, cx| set(&store, order, cx))
        .on_submit(move |_, _, cx| set(&ask, true, cx));
    let buttons = QuickTradeButtons::new("quick", 187.24, 187.26).on_trade(move |side, _, cx| {
        let words = match side {
            Side::Buy => "Bought 100 ELY at 187.26.",
            Side::Sell => "Sold 100 ELY at 187.24.",
        };
        set(&quick, Some(words.into()), cx)
    });
    section(
        "OrderEntry / TradePanel / QuickTradeButtons / OrderConfirmDialog",
        "An order filled in field by field, with what it comes to; its button asks before the order goes. Quick buttons trade at the bid or the ask in one press.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(probe("ticket", div().w(px(340.)).child(entry)))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(buttons)
                    .child(Caption::new(caption)),
            ),
    )
    .children(dialog)
}

pub fn blotter(cx: &mut App) -> impl IntoElement + use<> {
    let positions = [
        ("ELY", 300.0, 171.40, LAST),
        ("NOVA", -150.0, 64.80, 62.08),
        ("ORBIT", 80.0, 139.10, 131.52),
        ("PINE", 1_200.0, 25.90, 27.33),
    ]
    .map(|(symbol, quantity, cost, last)| Position {
        symbol: symbol.into(),
        quantity,
        cost,
        last,
    });
    let order = |symbol: &str, side, kind, quantity, price| Order {
        symbol: SharedString::from(symbol.to_string()),
        side,
        kind,
        quantity,
        price,
    };
    let working = [
        (
            4,
            order("ELY", Side::Buy, OrderKind::Limit, 200.0, 185.50),
            0.0,
        ),
        (
            11,
            order("NOVA", Side::Sell, OrderKind::Stop, 150.0, 60.00),
            0.0,
        ),
        (
            26,
            order("QUILL", Side::Buy, OrderKind::Limit, 500.0, 87.80),
            220.0,
        ),
    ]
    .map(|(minutes, order, filled)| Working {
        time: at(minutes),
        order,
        filled,
    });
    let fills = [
        (2, "ELY", Side::Buy, 100.0, 186.92, 1.00),
        (9, "PINE", Side::Buy, 400.0, 27.05, 1.60),
        (17, "ORBIT", Side::Sell, 40.0, 132.40, 1.00),
        (31, "ELY", Side::Buy, 200.0, 187.18, 1.20),
        (44, "NOVA", Side::Sell, 150.0, 62.31, 1.00),
    ]
    .map(|(minutes, symbol, side, quantity, price, fee)| Fill {
        time: at(minutes),
        symbol: symbol.into(),
        side,
        quantity,
        price,
        fee,
    });
    section(
        "PositionTable / OrderTable / TradeHistoryTable / PnLDisplay",
        "What is held and what it has made, orders still waiting with how much has filled, and each fill with its fee. Results read green for a gain and red for a loss.",
        cx,
    )
    .child(
        row()
            .gap_12()
            .child(PnLDisplay::new("pnl-day", "Today", (1_284.50, 0.0092), "USD"))
            .child(PnLDisplay::new("pnl-open", "Open positions", (4_910.20, 0.0371), "USD"))
            .child(PnLDisplay::new("pnl-month", "This month", (-2_046.75, -0.0148), "USD")),
    )
    .child(PositionTable::new("positions", positions))
    .child(OrderTable::new("orders", working).zone(TimeZone::UTC))
    .child(TradeHistoryTable::new("fills", fills).zone(TimeZone::UTC))
}

pub fn risk(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let leverage = keep("finance-leverage", || 5.0, window, cx);
    let now = *leverage.read(cx);
    let store = leverage.clone();
    let used = 21_500.0 * now / 5.0;
    section(
        "LeverageSlider / MarginIndicator / RiskMeter",
        "Leverage marked at the common steps; the margin it takes against the account, and the risk it carries. Each turns amber, then red, as it climbs. Drag the slider.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_12()
            .items_center()
            .child(
                div()
                    .w(px(360.))
                    .flex()
                    .flex_col()
                    .gap_8()
                    .child(
                        LeverageSlider::new("leverage", now, 100.0)
                            .on_change(move |value, _, cx| set(&store, value, cx)),
                    )
                    .child(MarginIndicator::new("margin", used.min(96_000.0), 96_000.0)),
            )
            .child(RiskMeter::new("risk", (now * 0.9 + 8.0).min(100.0))),
    )
}

pub fn options(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let spot = LAST;
    let strikes = (0..9).map(|ix| {
        let strike = 170.0 + ix as f64 * 5.0;
        let (call, put) = ((spot - strike).max(0.0), (strike - spot).max(0.0));
        let time = 3.2 - (strike - spot).abs() * 0.04;
        let quote = |inner: f64, volume: f64, volatility: f64| OptionQuote {
            bid: inner + time.max(0.3) - 0.05,
            ask: inner + time.max(0.3) + 0.05,
            volume,
            volatility,
        };
        Strike {
            strike,
            call: quote(
                call,
                400.0 + ix as f64 * 120.0,
                0.24 + (ix as f64 - 4.0).abs() * 0.01,
            ),
            put: quote(
                put,
                1_400.0 - ix as f64 * 110.0,
                0.26 + (ix as f64 - 4.0).abs() * 0.012,
            ),
        }
    });
    let picked = keep("finance-picked", || None::<SharedString>, window, cx);
    let (told, pick) = (picked.read(cx).clone(), picked.clone());
    let spread = [
        Leg {
            call: true,
            strike: 185.0,
            quantity: 1.0,
            premium: 4.10,
        },
        Leg {
            call: true,
            strike: 195.0,
            quantity: -1.0,
            premium: 1.05,
        },
    ];
    section(
        "OptionChain / GreeksTable / PayoffDiagram",
        "Calls to the left and puts to the right of each strike, the side in the money shaded and a line at the price; a position's greeks; and what a call spread pays at expiry. Hover the diagram to read any price.",
        cx,
    )
    .child(
        OptionChain::new("chain", strikes, spot).on_pick(move |strike, call, _, cx| {
            let words = format!("{} {strike:.0}", if call { "Call" } else { "Put" });
            set(&pick, Some(words.into()), cx)
        }),
    )
    .children(told.map(|words| Caption::new(format!("Picked: {words}."))))
    .child(div().w(px(640.)).child(GreeksTable::new([0.42, 0.031, -0.086, 0.121, 0.047])))
    .child(probe(
        "payoff",
        div()
            .w(px(WIDE))
            .child(PayoffDiagram::new("payoff", spread, (165.0, 215.0), spot)),
    ))
}
