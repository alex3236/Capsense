use crate::hook::{WINDOW_CLASS_NAME, WM_RELOAD_CONFIG};
use crate::load_config;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE,
    REG_SZ,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CAPITAL, VK_CONTROL, VK_LWIN,
    VK_MENU, VK_SHIFT, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW, PostMessageW, RegisterClassW,
    MSG, WM_CLOSE, WNDCLASSW,
};

use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

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
