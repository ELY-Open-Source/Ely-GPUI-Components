//! The containers: each takes children, and each component that takes children is here.

use std::{fs, path::Path};

use gpui::ParentElement;

fn holds<T: ParentElement>() {}

#[test]
fn every_listed_container_takes_children() {
    holds::<crate::agent::ToolCallGroup>();
    holds::<crate::buttons::ActionBar>();
    holds::<crate::canvas::FlowChart>();
    holds::<crate::canvas::InfiniteCanvas>();
    holds::<crate::chat::MessageBubble>();
    holds::<crate::chat::MessageFooter>();
    holds::<crate::chat::WelcomeScreen>();
    holds::<crate::collab::FollowMode>();
    holds::<crate::dashboard::DashboardCard>();
    holds::<crate::data_display::Carousel>();
    holds::<crate::data_display::TimelineItem>();
    holds::<crate::data_display::Watermark>();
    holds::<crate::documents::ZenMode>();
    holds::<crate::feedback::Callout>();
    holds::<crate::feedback::ConfirmationCard>();
    holds::<crate::finance::MultiChartLayout>();
    holds::<crate::forms::FieldArray>();
    holds::<crate::forms::Form>();
    holds::<crate::forms::FormField>();
    holds::<crate::forms::FormSection>();
    holds::<crate::forms::InlineForm>();
    holds::<crate::interaction::Resizable>();
    holds::<crate::interaction::ScrollSync>();
    holds::<crate::layout::AccordionItem>();
    holds::<crate::layout::AppShell>();
    holds::<crate::layout::AspectRatio>();
    holds::<crate::layout::Card>();
    holds::<crate::layout::Collapsible>();
    holds::<crate::layout::Container>();
    holds::<crate::layout::Drawer>();
    holds::<crate::layout::Fieldset>();
    holds::<crate::layout::FloatingPanel>();
    holds::<crate::layout::Frame>();
    holds::<crate::layout::Masonry>();
    holds::<crate::layout::Page>();
    holds::<crate::layout::Panel>();
    holds::<crate::layout::ScrollArea>();
    holds::<crate::layout::Section>();
    holds::<crate::layout::Sheet>();
    holds::<crate::layout::Sidebar>();
    holds::<crate::layout::SimpleGrid>();
    holds::<crate::layout::SplitPane>();
    holds::<crate::layout::StickyHeader>();
    holds::<crate::layout::Well>();
    holds::<crate::layout::ZStack>();
    holds::<crate::lists::InfiniteList>();
    holds::<crate::lists::List>();
    holds::<crate::mail::MailReader>();
    holds::<crate::mail::QuotedText>();
    holds::<crate::maps::MapPopup>();
    holds::<crate::menus::ContextMenu>();
    holds::<crate::menus::PieMenu>();
    holds::<crate::messaging::ChatMessage>();
    holds::<crate::messaging::ScreenShareView>();
    holds::<crate::messaging::ThreadPanel>();
    holds::<crate::messaging::VideoCallGrid>();
    holds::<crate::misc::ImportDialog>();
    holds::<crate::motion::AnimatedGradient>();
    holds::<crate::motion::Blink>();
    holds::<crate::motion::Flash<u32>>();
    holds::<crate::motion::Glow>();
    holds::<crate::motion::LoadingOverlay>();
    holds::<crate::motion::ParticleBackground>();
    holds::<crate::motion::Pulse>();
    holds::<crate::motion::Ripple>();
    holds::<crate::motion::Shake<u32>>();
    holds::<crate::motion::Shimmer>();
    holds::<crate::motion::Stagger>();
    holds::<crate::motion::Transition>();
    holds::<crate::navigation::Anchor>();
    holds::<crate::navigation::NavGroup>();
    holds::<crate::overlays::Dialog>();
    holds::<crate::overlays::FloatingToolbar>();
    holds::<crate::overlays::Peek>();
    holds::<crate::primitives::Backdrop>();
    holds::<crate::primitives::FocusScope>();
    holds::<crate::primitives::IntersectionObserver>();
    holds::<crate::primitives::Measure>();
    holds::<crate::settings::SettingsSection>();
    holds::<crate::shell::ResizeBorder>();
    holds::<crate::shell::ShellColumn>();
    holds::<crate::shell::TitleBar>();
    holds::<crate::shell::Toolbar>();
    holds::<crate::shell::ToolbarGroup>();
    holds::<crate::shell::Vibrancy>();
    holds::<crate::tooling::EventLogger>();
    holds::<crate::typography::Blockquote>();
}

/// The identifier `text` starts with, past white space.
fn ident(text: &str) -> String {
    let text = text.trim_start();
    let end = text
        .find(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .unwrap_or(text.len());
    text[..end].to_string()
}

/// Types that implement `ParentElement` in every library's sources, a macro's by its invocations.
fn implemented(dir: &Path, found: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("a source folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            implemented(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let text = fs::read_to_string(&path).expect("a source file");
            let mut maker = None;
            for line in text.lines().map(str::trim_start) {
                if let Some(name) = line.strip_prefix("macro_rules! ") {
                    maker = Some(ident(name));
                }
                let Some((_, rest)) = line.split_once("ParentElement for ") else {
                    continue;
                };
                if !line.starts_with("impl") {
                    continue;
                }
                if rest.starts_with('$') {
                    let maker = maker.as_deref().expect("an impl for $name sits in a macro");
                    let call = format!("{maker}!(");
                    for (at, _) in text.match_indices(&call) {
                        found.push(ident(&text[at + call.len()..]));
                    }
                } else {
                    found.push(ident(rest));
                }
            }
        }
    }
}

#[test]
fn every_component_that_takes_children_is_listed() {
    let mut listed: Vec<String> = include_str!("containers.rs")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("holds::<crate::"))
        .map(|path| {
            let path = path.split(['<', '>']).next().expect("a path");
            path.rsplit("::").next().expect("a type").to_string()
        })
        .collect();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates");
    let mut found = Vec::new();
    for entry in fs::read_dir(&crates).expect("the crates folder") {
        let dir = entry.expect("a crate").path();
        if !dir.ends_with("ely-gallery") {
            implemented(&dir.join("src"), &mut found);
        }
    }
    listed.sort();
    found.sort();
    assert_eq!(
        listed, found,
        "src/containers.rs and the ParentElement impls disagree"
    );
}
