use ely_gpui_component::{
    editor::{
        BranchIndicator, CursorPosition, Encoding, IndentSettings, LanguageMode, LineEnding,
        LspState, LspStatus, NotificationBell,
    },
    shell::StatusBar,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const LANGUAGES: [&str; 3] = ["Rust", "TOML", "Markdown"];

fn lsp(step: usize) -> LspState {
    match step % 4 {
        0 => LspState::Starting,
        1 => LspState::Busy("indexing 214 files".into()),
        2 => LspState::Ready,
        _ => LspState::Failed("the server exited with code 101".into()),
    }
}

pub fn status(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let language = keep("status-language", || 0usize, window, cx);
    let crlf = keep("status-crlf", || false, window, cx);
    let spaces = keep("status-spaces", || true, window, cx);
    let server = keep("status-server", || 1usize, window, cx);
    let unread = keep("status-unread", || 3usize, window, cx);
    let said = keep("status-said", || None::<SharedString>, window, cx);
    let (now_language, now_crlf, now_spaces, now_server, now_unread, told) = (
        *language.read(cx),
        *crlf.read(cx),
        *spaces.read(cx),
        *server.read(cx),
        *unread.read(cx),
        said.read(cx).clone(),
    );
    let (go, sync, encode) = (said.clone(), said.clone(), said.clone());
    let bar = StatusBar::new()
        .left(
            BranchIndicator::new("status-branch", "feature/editor")
                .dirty(true)
                .sync(2, 1)
                .on_click(move |_, cx| set(&sync, Some("Pulled 1 commit, pushed 2.".into()), cx)),
        )
        .left(
            LspStatus::new("status-lsp", "rust-analyzer", lsp(now_server))
                .on_click(move |_, cx| set(&server, now_server + 1, cx)),
        )
        .right(
            CursorPosition::new("status-cursor", 48, 27)
                .selected(6)
                .on_click(move |_, cx| set(&go, Some("Go to line…".into()), cx)),
        )
        .right(
            IndentSettings::new("status-indent", now_spaces, 4)
                .on_click(move |_, cx| set(&spaces, !now_spaces, cx)),
        )
        .right(
            Encoding::new("status-encoding", "UTF-8")
                .on_click(move |_, cx| set(&encode, Some("Reopen with encoding…".into()), cx)),
        )
        .right(
            LineEnding::new("status-ending", now_crlf)
                .on_click(move |_, cx| set(&crlf, !now_crlf, cx)),
        )
        .right(
            LanguageMode::new("status-language", LANGUAGES[now_language])
                .on_click(move |_, cx| set(&language, (now_language + 1) % LANGUAGES.len(), cx)),
        )
        .right(
            NotificationBell::new("status-bell", now_unread)
                .on_click(move |_, cx| set(&unread, 0, cx)),
        );
    section(
        "CursorPosition / LanguageMode / Encoding / LineEnding / IndentSettings / BranchIndicator / LspStatus / NotificationBell",
        "The quiet row under the code. Each item says one thing and opens the one control that changes it: press the language, the line ending or the indent to switch them, the server to step through its states, the bell to read.",
        cx,
    )
    .child(probe("status", div().w(px(840.)).child(bar)))
    .children(told.map(Caption::new))
}
