use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::CONFIG;
use crate::utils::*;

use windows_sys::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, EVENT_OBJECT_FOCUS, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED,
    MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WINEVENT_OUTOFCONTEXT, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONUP, WM_MBUTTONUP, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_USER, WM_XBUTTONUP,
};

// Custom messages
pub const WM_RELOAD_CONFIG: u32 = WM_USER + 1;

lazy_static::lazy_static! {
    pub static ref WINDOW_CLASS_NAME: Vec<u16> = encode_wide("CapsCustomHookClass");
}

// Global states
static mut HOOK_HANDLE: HHOOK = 0;
static mut FOCUS_HOOK_HANDLE: HWINEVENTHOOK = 0;
static mut MOUSE_HOOK_HANDLE: HHOOK = 0;
static CAPS_IS_DOWN: AtomicBool = AtomicBool::new(false);
static LONG_ACTION_FIRED: AtomicBool = AtomicBool::new(false);
static IGNORE_INJECTED_CAPS_EVENTS: AtomicU32 = AtomicU32::new(0);
static PRESS_START: Mutex<Option<Instant>> = Mutex::new(None);

static ACTIVE_PRESS_ID: AtomicU32 = AtomicU32::new(0);
static NEXT_PRESS_ID: AtomicU32 = AtomicU32::new(1);

pub fn run_hook_loop() -> Result<(), Box<dyn std::error::Error>> {
    // Create hidden window to receive messages
    thread::spawn(|| unsafe {
        create_message_window();
    });

    // Set hook
    unsafe {
        let h_instance: HINSTANCE = GetModuleHandleW(std::ptr::null());
        HOOK_HANDLE =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), h_instance, 0);
        if HOOK_HANDLE == 0 {
            return Err("SetWindowsHookExW failed".into());
        }

        MOUSE_HOOK_HANDLE =
            SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), h_instance, 0);
        if MOUSE_HOOK_HANDLE == 0 {
            UnhookWindowsHookEx(HOOK_HANDLE);
            return Err("SetWindowsHookExW for mouse failed".into());
        }

        FOCUS_HOOK_HANDLE = SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_FOCUS,
            0,
            Some(focus_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if FOCUS_HOOK_HANDLE == 0 {
            UnhookWindowsHookEx(MOUSE_HOOK_HANDLE);
            UnhookWindowsHookEx(HOOK_HANDLE);
            return Err("SetWinEventHook failed".into());
        }
    }

    let mut msg = MSG {
        hwnd: 0,
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 },
    };

    unsafe {
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWinEvent(FOCUS_HOOK_HANDLE);
        UnhookWindowsHookEx(MOUSE_HOOK_HANDLE);
        UnhookWindowsHookEx(HOOK_HANDLE);
    }

    Ok(())
}

unsafe extern "system" fn focus_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: isize,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _event_time: u32,
) {
    if hwnd == 0 {
        return;
    }

    let config_guard = CONFIG.read().unwrap();
    let Some(config) = config_guard.as_ref() else {
        return;
    };

    if !config.no_en {
        return;
    }

    schedule_chinese_ime_mode_sync(hwnd, false);
}

unsafe extern "system" fn low_level_mouse_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(MOUSE_HOOK_HANDLE, code, wparam, lparam) };
    }

    let msg = wparam as u32;
    if matches!(
        msg,
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP
    ) && no_en_enabled()
    {
        let hwnd = unsafe { GetForegroundWindow() };
        schedule_chinese_ime_mode_sync(hwnd, true);
    }

    unsafe { CallNextHookEx(MOUSE_HOOK_HANDLE, code, wparam, lparam) }
}

fn no_en_enabled() -> bool {
    let config_guard = CONFIG.read().unwrap();
    matches!(config_guard.as_ref(), Some(config) if config.no_en)
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(HOOK_HANDLE, code, wparam, lparam) };
    }

    let kb = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
    let msg = wparam as u32;
    let is_caps = kb.vkCode == VK_CAPITAL as u32;
    let is_injected = (kb.flags & LLKHF_INJECTED) != 0;

    if !is_caps {
        return unsafe { CallNextHookEx(HOOK_HANDLE, code, wparam, lparam) };
    }

    if is_injected {
        let remain = IGNORE_INJECTED_CAPS_EVENTS.load(Ordering::SeqCst);
        if remain > 0 {
            IGNORE_INJECTED_CAPS_EVENTS.fetch_sub(1, Ordering::SeqCst);
        }
        return unsafe { CallNextHookEx(HOOK_HANDLE, code, wparam, lparam) };
    }

    let config_guard = CONFIG.read().unwrap();
    let config = config_guard.as_ref().unwrap();
    let threshold = Duration::from_millis(config.tap_threshold_ms);

    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        if CAPS_IS_DOWN.swap(true, Ordering::SeqCst) {
            return 1;
        }

        let press_id = NEXT_PRESS_ID.fetch_add(1, Ordering::SeqCst);
        ACTIVE_PRESS_ID.store(press_id, Ordering::SeqCst);

        {
            let mut start = PRESS_START.lock().unwrap();
            *start = Some(Instant::now());
        }
        LONG_ACTION_FIRED.store(false, Ordering::SeqCst);

        thread::spawn(move || {
            thread::sleep(threshold);
            if ACTIVE_PRESS_ID.load(Ordering::SeqCst) != press_id {
                return;
            }
            if CAPS_IS_DOWN.load(Ordering::SeqCst)
                && !LONG_ACTION_FIRED.swap(true, Ordering::SeqCst)
            {
                IGNORE_INJECTED_CAPS_EVENTS.store(2, Ordering::SeqCst);
                send_inputs(&[key_down(VK_CAPITAL), key_up(VK_CAPITAL)]);
            }
        });

        return 1;
    }

    if msg == WM_KEYUP || msg == WM_SYSKEYUP {
        let was_down = CAPS_IS_DOWN.swap(false, Ordering::SeqCst);
        if !was_down {
            return 1;
        }

        ACTIVE_PRESS_ID.store(0, Ordering::SeqCst);

        let long_fired = LONG_ACTION_FIRED.load(Ordering::SeqCst);
        let mut start = PRESS_START.lock().unwrap();
        let elapsed = start.take().map(|t| t.elapsed()).unwrap_or_default();

        if !long_fired && elapsed < threshold {
            match config.tap_action.as_str() {
                "switch_layout" => rotate_layout(&config.layouts, config.no_en),
                _ => execute_custom_shortcut(&config.tap_shortcut),
            }
        }
        return 1;
    }

    unsafe { CallNextHookEx(HOOK_HANDLE, code, wparam, lparam) }
}
