use ely_gpui_component::{
    buttons::Button,
    forms::TextInput,
    git::{
        Branch, BranchList, BranchSelector, ChangeAction, Changed, ChangesList, Commit,
        CommitInput, CommitList, GitTag, GraphRow, Stash, StashAction, StashList, TagList, lanes,
    },
    lists::GitStatus,
    typography::Caption,
};
use std::rc::Rc;

use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, row, section, set},
};

fn changed(path: &str, status: GitStatus, added: usize, removed: usize) -> Changed {
    Changed {
        path: path.to_string().into(),
        status,
        lines: Some((added, removed)),
    }
}

fn field(
    key: &'static str,
    placeholder: &'static str,
    lines: Option<(usize, usize)>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        let field = TextInput::new(window, cx).placeholder(placeholder);
        match lines {
            Some((min, max)) => field.multi_line(min, max),
            None => field,
        }
    })
}

pub fn changes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let staged = keep(
        "git-staged",
        || vec![changed("src/palette.rs", GitStatus::Modified, 12, 4)],
        window,
        cx,
    );
    let unstaged = keep(
        "git-unstaged",
        || {
            vec![
                changed("src/theme/tokens.rs", GitStatus::Modified, 3, 1),
                changed("src/lift.rs", GitStatus::Added, 48, 0),
                changed("docs/old-palette.md", GitStatus::Deleted, 0, 31),
                changed("tests/palette.rs", GitStatus::Untracked, 22, 0),
            ]
        },
        window,
        cx,
    );
    let amend = keep("git-amend", || false, window, cx);
    let said = keep("git-said", || None::<SharedString>, window, cx);
    let subject = field("git-subject", "Summary of the change", None, window, cx);
    let body = field("git-body", "What changed and why", Some((3, 6)), window, cx);
    let (now_staged, now_unstaged, now_amend, told) = (
        staged.read(cx).clone(),
        unstaged.read(cx).clone(),
        *amend.read(cx),
        said.read(cx).clone(),
    );
    let (act_staged, act_unstaged, act_said) = (staged.clone(), unstaged.clone(), said.clone());
    let (all_staged, all_unstaged) = (staged.clone(), unstaged.clone());
    let (flip, done, commit_staged) = (amend.clone(), said.clone(), staged.clone());
    let count = now_staged.len();
    section(
        "ChangesList / CommitInput",
        "The work tree's changes, the staged apart, each with its status and size; hover a file to open, discard, stage or unstage it. The subject counts toward fifty; Cmd-Enter commits.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(420.)).child(
                    ChangesList::new("git-changes", now_staged, now_unstaged)
                        .on_action(move |path, _, action, _, cx| {
                            let (from, to) = match action {
                                ChangeAction::Stage => (&act_unstaged, &act_staged),
                                ChangeAction::Unstage => (&act_staged, &act_unstaged),
                                ChangeAction::Discard => {
                                    let left: Vec<Changed> = act_unstaged.read(cx).iter().filter(|file| file.path != *path).cloned().collect();
                                    set(&act_unstaged, left, cx);
                                    return;
                                }
                                ChangeAction::Open => {
                                    set(&act_said, Some(format!("Opened {path}.").into()), cx);
                                    return;
                                }
                            };
                            let moving: Vec<Changed> = from.read(cx).iter().filter(|file| file.path == *path).cloned().collect();
                            let left: Vec<Changed> = from.read(cx).iter().filter(|file| file.path != *path).cloned().collect();
                            let mut grown = to.read(cx).clone();
                            grown.extend(moving);
                            set(from, left, cx);
                            set(to, grown, cx);
                        })
                        .on_all(move |stage, _, cx| {
                            let (from, to) = if stage { (&all_unstaged, &all_staged) } else { (&all_staged, &all_unstaged) };
                            let mut grown = to.read(cx).clone();
                            grown.extend(from.read(cx).iter().cloned());
                            set(to, grown, cx);
                            set(from, Vec::new(), cx);
                        }),
                ),
            )
            .child(probe(
                "git-commit",
                div().w(px(380.)).child(
                    CommitInput::new("git-commit", &subject, &body, "main", count)
                        .amend(now_amend)
                        .on_amend(move |on, _, cx| set(&flip, on, cx))
                        .on_commit(move |_, cx| {
                            let files = commit_staged.read(cx).len();
                            set(&commit_staged, Vec::new(), cx);
                            set(&done, Some(format!("Committed {files} file{}.", if files == 1 { "" } else { "s" }).into()), cx);
                        }),
                ),
            )),
    )
    .children(told.map(Caption::new))
}

fn commit(
    id: &str,
    parents: &[&str],
    subject: &str,
    author: &str,
    when: &str,
    refs: &[&str],
) -> Commit {
    Commit {
        id: format!("{id}{}", "0".repeat(40 - id.len())).into(),
        parents: parents
            .iter()
            .map(|parent| format!("{parent}{}", "0".repeat(40 - parent.len())).into())
            .collect(),
        subject: subject.to_string().into(),
        author: author.to_string().into(),
        when: when.to_string().into(),
        refs: refs
            .iter()
            .map(|name| SharedString::from(name.to_string()))
            .collect(),
    }
}

/// A short history with a branch merged back.
pub(super) fn history() -> Vec<Commit> {
    vec![
        commit(
            "f4c1",
            &["e9a2", "d3b7"],
            "Merge branch 'feature/lift'",
            "Mei Lin",
            "just now",
            &["main", "origin/main"],
        ),
        commit(
            "e9a2",
            &["c8d0"],
            "Keep palette order with a BTreeMap",
            "Ada Park",
            "1 h ago",
            &[],
        ),
        commit(
            "d3b7",
            &["b21e"],
            "Blend toward white by lift",
            "Mei Lin",
            "3 h ago",
            &["feature/lift"],
        ),
        commit(
            "b21e",
            &["c8d0"],
            "Add Color::lift",
            "Mei Lin",
            "5 h ago",
            &[],
        ),
        commit(
            "c8d0",
            &["a6f3"],
            "Count tints in the palette",
            "Ada Park",
            "yesterday",
            &["v0.2.0"],
        ),
        commit(
            "a6f3",
            &[],
            "First palette",
            "Sam Ortiz",
            "3 days ago",
            &["v0.1.0"],
        ),
    ]
}

/// The sample history's lanes.
fn graph() -> Vec<GraphRow> {
    let commits = history();
    let parents: Vec<Vec<&str>> = commits
        .iter()
        .map(|commit| commit.parents.iter().map(|id| id.as_ref()).collect())
        .collect();
    lanes(
        commits
            .iter()
            .zip(&parents)
            .map(|(commit, parents)| (commit.id.as_ref(), parents.as_slice())),
    )
}

pub fn commits(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep("git-picked", || history()[0].id.clone(), window, cx);
    let now = picked.read(cx).clone();
    let pick = picked.clone();
    section(
        "CommitGraph / CommitList / CommitItem",
        "Commits newest first beside the lanes they take: a branch forks off, merges join, each lane in its own color; refs sit on their commits.",
        cx,
    )
    .child(
        div().w(px(840.)).h(px(320.)).child(
            CommitList::new("git-graph", Rc::new(history()))
                .graph(Rc::new(graph()))
                .selected(now)
                .on_pick(move |id, _, cx| set(&pick, id.clone(), cx)),
        ),
    )
}

fn branch(
    name: &str,
    remote: bool,
    current: bool,
    ahead: usize,
    behind: usize,
    subject: &str,
    when: &str,
) -> Branch {
    Branch {
        name: name.to_string().into(),
        remote,
        current,
        ahead,
        behind,
        subject: subject.to_string().into(),
        when: when.to_string().into(),
    }
}

fn branches() -> Vec<Branch> {
    vec![
        branch(
            "main",
            false,
            true,
            2,
            0,
            "Merge branch 'feature/lift'",
            "just now",
        ),
        branch(
            "feature/lift",
            false,
            false,
            0,
            1,
            "Blend toward white by lift",
            "3 h ago",
        ),
        branch(
            "fix/order",
            false,
            false,
            1,
            3,
            "Sort names before hashing",
            "2 days ago",
        ),
        branch(
            "origin/main",
            true,
            false,
            0,
            0,
            "Keep palette order with a BTreeMap",
            "1 h ago",
        ),
        branch(
            "origin/release",
            true,
            false,
            0,
            0,
            "Count tints in the palette",
            "yesterday",
        ),
    ]
}

pub fn refs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("git-branch-open", || false, window, cx);
    let said = keep("git-branch-said", || None::<SharedString>, window, cx);
    let stashes = keep(
        "git-stashes",
        || {
            vec![
                Stash {
                    message: "WIP on lift: try gamma".into(),
                    branch: "feature/lift".into(),
                    when: "2 h ago".into(),
                },
                Stash {
                    message: "half a theme toggle".into(),
                    branch: "main".into(),
                    when: "yesterday".into(),
                },
            ]
        },
        window,
        cx,
    );
    let (now_open, told, now_stashes) = (
        *open.read(cx),
        said.read(cx).clone(),
        stashes.read(cx).clone(),
    );
    let (opener, close, pick, create, switch, delete, stash_said, drop) = (
        open.clone(),
        open.clone(),
        said.clone(),
        said.clone(),
        said.clone(),
        said.clone(),
        said.clone(),
        stashes.clone(),
    );
    let tags = [
        GitTag {
            name: "v0.2.0".into(),
            message: Some("Tints and counts".into()),
            commit: history()[4].id.clone(),
            when: "yesterday".into(),
        },
        GitTag {
            name: "v0.1.0".into(),
            message: None,
            commit: history()[5].id.clone(),
            when: "3 days ago".into(),
        },
    ];
    section(
        "BranchSelector / BranchList / TagList / StashList",
        "Branches as a palette, local and remote apart, a new name offering a new branch; the lists show how far each branch is from its upstream, tags with their messages, and stashes to apply, pop or drop.",
        cx,
    )
    .child(row().child(Button::new("git-branch-button", "Switch branch…").on_click(move |_, _, cx| set(&opener, true, cx))))
    .children(told.map(Caption::new))
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(420.)).child(
                    BranchList::new("git-branches", branches().into_iter().filter(|branch| !branch.remote))
                        .on_switch(move |name, _, cx| set(&switch, Some(format!("Switched to {name}.").into()), cx))
                        .on_delete(move |name, _, cx| set(&delete, Some(format!("Deleted {name}.").into()), cx)),
                ),
            )
            .child(
                div()
                    .w(px(380.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(TagList::new("git-tags", tags))
                    .child(StashList::new("git-stash-list", now_stashes).on_action(move |ix, action, _, cx| {
                        let mut left = drop.read(cx).clone();
                        let stash = left.remove(ix);
                        if action != StashAction::Apply {
                            set(&drop, left, cx);
                        }
                        set(&stash_said, Some(format!("{action:?}: {}.", stash.message).into()), cx);
                    })),
            ),
    )
    .children(now_open.then(|| {
        BranchSelector::new("git-branch-palette", branches(), move |_, cx| set(&close, false, cx))
            .on_pick(move |name, _, cx| set(&pick, Some(format!("Switched to {name}.").into()), cx))
            .on_create(move |name, _, cx| set(&create, Some(format!("Created {name}.").into()), cx))
    }))
}
