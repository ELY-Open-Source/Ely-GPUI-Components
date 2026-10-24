use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*,
};

use crate::{
    buttons::{ButtonVariant, IconButton},
    motion::ProgressBar,
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, TextSize},
    typography::{Ellipsis, format::file_size, tabular},
};

/// Where one upload stands.
#[derive(Clone, Debug, PartialEq)]
pub enum UploadState {
    /// Sent this share, 0 to 1.
    Uploading(f32),
    /// Sent whole.
    Done,
    /// Why it stopped.
    Failed(SharedString),
}

/// One file on its way up.
#[derive(Clone, Debug, PartialEq)]
pub struct Upload {
    /// The file's name.
    pub name: SharedString,
    /// Its size in bytes.
    pub bytes: u64,
    /// Where it stands.
    pub state: UploadState,
}

type OnRow = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Files on their way up. Each row shows its name, the percent and a progress bar while it goes, then its size and a check, or why it failed with Retry.
///
/// ```
/// use ely_core::forms::{Upload, UploadList, UploadState};
///
/// let uploads = UploadList::new(
///     "uploads",
///     [Upload {
///         name: "report.pdf".into(),
///         bytes: 2_400_000,
///         state: UploadState::Uploading(0.4),
///     }],
/// );
/// ```
#[derive(IntoElement)]
pub struct UploadList {
    id: ElementId,
    uploads: Vec<Upload>,
    on_cancel: Option<OnRow>,
    on_retry: Option<OnRow>,
    on_remove: Option<OnRow>,
}

impl UploadList {
    /// Rows under `id`, one per upload. Panics on a share outside 0 to 1.
    pub fn new(id: impl Into<ElementId>, uploads: impl IntoIterator<Item = Upload>) -> Self {
        let uploads: Vec<Upload> = uploads.into_iter().collect();
        for upload in &uploads {
            if let UploadState::Uploading(share) = upload.state {
                assert!(
                    (0.0..=1.0).contains(&share),
                    "{} is {share} sent, not 0..=1",
                    upload.name
                );
            }
        }
        Self {
            id: id.into(),
            uploads,
            on_cancel: None,
            on_retry: None,
            on_remove: None,
        }
    }

    /// Runs with the row of an upload still under way.
    pub fn on_cancel(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }

    /// Runs with the row of a failed upload.
    pub fn on_retry(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_retry = Some(Rc::new(handler));
        self
    }

    /// Runs with the row of a finished upload.
    pub fn on_remove(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for UploadList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let (cancel, retry, remove) = (
            crate::i18n::text(cx, "upload.cancel", &[]),
            crate::i18n::text(cx, "upload.retry", &[]),
            crate::i18n::text(cx, "upload.remove", &[]),
        );
        let button = |name: String,
                      icon: IconName,
                      tip: SharedString,
                      handler: &Option<OnRow>,
                      ix: usize| {
            handler.clone().map(|run| {
                IconButton::new((self.id.clone(), SharedString::from(name)), icon)
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .tooltip(tip)
                    .on_click(move |_, window, cx| run(ix, window, cx))
            })
        };
        let rows = self.uploads.iter().enumerate().map(|(ix, upload)| {
            let (note, tone) = match &upload.state {
                UploadState::Uploading(share) => {
                    (format!("{}%", (share * 100.0).round()), colors.fg_subtle)
                }
                UploadState::Done => (file_size(upload.bytes, false), colors.fg_subtle),
                UploadState::Failed(why) => (why.to_string(), colors.danger),
            };
            let action = match &upload.state {
                UploadState::Uploading(_) => button(
                    format!("cancel-{ix}"),
                    IconName::X,
                    cancel.clone(),
                    &self.on_cancel,
                    ix,
                ),
                UploadState::Failed(_) => button(
                    format!("retry-{ix}"),
                    IconName::RotateCw,
                    retry.clone(),
                    &self.on_retry,
                    ix,
                ),
                UploadState::Done => button(
                    format!("remove-{ix}"),
                    IconName::X,
                    remove.clone(),
                    &self.on_remove,
                    ix,
                ),
            };
            div()
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .child(match upload.state {
                    UploadState::Done => Icon::new(IconName::CircleCheck)
                        .size(IconSize::Md)
                        .color(colors.success),
                    UploadState::Failed(_) => Icon::new(IconName::CircleAlert)
                        .size(IconSize::Md)
                        .color(colors.danger),
                    UploadState::Uploading(_) => Icon::new(IconName::FileText)
                        .size(IconSize::Md)
                        .color(colors.fg_muted),
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(Ellipsis::new(upload.name.clone())),
                                )
                                .child(
                                    tabular(div())
                                        .flex_none()
                                        .text_size(theme.text_size(TextSize::Xs))
                                        .text_color(tone)
                                        .child(note),
                                ),
                        )
                        .when_some(
                            match upload.state {
                                UploadState::Uploading(share) => Some(share),
                                _ => None,
                            },
                            |text, share| {
                                text.child(ProgressBar::new(
                                    (self.id.clone(), SharedString::from(format!("bar-{ix}"))),
                                    share,
                                ))
                            },
                        ),
                )
                .children(action)
        });
        div()
            .flex()
            .flex_col()
            .text_size(theme.text_size(TextSize::Base))
            .children(rows)
    }
}
