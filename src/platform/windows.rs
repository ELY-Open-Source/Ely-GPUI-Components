//! Win32: a notification-area icon on a message-only window.

use std::sync::Once;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreateIconFromResourceEx, CreatePopupMenu, CreateWindowExW,
                DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, GWLP_USERDATA,
                GetCursorPos, GetSystemMetrics, GetWindowLongPtrW, HICON, HWND_MESSAGE,
                LR_DEFAULTCOLOR, MF_SEPARATOR, MF_STRING, PostMessageW, RegisterClassW,
                RegisterWindowMessageW, SM_CXSMICON, SM_CYSMICON, SetForegroundWindow,
                SetWindowLongPtrW, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
                WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
                WNDCLASSW,
            },
        },
    },
    core::{PCWSTR, w},
};

const CLASS: PCWSTR = w!("ElyTrayWindow");
/// The icon's messages to its window.
const CALLBACK: u32 = WM_APP + 1;

type OnPick = Box<dyn Fn(usize)>;

/// What the window procedure reads through `GWLP_USERDATA`.
struct State {
    on_pick: OnPick,
    /// Labels; `None` is a separator.
    entries: Vec<Option<String>>,
    data: NOTIFYICONDATAW,
    /// Explorer restarted: icons must be added again.
    restarted: u32,
}

/// The icon and its window; dropping it removes both.
pub(crate) struct Tray {
    window: HWND,
    icon: HICON,
    state: Box<State>,
}

impl Tray {
    /// `entries` are labels, `None` a separator; `on_pick` gets a picked entry's index.
    pub(crate) fn new(
        png: &[u8],
        name: &str,
        entries: &[Option<&str>],
        on_pick: OnPick,
    ) -> anyhow::Result<Self> {
        static REGISTER: Once = Once::new();
        // SAFETY: a null name asks for this process's own module.
        let instance = unsafe { GetModuleHandleW(None) }?.into();
        REGISTER.call_once(|| {
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: CLASS,
                ..Default::default()
            };
            // SAFETY: the class names a static string and a procedure that lives forever.
            let atom = unsafe { RegisterClassW(&class) };
            assert_ne!(atom, 0, "the tray window class registers");
        });
        // SAFETY: plain metric queries.
        let size = unsafe { (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON)) };
        // SAFETY: Windows reads PNG bytes as an icon resource and copies them.
        let icon = unsafe {
            CreateIconFromResourceEx(png, true, 0x0003_0000, size.0, size.1, LR_DEFAULTCOLOR)
        }
        .map_err(|error| anyhow::anyhow!("the tray PNG is no icon: {error}"))?;
        // SAFETY: the class is registered above; a message-only window takes no parent but HWND_MESSAGE.
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                CLASS,
                None,
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(instance),
                None,
            )
        };
        let window = match window {
            Ok(window) => window,
            Err(error) => {
                // SAFETY: the icon was made above and nothing else holds it.
                unsafe { DestroyIcon(icon) }.ok();
                anyhow::bail!("no tray window: {error}");
            }
        };
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: window,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: CALLBACK,
            hIcon: icon,
            ..Default::default()
        };
        let tip: Vec<u16> = name.encode_utf16().take(data.szTip.len() - 1).collect();
        data.szTip[..tip.len()].copy_from_slice(&tip);
        // SAFETY: the name is a static wide string.
        let restarted = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        let state = Box::new(State {
            on_pick,
            entries: entries
                .iter()
                .map(|entry| entry.map(str::to_string))
                .collect(),
            data,
            restarted,
        });
        // SAFETY: the box outlives the window: Drop clears the pointer before it frees either.
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, &*state as *const State as isize) };
        let tray = Self {
            window,
            icon,
            state,
        };
        // SAFETY: the data names this window and icon, both alive.
        let added = unsafe { Shell_NotifyIconW(NIM_ADD, &tray.state.data) };
        anyhow::ensure!(added.as_bool(), "the notification area refused the icon");
        log::info!("tray: shown with {} items", tray.state.entries.len());
        Ok(tray)
    }

    /// Chooses entry `index` as a click would.
    pub(crate) fn pick(&self, index: usize) {
        assert!(index < self.state.entries.len(), "tray has no item {index}");
        (self.state.on_pick)(index);
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        // SAFETY: the data names the icon added in `new`.
        if !unsafe { Shell_NotifyIconW(NIM_DELETE, &self.state.data) }.as_bool() {
            log::warn!("tray: the notification area kept the icon");
        }
        // SAFETY: the window and icon are this tray's; the procedure reads no state once the pointer is cleared.
        unsafe {
            SetWindowLongPtrW(self.window, GWLP_USERDATA, 0);
            DestroyWindow(self.window).ok();
            DestroyIcon(self.icon).ok();
        }
        log::info!("tray: removed");
    }
}

/// Left click picks the first entry; right shows the menu.
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: the pointer is a live `State` or null, as `new` and Drop set it.
    let state = unsafe { (GetWindowLongPtrW(window, GWLP_USERDATA) as *const State).as_ref() };
    if let Some(state) = state {
        if message == state.restarted {
            // SAFETY: the data names this window and its icon.
            match unsafe { Shell_NotifyIconW(NIM_ADD, &state.data) }.as_bool() {
                true => log::info!("tray: added again after Explorer restarted"),
                false => log::error!("tray: Explorer restarted and refused the icon"),
            }
        } else if message == CALLBACK {
            match lparam.0 as u32 {
                WM_LBUTTONUP if !state.entries.is_empty() => (state.on_pick)(0),
                WM_RBUTTONUP => {
                    if let Some(index) = menu(window, &state.entries) {
                        (state.on_pick)(index);
                    }
                }
                _ => {}
            }
        }
    }
    // SAFETY: the arguments are the ones Windows passed in.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// Shows the entries at the pointer; the one chosen.
fn menu(window: HWND, entries: &[Option<String>]) -> Option<usize> {
    // SAFETY: a fresh popup menu, destroyed below on every path.
    let menu = unsafe { CreatePopupMenu() }.ok()?;
    for (index, entry) in entries.iter().enumerate() {
        let label: Option<Vec<u16>> = entry
            .as_ref()
            .map(|label| label.encode_utf16().chain([0]).collect());
        // SAFETY: the menu is alive and each label is NUL-terminated and outlives the call.
        let added = unsafe {
            match &label {
                None => AppendMenuW(menu, MF_SEPARATOR, 0, None),
                Some(label) => AppendMenuW(menu, MF_STRING, index + 1, PCWSTR(label.as_ptr())),
            }
        };
        if let Err(error) = added {
            log::error!("tray: the menu took no item {index}: {error}");
            // SAFETY: the menu is this function's alone.
            unsafe { DestroyMenu(menu) }.ok();
            return None;
        }
    }
    let mut at = POINT::default();
    // SAFETY: `at` is a valid point to write.
    let pointed = unsafe { GetCursorPos(&mut at) };
    // SAFETY: the window and menu are alive; the menu goes at the end.
    let chosen = unsafe {
        let chosen = pointed.ok().map(|_| {
            // Else the menu stays when the pointer leaves it.
            let _ = SetForegroundWindow(window);
            TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                at.x,
                at.y,
                None,
                window,
                None,
            )
        });
        PostMessageW(Some(window), WM_NULL, WPARAM(0), LPARAM(0)).ok();
        DestroyMenu(menu).ok();
        chosen
    }?;
    (chosen.0 > 0).then(|| chosen.0 as usize - 1)
}

/// A left click on `tray`'s icon, as Windows reports it.
#[cfg(test)]
pub(crate) fn click(tray: &Tray) {
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
    // SAFETY: the window is alive while `tray` is.
    unsafe {
        SendMessageW(
            tray.window,
            CALLBACK,
            Some(WPARAM(0)),
            Some(LPARAM(WM_LBUTTONUP as isize)),
        )
    };
}

/// Whether `tray`'s window still exists, read after the tray goes.
#[cfg(test)]
pub(crate) fn window_lives(window: isize) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;
    // SAFETY: IsWindow takes any handle value.
    unsafe { IsWindow(Some(HWND(window as *mut _))) }.as_bool()
}

#[cfg(test)]
impl Tray {
    pub(crate) fn window(&self) -> isize {
        self.window.0 as isize
    }
}
