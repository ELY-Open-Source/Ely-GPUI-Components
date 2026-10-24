//! Ely's generative AI components for GPUI: prompts, generation grids and queues, masks, models, fine-tuning, voices and video shots.
#![deny(missing_docs)]

use ely_charts as charts;
use ely_chat as chat;
use ely_core::{buttons, data_display, forms, i18n, layout, motion, primitives, theme, typography};
use ely_debug as debug;
use ely_documents as documents;

mod audio;
mod brush;
mod compare;
mod downloads;
mod embed;
mod grid;
mod hardware;
#[doc(hidden)]
pub mod lens;
mod mask;
mod models;
mod playground;
mod prompt;
mod queue;
mod ratio;
mod seed;
mod style;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod tuning;
mod variation;
mod versions;
mod video;
mod voice;

pub use audio::AudioGenerationPlayer;
pub use brush::MaskBrush;
pub use compare::{ABCompareView, Verdict};
pub use downloads::{Download, DownloadState, ModelDownloadManager};
pub use embed::{Embedded, EmbeddingVisualizer};
pub use grid::{GenerationGrid, Outcome};
pub use hardware::{HardwareMonitor, Reading};
pub use mask::{InpaintCanvas, MaskStroke};
pub use models::{ModelCard, ModelState, ModelStatus};
pub use playground::PromptPlayground;
pub use prompt::PromptEnhancer;
pub use queue::{GenerationQueue, Job, JobState};
pub use ratio::AspectRatioPicker;
pub use seed::SeedInput;
pub use style::{StylePreset, StylePresetPicker};
pub use tuning::{FineTuneJobCard, TunePhase};
pub use variation::VariationPicker;
pub use versions::{PromptDiff, PromptVersion, PromptVersionHistory};
pub use video::{Shot, VideoGenerationTimeline};
pub use voice::{TTSVoicePicker, Voice};
