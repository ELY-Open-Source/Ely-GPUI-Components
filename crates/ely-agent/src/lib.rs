//! Ely's agent components for GPUI: tool calls, approvals, plans and progress, proposed changes, sandboxes, servers, memory and previews.
#![deny(missing_docs)]

use ely_chat as chat;
use ely_collab as collab;
use ely_core::{
    buttons, data_display, forms, i18n, layout, lists, motion, overlays, primitives, theme,
    typography,
};
use ely_documents as documents;
use ely_git as git;

mod artifact;
mod changes;
mod control;
mod environment;
mod line;
mod memory;
mod permission;
mod previews;
mod progress;
mod registry;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod tools;

pub use artifact::ArtifactPanel;
pub use changes::{ChangeState, FileChange, FileChangeCard, MultiFileDiffReview};
pub use control::{CostBreakdown, HumanInputRequest};
pub use environment::{McpServer, McpServerList, SandboxState, SandboxStatus, ServerState};
pub use memory::{Checkpoint, CheckpointList, Memory, MemoryPanel};
pub use permission::{Permission, PermissionPrompt, ToolApprovalDialog};
pub use previews::{BrowserPreview, ComputerUseViewer, LivePreview};
pub use progress::{AgentPlan, AgentProgress, AgentState, AgentStatus, AgentStep, AgentStepList};
pub use registry::{RegisteredTool, ToolAccess, ToolRegistry};
pub use tools::{ToolCallCard, ToolCallGroup};
