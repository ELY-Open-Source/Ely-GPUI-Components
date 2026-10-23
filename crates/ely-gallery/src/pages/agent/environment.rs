use std::time::Duration;

use ely_gpui_component::{
    agent::{
        Checkpoint, CheckpointList, McpServer, McpServerList, Memory, MemoryPanel, RegisteredTool,
        SandboxState, SandboxStatus, ServerState, ToolAccess, ToolRegistry,
    },
    forms::{SearchInput, TextInput},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn sandbox_section(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "SandboxStatus",
        "Where an agent's sandbox stands: a dot and a word, what it uses of what it has, and how long it has run. A dot rings while it starts or works.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_4()
            .child(
                div().w(px(320.)).child(
                    SandboxStatus::new("agent-sandbox", "python 3.12 · 2 vCPU", SandboxState::Ready)
                        .cpu(0.18)
                        .memory(1_240_000_000, 4_294_967_296)
                        .uptime(Duration::from_secs(47 * 60)),
                ),
            )
            .child(
                div().w(px(280.)).child(
                    SandboxStatus::new("agent-sandbox-starting", "node 22 · 1 vCPU", SandboxState::Starting),
                ),
            ),
    )
}

fn servers() -> Vec<McpServer> {
    [
        (
            "files",
            "Filesystem",
            "npx @modelcontextprotocol/server-filesystem ~/ely",
            ServerState::Connected,
            11,
        ),
        (
            "github",
            "GitHub",
            "https://api.githubcopilot.com/mcp",
            ServerState::Connected,
            38,
        ),
        (
            "figma",
            "Figma",
            "Connecting to the desktop app…",
            ServerState::Connecting,
            0,
        ),
        (
            "postgres",
            "Postgres",
            "Refused: password authentication failed",
            ServerState::Failed,
            0,
        ),
        (
            "linear",
            "Linear",
            "https://mcp.linear.app/sse",
            ServerState::Off,
            24,
        ),
    ]
    .into_iter()
    .map(|(key, name, detail, state, tools)| McpServer {
        key: key.into(),
        name: name.into(),
        detail: detail.into(),
        state,
        tools,
    })
    .collect()
}

fn tools() -> Vec<RegisteredTool> {
    [
        (
            "read_file",
            "Filesystem",
            "Reads a file under the project",
            ToolAccess::Allow,
        ),
        (
            "write_file",
            "Filesystem",
            "Writes or replaces a file",
            ToolAccess::Ask,
        ),
        (
            "run_command",
            "Filesystem",
            "Runs a shell command in the sandbox",
            ToolAccess::Ask,
        ),
        (
            "create_issue",
            "GitHub",
            "Opens an issue in a repository",
            ToolAccess::Ask,
        ),
        (
            "merge_pull_request",
            "GitHub",
            "Merges a pull request",
            ToolAccess::Deny,
        ),
        (
            "web_search",
            "Built in",
            "Searches the web and cites sources",
            ToolAccess::Allow,
        ),
    ]
    .into_iter()
    .map(|(name, source, description, access)| RegisteredTool {
        name: name.into(),
        source: source.into(),
        description: description.into(),
        access,
    })
    .collect()
}

pub fn connectors_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let list = keep("agent-servers", servers, window, cx);
    let registry = keep("agent-tools", tools, window, cx);
    let query = window.use_keyed_state("agent-find-tool", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Find a tool")
    });
    let (now_list, now_tools, now_query) = (
        list.read(cx).clone(),
        registry.read(cx).clone(),
        query.read(cx).text().to_string(),
    );
    section(
        "MCPServerList / ConnectorList / ToolRegistry Panel",
        "Servers and connectors the agent can reach, each with where it stands and a switch; and every tool they bring, found by name, each allowed, asked for first, or denied.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(420.)).child(McpServerList::new("agent-mcp", now_list).on_toggle(move |key, on, _, cx| {
                    let mut next = list.read(cx).clone();
                    let server = next.iter_mut().find(|server| &server.key == key).expect("a listed server");
                    server.state = if on { ServerState::Connected } else { ServerState::Off };
                    set(&list, next, cx)
                })),
            )
            .child(
                div()
                    .w(px(460.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(SearchInput::new("agent-find-tool", &query))
                    .child(
                        ToolRegistry::new("agent-registry", now_tools)
                            .query(now_query)
                            .on_access(move |ix, access, _, cx| {
                                let mut next = registry.read(cx).clone();
                                next[ix].access = access;
                                set(&registry, next, cx)
                            }),
                    ),
            ),
    )
}

pub fn memory_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let now = Timestamp::now();
    let kept = keep(
        "agent-memories",
        || {
            vec![
                Memory {
                    key: "tone".into(),
                    text: "Prefers plain words and short sentences in notes.".into(),
                    source: Some("Design review".into()),
                    at: now - 72.hours(),
                },
                Memory {
                    key: "lift".into(),
                    text: "Dark themes lift accents by 18%; surfaces by 4%.".into(),
                    source: Some("Lift the dark theme".into()),
                    at: now - 5.hours(),
                },
            ]
        },
        window,
        cx,
    );
    let field = window.use_keyed_state("agent-memory-field", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Something to keep")
    });
    let points = keep(
        "agent-checkpoints",
        || {
            [
                ("Read the palette", 0, 40),
                ("Measured contrast", 0, 32),
                ("Lifted the accents", 2, 18),
                ("Ran the tests", 2, 9),
                ("Fixed the dark lift", 3, 2),
            ]
            .into_iter()
            .map(|(label, files, minutes)| Checkpoint {
                key: SharedString::from(label),
                label: label.into(),
                at: now - (minutes as i64).minutes(),
                files,
            })
            .collect::<Vec<_>>()
        },
        window,
        cx,
    );
    let at = keep("agent-checkpoint", || 4_usize, window, cx);
    let (now_kept, now_points, now_at) =
        (kept.read(cx).clone(), points.read(cx).clone(), *at.read(cx));
    let (added, forgot) = (kept.clone(), kept);
    section(
        "MemoryPanel / CheckpointList / Rewind",
        "What an agent keeps between conversations, a line to add to it and a way to forget; and the points it can go back to, newest first, the one it stands on marked.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(400.)).child(
                    MemoryPanel::new("agent-memory", &field, now_kept)
                        .on_add(move |text, _, cx| {
                            let mut next = added.read(cx).clone();
                            let key = SharedString::from(format!("kept-{}", next.len()));
                            next.insert(0, Memory { key, text: text.to_string().into(), source: None, at: Timestamp::now() });
                            set(&added, next, cx)
                        })
                        .on_forget(move |key, _, cx| {
                            let mut next = forgot.read(cx).clone();
                            next.retain(|memory| &memory.key != key);
                            set(&forgot, next, cx)
                        }),
                ),
            )
            .child(probe(
                "agent-checkpoints",
                div().w(px(400.)).child(
                    CheckpointList::new("agent-points", now_points, now_at)
                        .on_rewind(move |ix, _, cx| set(&at, ix, cx)),
                ),
            )),
    )
}
