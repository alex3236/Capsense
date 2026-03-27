use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MessageBoxW, PostMessageW,
    RegisterClassW, ShowWindow, TranslateMessage, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, MB_ICONINFORMATION,
    MB_OK, MSG, WM_CLOSE, WM_COMMAND, WM_DESTROY, WNDCLASSW, WS_CHILD,
    WS_MAXIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_THICKFRAME, WS_VISIBLE,
};

use crate::hook::WM_RELOAD_CONFIG;
use crate::utils::{encode_wide, send_msg_to_instance};

const ID_STOP_BUTTON: usize = 1;
const ID_RELOAD_BUTTON: usize = 2;
const ID_EDIT_BUTTON: usize = 3;

pub fn show_instance_manager_window(pid: u32) {
    let window_title = encode_wide("Capsense");
    let class_name = encode_wide("CapsenseManagerWindowClass");

    println!(
        "Attempting to show instance manager window for PID: {}",
        pid
    );

    unsafe {
        let h_instance = GetModuleHandleW(null_mut());

        let cursor = windows_sys::Win32::UI::WindowsAndMessaging::LoadCursorW(
            0,
            windows_sys::Win32::UI::WindowsAndMessaging::IDC_ARROW,
        );

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: 0,
            hCursor: cursor,
            hbrBackground: (windows_sys::Win32::Graphics::Gdi::COLOR_WINDOW + 1) as isize,
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };

        if RegisterClassW(&wnd_class) == 0 {
            let err = windows_sys::Win32::Foundation::GetLastError();
            if err != 1410 {
                // 1410 = ERROR_CLASS_ALREADY_EXISTS
                println!("Failed to register class. Error: {}", err);
                return;
            }
        }

        let hwnd = CreateWindowExW(
            windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_APPWINDOW,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            320,
            240,
            0,
            0,
            h_instance,
            null_mut(),
        );

        if hwnd == 0 {
            let err = windows_sys::Win32::Foundation::GetLastError();
            println!("Failed to create window. Error: {}", err);
            return;
        }

        println!("Window created successfully: {:?}", hwnd);

        // Bring to front
        windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);

        // Display PID label
        let pid_text = encode_wide(&format!("Capsense is running with PID: {}", pid));
        let static_class = encode_wide("STATIC");
        CreateWindowExW(
            0,
            static_class.as_ptr(),
            pid_text.as_ptr(),
            WS_CHILD | WS_VISIBLE,
            20,
            20,
            280,
            25,
            hwnd,
            0,
            h_instance,
            null_mut(),
        );

        let button_class = encode_wide("BUTTON");

        // Stop Button
        let stop_text = encode_wide("Stop Instance");
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            stop_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            20,
            60,
            260,
            35,
            hwnd,
            ID_STOP_BUTTON as isize,
            h_instance,
            null_mut(),
        );

        // Reload Button
        let reload_text = encode_wide("Reload Config");
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            reload_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            20,
            105,
            260,
            35,
            hwnd,
            ID_RELOAD_BUTTON as isize,
            h_instance,
            null_mut(),
        );

        // Edit Button
        let edit_text = encode_wide("Edit Config");
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            edit_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            20,
            150,
            260,
            35,
            hwnd,
            ID_EDIT_BUTTON as isize,
            h_instance,
            null_mut(),
        );

        ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW);
        println!("ShowWindow called.");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        println!("Message loop exited.");
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = wparam as usize;
            match id {
                ID_STOP_BUTTON => {
                    if send_msg_to_instance(WM_CLOSE) {
                        create_alert_window("Sent stop signal to instance.");
                        std::process::exit(0);
                    }
                }
                ID_RELOAD_BUTTON => {
                    if send_msg_to_instance(WM_RELOAD_CONFIG) {
                        create_alert_window("Sent reload signal to instance.");
                    }
                }
                ID_EDIT_BUTTON => {
                    let _ = std::process::Command::new("notepad.exe")
                        .arg("config.toml")
                        .spawn();
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            0
        }
        WM_CLOSE => {
            std::process::exit(0);
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn create_alert_window(message: &str) {
    let title_w = encode_wide("Capsense");
    let message_w = encode_wide(message);

    unsafe {
        MessageBoxW(
            0,
            message_w.as_ptr(),
            title_w.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
