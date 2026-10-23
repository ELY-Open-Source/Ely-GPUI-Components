use ely_gpui_component::{
    forms::TextInput,
    git::{
        Blame, BlameView, DiffStat, FileHistory, GitStatusBadge, PullRequest, PullRequestCard,
        PullState, ReviewComment, ReviewNote,
    },
    lists::GitStatus,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::work::history;
use crate::ui::{keep, row, section, set};

const CODE: &str = "impl Palette {\n    pub fn lookup(&self, name: &str, lift: f32) -> Color {\n        let base = self.colors.get(name).copied().unwrap_or(self.fallback);\n        base.lift(lift)\n    }\n\n    pub fn len(&self) -> usize {\n        self.colors.len()\n    }\n}";

pub fn blame(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let said = keep("git-blame-said", || None::<SharedString>, window, cx);
    let told = said.read(cx).clone();
    let blame = |commit: usize, author: &str, when: &str, subject: &str, age: f32| Blame {
        commit: history()[commit].id.clone(),
        author: author.to_string().into(),
        when: when.to_string().into(),
        subject: subject.to_string().into(),
        age,
    };
    let blames = [
        blame(5, "Sam Ortiz", "3 days ago", "First palette", 1.0),
        blame(2, "Mei Lin", "3 h ago", "Blend toward white by lift", 0.2),
        blame(
            1,
            "Ada Park",
            "1 h ago",
            "Keep palette order with a BTreeMap",
            0.1,
        ),
    ];
    let history_rows: Vec<_> = history()
        .into_iter()
        .zip([
            Some((12, 4)),
            Some((3, 3)),
            Some((9, 1)),
            Some((18, 0)),
            None,
            Some((40, 0)),
        ])
        .collect();
    let (pick, select) = (said.clone(), said.clone());
    section(
        "BlameView / GitBlameAnnotation / FileHistory / DiffStat / GitStatusBadge",
        "Who last changed each run of lines, the bar fading with age, the current line annotated; a file's commits on a line through them with what each changed; sizes as counts and five dots; statuses as letters.",
        cx,
    )
    .child(
        div().w(px(840.)).h(px(240.)).child(
            BlameView::new("git-blame", CODE, [0, 1, 1, 1, 0, 0, 2, 2, 0, 0], blames)
                .current(2)
                .on_commit(move |commit, _, cx| set(&pick, Some(format!("Picked {}.", &commit[..7]).into()), cx)),
        ),
    )
    .children(told.map(Caption::new))
    .child(
        div().w(px(520.)).h(px(360.)).child(
            FileHistory::new("git-file-history", std::rc::Rc::new(history_rows))
                .selected(history()[1].id.clone())
                .on_pick(move |commit, _, cx| set(&select, Some(format!("Opened {}.", &commit[..7]).into()), cx)),
        ),
    )
    .child(
        row()
            .gap_6()
            .child(DiffStat::new(120, 45))
            .child(DiffStat::new(3, 0))
            .child(DiffStat::new(0, 31))
            .children(
                [GitStatus::Modified, GitStatus::Added, GitStatus::Deleted, GitStatus::Untracked, GitStatus::Renamed, GitStatus::Conflicted]
                    .into_iter()
                    .enumerate()
                    .map(|(ix, status)| GitStatusBadge::new(("git-status", ix), status)),
            ),
    )
}

pub fn review(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let reply = window.use_keyed_state("git-reply", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Reply")
    });
    let resolved = keep("git-resolved", || false, window, cx);
    let notes = keep(
        "git-notes",
        || {
            vec![
                ReviewNote {
                    author: "Ada Park".into(),
                    when: "2 h ago".into(),
                    body: "Should a lift past one clamp, or panic? The docs say clamp.".into(),
                },
                ReviewNote {
                    author: "Mei Lin".into(),
                    when: "1 h ago".into(),
                    body: "Clamp. I added a test for 1.5 and for -0.2.".into(),
                },
            ]
        },
        window,
        cx,
    );
    let said = keep("git-pr-said", || None::<SharedString>, window, cx);
    let (now_resolved, now_notes, told) = (
        *resolved.read(cx),
        notes.read(cx).clone(),
        said.read(cx).clone(),
    );
    let (resolve, add, field, open, open_draft) = (
        resolved.clone(),
        notes.clone(),
        reply.clone(),
        said.clone(),
        said.clone(),
    );
    let pull = |number, title: &str, state, checks, added, removed, when: &str| PullRequest {
        number,
        title: title.to_string().into(),
        author: "Mei Lin".into(),
        head: "feature/lift".into(),
        base: "main".into(),
        state,
        checks,
        reviewers: vec!["Ada Park".into(), "Sam Ortiz".into()],
        comments: 4,
        added,
        removed,
        when: when.to_string().into(),
    };
    section(
        "PullRequestCard / ReviewComment",
        "A pull request at a glance: state, branches, checks, reviewers, comments and size. A review thread on a line: each comment, a reply, and resolve.",
        cx,
    )
    .child(
        div()
            .w(px(840.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                PullRequestCard::new("git-pr-open", pull(128, "Blend palette colors toward white by lift", PullState::Open, (11, 1, 0), 120, 45, "2 h ago"))
                    .on_open(move |_, cx| set(&open, Some("Opened #128.".into()), cx)),
            )
            .child(
                PullRequestCard::new("git-pr-draft", pull(131, "Try a gamma-aware lift", PullState::Draft, (3, 0, 2), 36, 8, "20 min ago"))
                    .on_open(move |_, cx| set(&open_draft, Some("Opened #131.".into()), cx)),
            ),
    )
    .children(told.map(Caption::new))
    .child(
        div().w(px(640.)).child(
            ReviewComment::new(
                "git-review",
                "src/palette.rs",
                48,
                "pub fn lookup(&self, name: &str, lift: f32) -> Color {",
                now_notes,
                &reply,
            )
            .resolved(now_resolved)
            .on_resolve(move |on, _, cx| set(&resolve, on, cx))
            .on_reply(move |_, cx| {
                let body = field.read(cx).text().trim().to_string();
                if body.is_empty() {
                    return;
                }
                let mut next = add.read(cx).clone();
                next.push(ReviewNote { author: "You".into(), when: "just now".into(), body: body.into() });
                set(&add, next, cx);
                field.update(cx, |field, cx| field.set_text("", cx));
            }),
        ),
    )
}
