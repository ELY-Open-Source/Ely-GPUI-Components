//! A filtered menu's marked button is heard and pressed; a disabled button reads as not enabled.

use std::{process::Command, time::Duration};

use anyhow::{Context as _, Result, anyhow, bail};
use ely_gpui_component::{
    buttons::Button,
    menus::{Menu, MenuItem, SearchableMenu},
    primitives::{FocusScope, IconName},
};
use gpui::{
    AnyWindowHandle, AppContext, AsyncApp, Context, FocusHandle, IntoElement, KeyUpEvent,
    Keystroke, ParentElement, PlatformInput, Render, Styled, Window, WindowHandle, WindowOptions,
};
use serde_json::Value;

use super::{connect, hwnd, wake};
use crate::script;

const ROWS: [&str; 3] = ["Cut", "Paste", "Select all"];
const OFF: &str = "Unavailable action";
const PRESSED: Duration = Duration::from_secs(10);

/// A filtered menu headed by two buttons, counting Cut's runs.
struct Probe {
    root: FocusHandle,
    cuts: usize,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let cut = move |_: &mut Window, cx: &mut gpui::App| {
            view.update(cx, |probe, _| probe.cuts += 1);
        };
        let menu = Menu::new()
            .buttons([
                MenuItem::new(ROWS[0])
                    .icon(IconName::Scissors)
                    .on_click(cut),
                MenuItem::new(ROWS[1]).icon(IconName::Clipboard),
            ])
            .item(MenuItem::new(ROWS[2]));
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(Button::new("off", OFF).disabled(true))
            .child(SearchableMenu::new("probe", "Find", menu))
    }
}

/// Arrows to Cut, hearing each row, then presses it once.
pub(super) async fn check(cx: &mut AsyncApp) -> Result<()> {
    let window = cx.open_window(WindowOptions::default(), |window, cx| {
        let root = cx.focus_handle();
        window.focus(&root, cx);
        cx.new(|_| Probe { root, cuts: 0 })
    })?;
    let any: AnyWindowHandle = window.into();
    let mut client = connect(any, cx)?;
    let checked = drive(window, any, cx).await;
    client.kill().ok();
    checked
}

async fn drive(window: WindowHandle<Probe>, any: AnyWindowHandle, cx: &mut AsyncApp) -> Result<()> {
    wake(any, cx).await?;
    any.update(cx, |_, window, cx| window.focus_next(cx))?;
    key(any, "enter", cx).await?;
    let mut heard = String::new();
    for step in 1..=2 * ROWS.len() {
        key(any, "down", cx).await?;
        heard = focus_name(&tree(any, cx)?);
        if !ROWS.contains(&heard.as_str()) {
            bail!("after {step} down arrows a reader hears {heard:?}, not the marked row");
        }
        if heard == ROWS[0] {
            break;
        }
    }
    if heard != ROWS[0] {
        bail!("down arrows never marked {}", ROWS[0]);
    }
    automate(
        any,
        ROWS[0],
        "$press = $null; \
         if (-not $node.TryGetCurrentPattern(\
         [System.Windows.Automation.InvokePattern]::Pattern, [ref]$press)) { exit 3 }; \
         $press.Invoke()",
        cx,
    )
    .await?;
    let start = cx.background_executor().now();
    while window.update(cx, |probe, _, _| probe.cuts)? == 0 {
        if cx.background_executor().now() - start > PRESSED {
            bail!("Invoke on {} ran nothing in {PRESSED:?}", ROWS[0]);
        }
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
    }
    script::shown(any, cx).await?;
    let cuts = window.update(cx, |probe, _, _| probe.cuts)?;
    if cuts != 1 {
        bail!("Invoke on {} ran it {cuts} times", ROWS[0]);
    }
    if labels(&tree(any, cx)?).any(|label| label == ROWS[2]) {
        bail!("the menu stayed open after Invoke");
    }
    log::info!("audit: a filtered menu's marked button is heard and pressed once");
    automate(any, OFF, "if ($node.Current.IsEnabled) { exit 4 }", cx).await?;
    log::info!("audit: a disabled button reads as not enabled");
    Ok(())
}

async fn key(any: AnyWindowHandle, key: &'static str, cx: &mut AsyncApp) -> Result<()> {
    let keystroke = Keystroke::parse(key).map_err(|error| anyhow!("{key}: {error:?}"))?;
    any.update(cx, |_, window, cx| {
        window.dispatch_keystroke(keystroke.clone(), cx)
    })?;
    script::shown(any, cx).await?;
    // Buttons act on release, as on a real keyboard.
    let up = PlatformInput::KeyUp(KeyUpEvent { keystroke });
    any.update(cx, |_, window, cx| window.dispatch_event(up, cx))?;
    script::shown(any, cx).await
}

fn tree(any: AnyWindowHandle, cx: &mut AsyncApp) -> Result<Value> {
    let tree = any.update(cx, |_, window, _| window.debug_a11y_tree_json())?;
    Ok(serde_json::from_str(&tree.context("no tree was drawn")?)?)
}

/// What a reader announces: active descendant, else focused node.
fn focus_name(tree: &Value) -> String {
    tree["active_descendant_focus"]
        .as_str()
        .or(tree["gpui_focus"].as_str())
        .and_then(|id| tree["nodes"][id]["aria"]["label"].as_str())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn labels(tree: &Value) -> impl Iterator<Item = &str> {
    tree["nodes"]
        .as_object()
        .into_iter()
        .flat_map(|nodes| nodes.values())
        .filter_map(|node| node["aria"]["label"].as_str())
}

/// Runs `act` on the node named `name` through UI Automation, from another process.
async fn automate(any: AnyWindowHandle, name: &str, act: &str, cx: &mut AsyncApp) -> Result<()> {
    let hwnd = any.update(cx, |_, window, _| hwnd(window))??;
    let query = format!(
        "Add-Type -AssemblyName UIAutomationClient; Add-Type -AssemblyName UIAutomationTypes; \
         $root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]{hwnd}); \
         $named = New-Object System.Windows.Automation.PropertyCondition(\
         [System.Windows.Automation.AutomationElement]::NameProperty, '{name}'); \
         $node = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $named); \
         if (-not $node) {{ exit 2 }}; {act}"
    );
    let mut client = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &query])
        .spawn()
        .context("start a UI Automation client")?;
    // gpui serves the client on this thread: poll, never block.
    let start = cx.background_executor().now();
    let status = loop {
        if let Some(status) = client.try_wait()? {
            break status;
        }
        if cx.background_executor().now() - start > PRESSED {
            client.kill().ok();
            bail!("the UI Automation client did not finish in {PRESSED:?}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
    };
    match status.code() {
        Some(0) => Ok(()),
        Some(2) => bail!("UI Automation finds no node named {name}"),
        Some(3) => bail!("UI Automation offers no Invoke on {name}"),
        Some(4) => bail!("UI Automation reports {name} enabled"),
        _ => bail!("the UI Automation client failed: {status}"),
    }
}
