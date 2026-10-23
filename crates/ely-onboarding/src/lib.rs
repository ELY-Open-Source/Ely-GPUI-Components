//! Onboarding, chapter 37: first-run wizards, checklists, hotspots, highlights, help, feedback and support.
#![deny(missing_docs)]

use ely_core::{
    buttons, data_display, forms, layout, lists, motion, navigation, overlays, primitives, theme,
    typography,
};
use ely_documents as documents;
use ely_settings as settings;

mod checklist;
mod dialogs;
mod feedback;
mod help;
mod highlight;
mod hotspot;
mod support;
mod wizard;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use checklist::{SetupChecklist, SetupTask};
pub use dialogs::{KeyboardShortcutCheatsheet, WhatsNewDialog};
pub use feedback::{FeedbackWidget, Sentiment};
pub use help::{HelpArticle, HelpPanel, found};
pub use highlight::FeatureHighlight;
pub use hotspot::Hotspot;
pub use support::{ContactSupport, InlineHelp};
pub use wizard::{OnboardingStep, OnboardingWizard};
