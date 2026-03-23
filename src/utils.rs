use crate::hook::{WINDOW_CLASS_NAME, WM_RELOAD_CONFIG};
use crate::load_config;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegSetValueExW,
};
use windows_sys::Win32::System::SystemServices::LANG_CHINESE;
use windows_sys::Win32::UI::Input::Ime::{
    IMC_SETCONVERSIONMODE, IME_CMODE_CHINESE, IME_CMODE_SYMBOL, ImmGetDefaultIMEWnd, ImmIsIME,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CAPITAL,
    VK_CONTROL, VK_LWIN, VK_MENU, VK_SHIFT, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetForegroundWindow,
    GetMessageW, GetWindowThreadProcessId, MSG, PostMessageW, RegisterClassW, SendMessageW,
    WM_CLOSE, WM_IME_CONTROL, WM_INPUTLANGCHANGEREQUEST, WNDCLASSW,
};

use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

const IME_MODE_SYNC_DELAY_MS: u64 = 50;
const CHINESE_IME_CONVERSION_MODE: isize = (IME_CMODE_CHINESE | IME_CMODE_SYMBOL) as isize;
const PRIMARY_LANGUAGE_ID_MASK: u16 = 0x03ff;
static LATEST_IME_SYNC_REQUEST_ID: AtomicU32 = AtomicU32::new(0);

pub(crate) unsafe fn attach_console() {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

pub(crate) fn execute_custom_shortcut(keys: &[String]) {
    let mut inputs = Vec::new();
    let vks: Vec<u16> = keys.iter().filter_map(|k| parse_vk(k)).collect();

    for &vk in &vks {
        inputs.push(key_down(vk));
    }
    for &vk in vks.iter().rev() {
        inputs.push(key_up(vk));
    }

    send_inputs(&inputs);
}

fn parse_vk(key: &str) -> Option<u16> {
    match key.to_uppercase().as_str() {
        "LWIN" | "WIN" => Some(VK_LWIN),
        "SPACE" => Some(VK_SPACE),
        "LCONTROL" | "CTRL" => Some(VK_CONTROL),
        "LSHIFT" | "SHIFT" => Some(VK_SHIFT),
        "LMENU" | "ALT" => Some(VK_MENU),
        "CAPSLOCK" => Some(VK_CAPITAL),
        s if s.len() == 1 => Some(s.as_bytes()[0] as u16),
        _ => None,
    }
}

pub(crate) fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        );
    }
}

pub(crate) fn key_down(vk: u16) -> INPUT {
    unsafe {
        let mut input = std::mem::zeroed::<INPUT>();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: 0,
        };
        input
    }
}

pub(crate) fn key_up(vk: u16) -> INPUT {
    unsafe {
        let mut input = std::mem::zeroed::<INPUT>();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };
        input
    }
}

// Keyboard layout management

pub(crate) unsafe fn set_keyboard_layout(hkl: usize) {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == 0 {
            return;
        }

        PostMessageW(
            hwnd,
            WM_INPUTLANGCHANGEREQUEST,
            0, // Current thread's window
            hkl as isize,
        );
    }
}

pub(crate) unsafe fn get_current_hkl() -> usize {
    unsafe {
        let hwnd = GetForegroundWindow();
        let thread_id = GetWindowThreadProcessId(hwnd, null_mut());
        let hkl = GetKeyboardLayout(thread_id);
        hkl as usize
    }
}

pub(crate) fn rotate_layout(layouts: &[i32], no_en: bool) {
    if layouts.is_empty() {
        return;
    }

    unsafe {
        let current_hkl = get_current_hkl() as i32;

        let current_index = layouts.iter().position(|&l| {
            // Compare low 16 bits (Language ID)
            (l & 0xFFFF) == (current_hkl & 0xFFFF)
        });

        let next_index = match current_index {
            Some(i) => (i + 1) % layouts.len(),
            None => 0,
        };

        set_keyboard_layout(layouts[next_index] as usize);
        if no_en {
            let hwnd = GetForegroundWindow();
            schedule_chinese_ime_mode_sync(hwnd, true);
        }
    }
}

pub(crate) fn schedule_chinese_ime_mode_sync(hwnd: HWND, require_same_foreground: bool) {
    if hwnd == 0 {
        return;
    }

    let request_id = LATEST_IME_SYNC_REQUEST_ID
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(IME_MODE_SYNC_DELAY_MS));

        if LATEST_IME_SYNC_REQUEST_ID.load(Ordering::SeqCst) != request_id {
            return;
        }

        unsafe {
            if require_same_foreground && GetForegroundWindow() != hwnd {
                return;
            }

            let current_hkl = get_keyboard_layout_for_window(hwnd);
            if !is_chinese_ime(current_hkl) {
                return;
            }

            let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
            if ime_hwnd == 0 {
                return;
            }

            SendMessageW(
                ime_hwnd,
                WM_IME_CONTROL,
                IMC_SETCONVERSIONMODE as usize,
                CHINESE_IME_CONVERSION_MODE,
            );
        }
    });
}

unsafe fn get_keyboard_layout_for_window(hwnd: HWND) -> usize {
    unsafe {
        let thread_id = GetWindowThreadProcessId(hwnd, null_mut());
        if thread_id == 0 {
            return 0;
        }
        GetKeyboardLayout(thread_id) as usize
    }
}

fn is_chinese_ime(hkl: usize) -> bool {
    if hkl == 0 {
        return false;
    }

    let language_id = (hkl & 0xFFFF) as u16;
    primary_language_id(language_id) == LANG_CHINESE as u16
        && unsafe { ImmIsIME(hkl as isize) != 0 }
}

fn primary_language_id(language_id: u16) -> u16 {
    language_id & PRIMARY_LANGUAGE_ID_MASK
}

// IPC via hidden window

pub(crate) unsafe fn create_message_window() {
    let h_instance = unsafe { GetModuleHandleW(null_mut()) };
    let wnd_class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: 0,
        hCursor: 0,
        hbrBackground: 0,
        lpszMenuName: null_mut(),
        lpszClassName: WINDOW_CLASS_NAME.as_ptr(),
    };

    unsafe {
        RegisterClassW(&wnd_class);
        CreateWindowExW(
            0,
            WINDOW_CLASS_NAME.as_ptr(),
            null_mut(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            h_instance,
            null_mut(),
        );
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
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            println!("Shutting down...");
            std::process::exit(0);
        }
        WM_RELOAD_CONFIG => {
            load_config();
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn send_msg_to_instance(msg: u32) -> bool {
    unsafe {
        let hwnd = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), null_mut());
        if hwnd != 0 {
            PostMessageW(hwnd, msg, 0, 0);
            true
        } else {
            false
        }
    }
}

pub fn get_instance_pid() -> Option<u32> {
    unsafe {
        let hwnd = FindWindowW(WINDOW_CLASS_NAME.as_ptr(), null_mut());
        if hwnd != 0 {
            let mut pid = 0;
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut pid);
            Some(pid)
        } else {
            None
        }
    }
}

pub fn encode_wide(s: &str) -> Vec<u16> {
    let mut res: Vec<u16> = s.encode_utf16().collect();
    res.push(0);
    res
}

pub fn set_startup(enable: bool) -> Result<(), String> {
    let run_key = encode_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let app_name = encode_wide("Capsense");

    unsafe {
        let mut hkey = 0 as _;
        let res = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            run_key.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if res != 0 {
            return Err(format!("Failed to open registry key: {}", res));
        }

        if enable {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let path_str = exe_path.to_str().ok_or("Invalid executable path")?;
            let wide_path = encode_wide(path_str);

            let res = RegSetValueExW(
                hkey,
                app_name.as_ptr(),
                0,
                REG_SZ,
                wide_path.as_ptr() as *const u8,
                (wide_path.len() * 2) as u32,
            );

            RegCloseKey(hkey);
            if res != 0 {
                return Err(format!("Failed to set registry value: {}", res));
            }
        } else {
            let res = RegDeleteValueW(hkey, app_name.as_ptr());
            RegCloseKey(hkey);
            // 2 = ERROR_FILE_NOT_FOUND, which is fine if we're disabling and it's not there
            if res != 0 && res != 2 {
                return Err(format!("Failed to delete registry value: {}", res));
            }
        }
    }

    Ok(())
}
