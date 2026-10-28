//! When an auto bar shows, and the frames it asks.

use std::time::Duration;

use gpui::{App, Entity, Pixels, Point, ScrollHandle, Task, Window};
use web_time::Instant;

use crate::motion;

const IDLE: Duration = Duration::from_millis(900);

pub(super) struct Activity {
    pub(super) handle: ScrollHandle,
    pub(super) last: Point<Pixels>,
    pub(super) active_at: Option<Instant>,
    pub(super) hovered: bool,
    /// The pointer on the area's own bars, by `slot`.
    pub(super) on_bar: [bool; 2],
    /// The pending wake and its activity; replacing it cancels it.
    wake: Option<(Instant, Task<()>)>,
}

impl Activity {
    pub(super) fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            last: Point::default(),
            active_at: None,
            hovered: false,
            on_bar: [false; 2],
            wake: None,
        }
    }

    pub(super) fn held(&self) -> bool {
        self.hovered || self.on_bar.contains(&true)
    }
}

/// Visibility 0..=1: full while active, fading after `IDLE`.
fn presence(activity: &Activity, fade: Duration) -> f32 {
    if activity.held() {
        return 1.0;
    }
    let Some(at) = activity.active_at else {
        return 0.0;
    };
    let quiet = at.elapsed().saturating_sub(IDLE);
    1.0 - (quiet.as_secs_f32() / fade.as_secs_f32()).min(1.0)
}

/// Frames only while fading; one wake ends a still hold.
pub(super) fn track_activity(
    state: &Entity<Activity>,
    offset: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    if state.read(cx).last != offset {
        state.update(cx, |activity, _| {
            activity.last = offset;
            activity.active_at = Some(Instant::now());
        });
    }
    let fade = motion::duration(motion::SLOW, cx);
    let activity = state.read(cx);
    let presence = presence(activity, fade);
    if presence <= 0.0 || activity.held() {
        return presence;
    }
    let left = activity
        .active_at
        .map_or(Duration::ZERO, |at| IDLE.saturating_sub(at.elapsed()));
    if left.is_zero() {
        window.request_animation_frame();
    } else if let Some(at) = activity
        .active_at
        .filter(|at| activity.wake.as_ref().map(|wake| wake.0) != Some(*at))
    {
        let view = window.current_view();
        let wake = window.spawn(cx, async move |cx| {
            cx.background_executor().timer(left).await;
            if cx.update(|_, cx| cx.notify(view)).is_err() {
                log::debug!("scrollbar: its window closed before the bar hid");
            }
        });
        state.update(cx, |activity, _| activity.wake = Some((at, wake)));
    }
    presence
}
