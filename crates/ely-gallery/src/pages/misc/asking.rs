use ely_gpui_component::misc::{Flashcards, Poll, Question, Quiz, QuizQuestion, Survey};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::probe::probe;
use crate::ui::{section, specimen, specimens};

/// The gallery's poll: each option's count, and this viewer's vote.
struct Tally {
    votes: [u64; 3],
    voted: Option<usize>,
}

pub fn render(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tally = window.use_keyed_state("poll-tally", cx, |_, _| Tally {
        votes: [14, 9, 5],
        voted: None,
    });
    let (votes, voted) = (tally.read(cx).votes, tally.read(cx).voted);
    let poll = Poll::new(
        "poll",
        "Which chart should come next?",
        ["Sankey", "Sunburst", "Candlestick"],
        move |vote, _, cx| {
            tally.update(cx, |tally, cx| {
                match (vote, tally.voted) {
                    (Some(ix), _) => tally.votes[ix] += 1,
                    (None, Some(ix)) => tally.votes[ix] -= 1,
                    (None, None) => panic!("gallery poll: no vote to take back"),
                }
                tally.voted = vote;
                cx.notify();
            })
        },
    )
    .votes(votes);
    let poll = match voted {
        Some(ix) => poll.voted(ix),
        None => poll,
    };
    div()
        .child(
            section(
                "Poll / Survey",
                "A poll turns to results once the owner holds the vote, with Change vote on the same stop. A survey asks on one page and sends when every question has its answer.",
                cx,
            )
            .child(
                specimens()
                    .child(specimen("poll", probe("poll", div().w(px(280.0)).child(poll)), cx))
                    .child(specimen(
                        "survey",
                        div().w(px(320.0)).child(
                            Survey::new(
                                "survey",
                                [
                                    Question::Choice(
                                        "How often do you ship?".into(),
                                        vec!["Daily".into(), "Weekly".into(), "Monthly".into()],
                                    ),
                                    Question::Several(
                                        "Which parts do you use?".into(),
                                        vec!["Forms".into(), "Charts".into(), "Tables".into()],
                                    ),
                                    Question::Text("What should we build next?".into()),
                                ], |answers, _, _| log::info!("gallery: survey {answers:?}")),
                        ),
                        cx,
                    )),
            ),
        )
        .child(
            section(
                "Quiz / Flashcard",
                "A quiz checks each answer and shows the right one, then the score. Flashcards turn on a press and go round the deck.",
                cx,
            )
            .child(
                specimens()
                    .child(specimen(
                        "quiz",
                        probe(
                            "quiz",
                            div().w(px(280.0)).child(Quiz::new(
                                "quiz",
                                [
                                    QuizQuestion::new(
                                        "Which planet is the largest?",
                                        ["Mars", "Jupiter", "Venus"],
                                        1,
                                    ),
                                    QuizQuestion::new(
                                        "How many sides has a hexagon?",
                                        ["Five", "Six", "Eight"],
                                        1,
                                    ),
                                ],
                            )),
                        ),
                        cx,
                    ))
                    .child(specimen(
                        "flashcards",
                        probe(
                            "flashcards",
                            div().w(px(280.0)).child(Flashcards::new(
                                "flashcards",
                                [
                                    ("Hola", "Hello"),
                                    ("Gracias", "Thank you"),
                                    ("Buenos días", "Good morning"),
                                ],
                            )),
                        ),
                        cx,
                    )),
            ),
        )
}
