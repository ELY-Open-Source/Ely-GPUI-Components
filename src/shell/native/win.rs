//! Windows: a notification-area icon on a message-only window.

use std::sync::Once;

use gpui::App;
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

use super::{Look, Route, TrayItem};

const CLASS: PCWSTR = w!("ElyTrayWindow");
/// The icon's messages to its window.
pub(super) const CALLBACK: u32 = WM_APP + 1;

/// What the window procedure reads through `GWLP_USERDATA`.
struct State {
    route: Route,
    items: Vec<TrayItem>,
    data: NOTIFYICONDATAW,
    /// Explorer restarted: icons must be added again.
    restarted: u32,
}

pub struct Tray {
    window: HWND,
    icon: HICON,
    state: Box<State>,
}

impl Tray {
    pub fn new(
        look: &Look,
        items: Vec<TrayItem>,
        route: Route,
        _: &mut App,
    ) -> anyhow::Result<Self> {
        static REGISTER: Once = Once::new();
        unsafe {
            let instance = GetModuleHandleW(None)?.into();
            REGISTER.call_once(|| {
                let class = WNDCLASSW {
                    lpfnWndProc: Some(procedure),
                    hInstance: instance,
                    lpszClassName: CLASS,
                    ..Default::default()
                };
                assert_ne!(RegisterClassW(&class), 0, "the tray window class registers");
            });
            let (width, height) = (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON));
            // Windows reads PNG images as icon resources.
            let icon = CreateIconFromResourceEx(
                look.png,
                true,
                0x0003_0000,
                width,
                height,
                LR_DEFAULTCOLOR,
            )
            .map_err(|error| anyhow::anyhow!("the tray PNG is no icon: {error}"))?;
            let window = CreateWindowExW(
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
            );
            let window = match window {
                Ok(window) => window,
                Err(error) => {
                    DestroyIcon(icon).ok();
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
            let tip: Vec<u16> = look
                .name
                .encode_utf16()
                .take(data.szTip.len() - 1)
                .collect();
            data.szTip[..tip.len()].copy_from_slice(&tip);
            let state = Box::new(State {
                route,
                items,
                data,
                restarted: RegisterWindowMessageW(w!("TaskbarCreated")),
            });
            SetWindowLongPtrW(window, GWLP_USERDATA, &*state as *const State as isize);
            let tray = Self {
                window,
                icon,
                state,
            };
            anyhow::ensure!(
                Shell_NotifyIconW(NIM_ADD, &tray.state.data).as_bool(),
                "the notification area refused the icon"
            );
            log::info!("tray: shown with {} items", tray.state.items.len());
            Ok(tray)
        }
    }

    pub fn pick(&self, index: usize) {
        assert!(index < self.state.items.len(), "tray has no item {index}");
        self.state.route.pick(index);
    }

    #[cfg(test)]
    pub fn window(&self) -> HWND {
        self.window
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            if !Shell_NotifyIconW(NIM_DELETE, &self.state.data).as_bool() {
                log::warn!("tray: the notification area kept the icon");
            }
            SetWindowLongPtrW(self.window, GWLP_USERDATA, 0);
            DestroyWindow(self.window).ok();
            DestroyIcon(self.icon).ok();
        }
        log::info!("tray: removed");
    }
}

/// Left click picks the first item; right shows the menu.
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state = GetWindowLongPtrW(window, GWLP_USERDATA) as *const State;
        if state.is_null() {
            return DefWindowProcW(window, message, wparam, lparam);
        }
        let state = &*state;
        if message == state.restarted {
            match Shell_NotifyIconW(NIM_ADD, &state.data).as_bool() {
                true => log::info!("tray: added again after Explorer restarted"),
                false => log::error!("tray: Explorer restarted and refused the icon"),
            }
        } else if message == CALLBACK {
            match lparam.0 as u32 {
                WM_LBUTTONUP if !state.items.is_empty() => state.route.pick(0),
                WM_RBUTTONUP => {
                    if let Some(index) = menu(window, &state.items) {
                        state.route.pick(index);
                    }
                }
                _ => {}
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}

/// Shows the items at the pointer; the one chosen.
unsafe fn menu(window: HWND, items: &[TrayItem]) -> Option<usize> {
    unsafe {
        let menu = CreatePopupMenu().ok()?;
        for (index, item) in items.iter().enumerate() {
            let added = match item {
                TrayItem::Separator => AppendMenuW(menu, MF_SEPARATOR, 0, None),
                TrayItem::Action(label) => {
                    let label: Vec<u16> = label.encode_utf16().chain([0]).collect();
                    AppendMenuW(menu, MF_STRING, index + 1, PCWSTR(label.as_ptr()))
                }
            };
            if let Err(error) = added {
                log::error!("tray: the menu took no item {index}: {error}");
                DestroyMenu(menu).ok();
                return None;
            }
        }
        let mut at = POINT::default();
        GetCursorPos(&mut at).ok()?;
        // Else the menu stays when the pointer leaves it.
        let _ = SetForegroundWindow(window);
        let chosen = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            at.x,
            at.y,
            None,
            window,
            None,
        );
        PostMessageW(Some(window), WM_NULL, WPARAM(0), LPARAM(0)).ok();
        DestroyMenu(menu).ok();
        (chosen.0 > 0).then(|| chosen.0 as usize - 1)
    }
}
