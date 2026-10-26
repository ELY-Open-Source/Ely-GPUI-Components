use std::{path::PathBuf, time::Duration};

use anyhow::{Context as _, Result, bail};
use futures::{StreamExt as _, channel::mpsc};
use gpui::{App, Global, Task};
use notify::{
    EventKind, RecommendedWatcher, RecursiveMode, Watcher as _,
    event::{AccessKind, AccessMode},
};

use super::{OmarchyTheme, read_omarchy_theme};
use crate::theme::{Mode, Theme};

/// The quiet a switch ends with: Omarchy removes `theme`, then moves `next-theme` in and writes `theme.name`.
const SETTLE: Duration = Duration::from_millis(250);

/// The folder followed, the theme worn, how often it was read, and what watches it.
pub(super) struct Following {
    current: PathBuf,
    worn: Option<OmarchyTheme>,
    pub(super) reads: usize,
    _watcher: RecommendedWatcher,
    _task: Task<()>,
}

impl Global for Following {}

/// Wears Omarchy's theme from `current` now and after each switch, on the theme's own mode; Ely's palettes stand in for the other mode and while Omarchy has no colors, the owner's put aside. A theme that cannot be read logs an error and the colors stay. Fails while following already, or on a folder that cannot be watched.
///
/// Platforms: macOS, Windows, Linux.
pub fn follow_omarchy(current: PathBuf, cx: &mut App) -> Result<()> {
    if let Some(following) = cx.try_global::<Following>() {
        bail!("omarchy: following {} already", following.current.display());
    }
    let (sender, mut changes) = mpsc::unbounded();
    let mut watcher =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
            Ok(event) if is_read(&event.kind) => {}
            Ok(_) => {
                // Closed once stopped.
                let _ = sender.unbounded_send(());
            }
            Err(error) => log::error!("omarchy: watching failed: {error}"),
        })
        .context("omarchy: starting the file watcher")?;
    watcher
        .watch(&current, RecursiveMode::NonRecursive)
        .with_context(|| format!("omarchy: watching {}", current.display()))?;
    let task = cx.spawn(async move |cx| {
        while changes.next().await.is_some() {
            loop {
                cx.background_executor().timer(SETTLE).await;
                let mut more = false;
                while changes.try_recv().is_ok() {
                    more = true;
                }
                if !more {
                    break;
                }
            }
            cx.update(|cx| reload(false, cx));
        }
    });
    log::info!("omarchy: following {}", current.display());
    cx.set_global(Following {
        current,
        worn: None,
        reads: 0,
        _watcher: watcher,
        _task: task,
    });
    Theme::set_palette(Mode::Light, None, cx);
    Theme::set_palette(Mode::Dark, None, cx);
    reload(true, cx);
    Ok(())
}

/// Stops following; both modes get Ely's palettes back, and the mode stays the owner's to set.
///
/// Platforms: macOS, Windows, Linux.
pub fn stop_omarchy(cx: &mut App) {
    if !cx.has_global::<Following>() {
        log::info!("omarchy: not following, nothing to stop");
        return;
    }
    let following = cx.remove_global::<Following>();
    log::info!("omarchy: stopped following {}", following.current.display());
    Theme::set_palette(Mode::Light, None, cx);
    Theme::set_palette(Mode::Dark, None, cx);
}

/// Inotify reports opens and reads, ours too; a written close is a change.
fn is_read(kind: &EventKind) -> bool {
    matches!(kind, EventKind::Access(access) if *access != AccessKind::Close(AccessMode::Write))
}

fn reload(at_once: bool, cx: &mut App) {
    let following = cx.global_mut::<Following>();
    following.reads += 1;
    let (current, reads) = (following.current.clone(), following.reads);
    log::debug!("omarchy: reading {}, read {reads}", current.display());
    match read_omarchy_theme(&current) {
        Ok(Some(theme)) => {
            let name = theme.name.clone().unwrap_or_else(|| "unnamed".into());
            if cx.global::<Following>().worn.as_ref() == Some(&theme) {
                log::debug!("omarchy: {name} unchanged");
                return;
            }
            let other = match theme.mode {
                Mode::Light => Mode::Dark,
                Mode::Dark => Mode::Light,
            };
            Theme::set_palette(theme.mode, Some(theme.colors.clone()), cx);
            Theme::set_palette(other, None, cx);
            match at_once {
                true => Theme::set_mode_now(theme.mode, cx),
                false => Theme::set_mode(theme.mode, cx),
            }
            log::info!("omarchy: wearing {name}, {:?}", theme.mode);
            cx.global_mut::<Following>().worn = Some(theme);
        }
        Ok(None) => {
            if cx.global_mut::<Following>().worn.take().is_some() {
                Theme::set_palette(Mode::Light, None, cx);
                Theme::set_palette(Mode::Dark, None, cx);
            }
            log::info!(
                "omarchy: {} holds no colors.toml, Ely's palettes show",
                current.display()
            );
        }
        Err(error) => log::error!("{error:#}; the colors stay"),
    }
}

#[cfg(test)]
mod tests {
    use notify::{
        EventKind,
        event::{AccessKind, AccessMode, CreateKind, ModifyKind, RemoveKind, RenameMode},
    };

    use super::is_read;

    #[test]
    fn opens_and_reads_are_no_change_and_a_written_close_is_one() {
        for read in [
            AccessKind::Open(AccessMode::Any),
            AccessKind::Read,
            AccessKind::Close(AccessMode::Read),
        ] {
            assert!(is_read(&EventKind::Access(read)), "{read:?}");
        }
        for change in [
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            EventKind::Create(CreateKind::Folder),
            EventKind::Remove(RemoveKind::Folder),
            EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        ] {
            assert!(!is_read(&change), "{change:?}");
        }
    }
}
