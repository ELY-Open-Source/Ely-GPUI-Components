//! Tabs through every page with gpui's accessibility tree on and lists each stop a screen reader cannot name.

use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    time::Duration,
};

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::theme::Theme;
use gpui::{AnyWindowHandle, App, AsyncApp, FocusHandle, Window, WindowHandle};
use serde_json::Value;

use crate::{pages, script, shell::Gallery};

const SETTLE: Duration = Duration::from_millis(700);
const WAKE: Duration = Duration::from_secs(20);
/// More stops than any page holds; past it the walk has lost its way.
const MOST_STOPS: usize = 2000;

/// One Tab stop as the tree shows it.
struct Stop {
    page: &'static str,
    index: usize,
    fault: Option<&'static str>,
    role: String,
    name: String,
    place: String,
}

/// Audits one page, or all, writes the report, then quits; exits 1 on any fault.
pub fn run(window: WindowHandle<Gallery>, report: PathBuf, only: Option<usize>, cx: &mut App) {
    Theme::update(cx, |theme| theme.reduced_motion = true);
    cx.spawn(async move |cx| {
        let mut stops = Vec::new();
        let walked = walk_all(window, only, &mut stops, cx).await;
        // A walk that fails still reports the stops it read.
        let outcome = write(&report, &stops).and_then(|faults| walked.map(|()| faults));
        match outcome {
            Ok(0) => {
                log::info!("audit: every stop has a role and a name");
                cx.update(|cx| cx.quit());
            }
            Ok(faults) => {
                log::error!(
                    "audit: {faults} stops a screen reader cannot name; see {}",
                    report.display()
                );
                std::process::exit(1);
            }
            Err(error) => {
                log::error!(
                    "audit failed: {error:#}; stops so far in {}",
                    report.display()
                );
                std::process::exit(1);
            }
        }
    })
    .detach();
}

async fn walk_all(
    window: WindowHandle<Gallery>,
    only: Option<usize>,
    stops: &mut Vec<Stop>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let mut client = connect(window, cx)?;
    wake(window, cx).await?;
    for (ix, page) in pages::ALL.iter().enumerate() {
        if only.is_some_and(|only| only != ix) {
            continue;
        }
        window.update(cx, |gallery, window, cx| {
            gallery.select(ix, cx);
            gallery.fresh(cx);
            gallery.focus_root(window, cx);
        })?;
        cx.background_executor().timer(SETTLE).await;
        let before = stops.len();
        walk(window, page.slug, stops, cx).await?;
        log::info!("audit: {} has {} stops", page.slug, stops.len() - before);
    }
    client.kill().ok();
    Ok(())
}

/// A UI Automation client in another process, so gpui builds its tree.
fn connect(window: WindowHandle<Gallery>, cx: &mut AsyncApp) -> Result<Child> {
    let hwnd = window.update(cx, |_, window, _| hwnd(window))??;
    let query = format!(
        "Add-Type -AssemblyName UIAutomationClient; \
         $root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]{hwnd}); \
         $null = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, \
         [System.Windows.Automation.Condition]::TrueCondition)"
    );
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &query])
        .spawn()
        .context("start a UI Automation client")
}

fn hwnd(window: &Window) -> Result<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("window has no native handle: {error:?}"))?;
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        bail!("window is not a Win32 window");
    };
    Ok(win32.hwnd.get())
}

/// Waits for the client's first request to turn gpui's tree on.
async fn wake(window: WindowHandle<Gallery>, cx: &mut AsyncApp) -> Result<()> {
    let start = cx.background_executor().now();
    loop {
        if window.update(cx, |_, window, _| window.is_a11y_active())? {
            return Ok(());
        }
        if cx.background_executor().now() - start > WAKE {
            bail!("no accessibility client reached the window in {WAKE:?}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
    }
}

/// Steps gpui's Tab order from the root until the first stop comes again.
async fn walk(
    window: WindowHandle<Gallery>,
    page: &'static str,
    stops: &mut Vec<Stop>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let any: AnyWindowHandle = window.into();
    let mut seen: Vec<FocusHandle> = Vec::new();
    let mut named = (String::from("the page's start"), 0);
    for _ in 0..MOST_STOPS {
        let index = stops.iter().filter(|stop| stop.page == page).count();
        // gpui's own Tab order, past traps and editors that keep Tab.
        let focused = any.update(cx, |_, window, cx| {
            window.focus_next(cx);
            window.focused(cx)
        })?;
        script::shown(any, cx).await?;
        let Some(focused) = focused else {
            bail!("{page}: nothing focused at stop {index}");
        };
        match seen.iter().position(|handle| *handle == focused) {
            Some(0) => return Ok(()),
            Some(at) => bail!("{page}: the Tab order came back to stop {at}, not the first"),
            None => {}
        }
        seen.push(focused);
        if !window.update(cx, |_, window, _| window.is_a11y_active())? {
            bail!("{page}: the accessibility tree turned off at stop {index}");
        }
        let tree = window.update(cx, |_, window, _| window.debug_a11y_tree_json())?;
        let tree: Value = serde_json::from_str(&tree.context("no tree was drawn")?)?;
        let stop = read(page, index, &tree, &named);
        named = match stop.fault {
            Some("no node") => (named.0, named.1 + 1),
            _ => (format!("{} \"{}\" {}", stop.role, stop.name, stop.place), 1),
        };
        stops.push(stop);
    }
    bail!("{page}: the Tab order did not come round in {MOST_STOPS} stops")
}

/// The focused node, or the fault that hides it.
fn read(page: &'static str, index: usize, tree: &Value, named: &(String, usize)) -> Stop {
    // A reader announces the active descendant over the focused box.
    let focus = tree["active_descendant_focus"]
        .as_str()
        .or(tree["gpui_focus"].as_str());
    let Some(node) = focus.map(|id| &tree["nodes"][id]) else {
        return Stop {
            page,
            index,
            fault: Some("no node"),
            role: String::new(),
            name: String::new(),
            place: format!("{} past {}", named.1, named.0),
        };
    };
    let role = node["aria"]["role"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let name = node["aria"]["label"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();
    Stop {
        page,
        index,
        fault: name.is_empty().then_some("no name"),
        role,
        name,
        // The element id names who drew the node.
        place: format!(
            "{} {}",
            node["source_location"].as_str().unwrap_or_default(),
            node["element_id"].as_str().unwrap_or_default()
        ),
    }
}

/// Writes every stop and returns how many fault.
fn write(report: &Path, stops: &[Stop]) -> Result<usize> {
    let mut out = String::from("page\tstop\tfault\trole\tname\tplace\n");
    for stop in stops {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            stop.page,
            stop.index,
            stop.fault.unwrap_or("ok"),
            stop.role,
            stop.name.replace(['\t', '\n'], " "),
            stop.place
        ));
    }
    std::fs::write(report, out).with_context(|| format!("write {}", report.display()))?;
    Ok(stops.iter().filter(|stop| stop.fault.is_some()).count())
}
