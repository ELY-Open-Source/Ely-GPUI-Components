use std::time::Duration;

use ely_gpui_component::{
    charts::PieChart,
    data_display::Gauge,
    feedback::Countdown,
    finance::{
        CryptoWalletCard, CurrencyConverter, Dividend, DividendTable, FinancialStatementTable,
        PerformanceChart, PortfolioSummary, Settled, Statement, TransactionList, Transfer,
    },
    theme::TextSize,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, civil::date, tz::TimeZone};
use web_time::Instant;

use crate::ui::{keep, noise, row, section, set};

const HOLDINGS: [(&str, f64); 5] = [
    ("ELY", 48_210.0),
    ("SOLACE", 31_050.0),
    ("Bonds", 26_400.0),
    ("NOVA", 12_480.0),
    ("PINE", 8_196.0),
];

pub fn portfolio(cx: &mut App) -> impl IntoElement + use<> {
    let cash = 9_864.0;
    let worth = HOLDINGS.iter().map(|(_, worth)| worth).sum::<f64>() + cash;
    let summary = HOLDINGS.iter().fold(
        PortfolioSummary::new(
            "portfolio",
            worth,
            ((1_284.50, 0.0092), (18_402.3, 0.1553)),
            cash,
            "USD",
        ),
        |summary, (name, worth)| summary.holding(*name, *worth),
    );
    let allocation = HOLDINGS
        .iter()
        .fold(PieChart::new("allocation"), |pie, (name, worth)| {
            pie.slice(*name, *worth)
        })
        .slice("Cash", cash)
        .donut("$136 K", "Invested");
    let months = [
        "Oct", "Nov", "Dec", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
    ];
    let mut rng = noise(83);
    let (mut equity, mut index) = (100.0, 100.0);
    let (equity, index): (Vec<f64>, Vec<f64>) = months
        .iter()
        .map(|_| {
            equity *= 1.0 + (rng() - 0.38) * 0.07;
            index *= 1.0 + (rng() - 0.42) * 0.04;
            (equity, index)
        })
        .unzip();
    section(
        "PortfolioSummary / AssetAllocationChart / PerformanceChart / SentimentGauge",
        "What the portfolio is worth and how it moved, split across holdings; the same split as a donut; its growth against the market with each fall from a high below; and the market's mood on a dial.",
        cx,
    )
    .child(summary)
    .child(
        div()
            .flex()
            .gap_8()
            .items_center()
            .child(div().w(px(420.)).child(allocation))
            .child(Gauge::new("sentiment", "Sentiment", 64.0, 0.0, 100.0).decimals(0)),
    )
    .child(PerformanceChart::new("performance", months, equity).benchmark("EGX 500", index))
}

pub fn statements(cx: &mut App) -> impl IntoElement + use<> {
    let dividends = [
        (date(2025, 11, 7), date(2025, 11, 14), 0.24),
        (date(2026, 2, 6), date(2026, 2, 13), 0.24),
        (date(2026, 5, 8), date(2026, 5, 15), 0.26),
        (date(2026, 8, 7), date(2026, 8, 14), 0.26),
    ]
    .map(|(ex, pay, amount)| Dividend {
        ex,
        pay,
        amount,
        dividend_yield: amount * 4.0 / 187.25,
        frequency: "Quarterly".into(),
    });
    let line = |name: &str, values: [f64; 4]| Statement::new(name.to_string(), values);
    let lines = [
        line("Revenue", [18_420.0, 19_106.0, 20_388.0, 21_950.0]).lines([
            line("Products", [11_210.0, 11_380.0, 11_902.0, 12_430.0]),
            line("Services", [7_210.0, 7_726.0, 8_486.0, 9_520.0]),
        ]),
        line("Cost of revenue", [-7_920.0, -8_120.0, -8_480.0, -8_940.0]),
        line(
            "Operating expenses",
            [-5_880.0, -6_010.0, -6_240.0, -6_530.0],
        )
        .lines([
            line("Research", [-3_120.0, -3_210.0, -3_360.0, -3_520.0]),
            line("Sales and admin", [-2_760.0, -2_800.0, -2_880.0, -3_010.0]),
        ]),
        line("Operating income", [4_620.0, 4_976.0, 5_668.0, 6_480.0]),
        line("Net income", [3_710.0, 3_990.0, 4_560.0, 5_190.0]),
    ];
    section(
        "DividendTable / FinancialStatementTable",
        "Dividends by date with their yield, and a statement by quarter whose lines open into their parts.",
        cx,
    )
    .child(DividendTable::new("dividends", dividends, "USD"))
    .child(FinancialStatementTable::new(
        "statement",
        ["Q4 2025", "Q1 2026", "Q2 2026", "Q3 2026"],
        lines,
    ))
}

/// Demo rates in US dollars per unit; codes outside the table get a steady made-up rate.
fn dollars(code: &str) -> f64 {
    match code {
        "USD" => 1.0,
        "EUR" => 1.09,
        "GBP" => 1.27,
        "CHF" => 1.13,
        "CAD" => 0.73,
        "AUD" => 0.66,
        "SGD" => 0.75,
        "HKD" => 0.128,
        "CNY" => 0.14,
        "INR" => 0.012,
        "JPY" => 0.0067,
        "KRW" => 0.00073,
        _ => {
            let seed = code
                .bytes()
                .fold(7, |seed: u64, byte| seed * 31 + u64::from(byte));
            0.001 + noise(seed)() * 0.5
        }
    }
}

pub fn money(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let exchange = keep(
        "finance-exchange",
        || {
            (
                1_000.0,
                SharedString::from("USD"),
                SharedString::from("EUR"),
            )
        },
        window,
        cx,
    );
    let (amount, from, to) = exchange.read(cx).clone();
    let store = exchange.clone();
    let candle = keep(
        "finance-candle",
        || Instant::now() + Duration::from_secs(222),
        window,
        cx,
    );
    let until = *candle.read(cx);
    let restart = candle.clone();
    let transfers = [
        (15, 40, "Salary", 6_250.0, "USD", Settled::Done),
        (14, 5, "Card ending 4417", -86.40, "USD", Settled::Pending),
        (11, 52, "Wallet bc1q…yg3c", 0.0421, "BTC", Settled::Done),
        (9, 18, "Rent", -2_400.0, "USD", Settled::Done),
        (8, 2, "Exchange withdrawal", -500.0, "EUR", Settled::Failed),
    ]
    .map(|(hour, minute, party, amount, unit, settled)| Transfer {
        time: Timestamp::from_second(1_790_294_400 + hour * 3_600 + minute * 60).expect("a time"),
        party: party.into(),
        amount,
        unit: unit.into(),
        settled,
    });
    section(
        "CurrencyConverter / CryptoWalletCard / TransactionList / Candle Countdown",
        "An amount in another currency, at an illustrative rate; a wallet with its balance and an address to scan; transfers in and out with whether each settled; and the time left in the current candle.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(460.)).child(
                    CurrencyConverter::new("converter", amount, (&from, &to), dollars(&from) / dollars(&to))
                        .on_change(move |amount, from, to, _, cx| set(&store, (amount, from, to), cx)),
                ),
            )
            .child(div().flex_1().child(CryptoWalletCard::new(
                "wallet",
                ("Savings", "Bitcoin"),
                (0.8412, "BTC"),
                (54_310.22, "USD"),
                "bc1q9h7garjy5kd5nwmrq2z6yufl0yq0p0a9w6yg3c",
            ))),
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(div().flex_1().child(TransactionList::new(transfers).zone(TimeZone::UTC)))
            .child(
                div()
                    .w(px(220.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Caption::new("5m candle closes in"))
                    .child(row().child(Countdown::new("candle", until).size(TextSize::Xl).on_done(
                        move |_, cx| set(&restart, Instant::now() + Duration::from_secs(300), cx),
                    ))),
            ),
    )
}
