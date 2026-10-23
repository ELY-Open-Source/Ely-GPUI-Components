/// One scripted input, aimed at a probe by key.
pub enum Step {
    Rest,
    Hover(&'static str),
    /// Moves the pointer to an offset in the probe.
    HoverAt(&'static str, f32, f32),
    Down(&'static str),
    Up(&'static str),
    Click(&'static str),
    /// Clicks just inside the probe's right edge.
    ClickEnd(&'static str),
    /// Presses at an offset from the probe's top-left corner.
    DownAt(&'static str, f32, f32),
    /// Right-clicks at an offset from the probe's top-left corner.
    RightAt(&'static str, f32, f32),
    /// Drags from the last point to an offset, in steps.
    DragTo(&'static str, f32, f32),
    /// Swipes two fingers sideways across the probe's middle.
    Swipe(&'static str, f32),
    UpAt(&'static str, f32, f32),
    Key(&'static str),
    /// Types text into the focus, one key at a time.
    Type(&'static str),
    Wait(u64),
    Shot(&'static str),
    /// Photographs a window a demo opened, by its key.
    ShotWindow(&'static str, &'static str),
    CloseWindow(&'static str),
    /// Sends a keystroke to a window a demo opened.
    KeyWindow(&'static str, &'static str),
    /// Asks AppKit to close a window, as its button would.
    CloseNative(&'static str),
    /// Fails unless the window is gone.
    ExpectClosed(&'static str),
    /// Photographs this app's frontmost popup, such as the share picker.
    ShotPopup(&'static str),
    /// Posts a key down to AppKit: `escape` or `5`.
    NativeKey(&'static str),
    /// Cancels the open file panel, which runs out of process.
    CancelPanel,
}
