use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::InvalidateRect;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDlgItem, GetMessageW,
    MessageBoxW, PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, BS_PUSHBUTTON, CS_HREDRAW,
    CS_VREDRAW, MB_ICONINFORMATION, MB_OK, MSG, WM_CLOSE, WM_COMMAND,
    WM_DESTROY, WNDCLASSW, WS_CHILD, WS_MAXIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_THICKFRAME,
    WS_VISIBLE,
};

use crate::config::get_config_path;
use crate::hook::WM_RELOAD_CONFIG;
use crate::i18n::get_i18n;
use crate::utils::{encode_wide, is_elevated, is_process_elevated, send_msg_to_instance};

const ID_STOP_BUTTON: usize = 1;
const ID_RELOAD_BUTTON: usize = 2;
const ID_EDIT_BUTTON: usize = 3;
const ID_ENABLE_REG_BUTTON: usize = 4;
const ID_ENABLE_TASK_BUTTON: usize = 5;
const ID_DISABLE_ALL_BUTTON: usize = 6;

// IDs for dynamic labels and components that need to be destroyed/recreated on refresh
const ID_PID_LABEL: isize = 98;
const ID_PID_LABEL_ELEVATED: isize = 99;
const ID_STARTUP_STATUS_LABEL: isize = 100;
const ID_REG_STATUS_LABEL: isize = 101;
const ID_TASK_STATUS_LABEL: isize = 102;
const ID_PERMISSION_DENIED_LABEL: isize = 103;
const ID_BOTTOM_TIP_LABEL: isize = 104;

const WINDOW_WIDTH: i32 = 320;
const WINDOW_HEIGHT: i32 = 470;

const COLOR_RED: u32 = 0x0000FF;
const COLOR_GREEN: u32 = 0x008000;
const COLOR_TIP: u32 = 0x0050AA;

pub fn show_instance_manager_window(pid: u32) {
    let i18n = get_i18n();
    let window_title = encode_wide(i18n.title);
    let class_name = encode_wide("CapsenseManagerWindowClass");

    println!(
        "Attempting to show instance manager window for PID: {}",
        pid
    );

    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let cursor = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::LoadCursorW(
            0,
            windows_sys::Win32::UI::WindowsAndMessaging::IDC_ARROW,
        )
    };

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

    let register_ok = unsafe { RegisterClassW(&wnd_class) };
    if register_ok == 0 {
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        if err != 1410 {
            // ERROR_CLASS_ALREADY_EXISTS
            println!("Failed to register class. Error: {}", err);
            return;
        }
    }

    let hwnd = unsafe {
        CreateWindowExW(
            windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_APPWINDOW,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            windows_sys::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            0,
            0,
            h_instance,
            null_mut(),
        )
    };

    if hwnd == 0 {
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        println!("Failed to create window. Error: {}", err);
        return;
    }

    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
    }

    let process_elevated = is_process_elevated(pid);
    let permission_denied = !is_elevated() && process_elevated;

    // PID label
    let pid_text = encode_wide(&i18n.running_pid.replace("{}", &pid.to_string()));
    create_static(
        hwnd,
        h_instance,
        &pid_text,
        20,
        20,
        280,
        25,
        if process_elevated {
            ID_PID_LABEL_ELEVATED
        } else {
            ID_PID_LABEL
        },
    );

    if permission_denied {
        let denied_text = encode_wide(i18n.permission_denied);
        let static_hwnd = create_static(
            hwnd,
            h_instance,
            &denied_text,
            20,
            45,
            280,
            25,
            ID_PERMISSION_DENIED_LABEL,
        );
        apply_default_gui_font(static_hwnd);
    }

    let button_style = if permission_denied {
        WS_CHILD
            | WS_VISIBLE
            | BS_PUSHBUTTON as u32
            | windows_sys::Win32::UI::WindowsAndMessaging::WS_DISABLED
    } else {
        WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32
    };

    let stop_text = encode_wide(i18n.stop_instance);
    create_button(
        hwnd,
        h_instance,
        &stop_text,
        button_style,
        20,
        75,
        260,
        35,
        ID_STOP_BUTTON as isize,
    );

    let reload_text = encode_wide(i18n.reload_config);
    create_button(
        hwnd,
        h_instance,
        &reload_text,
        button_style,
        20,
        120,
        260,
        35,
        ID_RELOAD_BUTTON as isize,
    );

    let edit_text = encode_wide(i18n.edit_config);
    create_button(
        hwnd,
        h_instance,
        &edit_text,
        button_style,
        20,
        165,
        260,
        35,
        ID_EDIT_BUTTON as isize,
    );

    draw_startup_controls(hwnd, h_instance);

    unsafe {
        ShowWindow(hwnd, windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW);
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    while unsafe { GetMessageW(&mut msg, 0, 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&msg);
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
    let i18n = get_i18n();

    match msg {
        windows_sys::Win32::UI::WindowsAndMessaging::WM_CTLCOLORSTATIC => {
            let hdc = wparam as windows_sys::Win32::Graphics::Gdi::HDC;
            let static_hwnd = lparam as HWND;

            let ctrl_id = unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    static_hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::GWL_ID,
                )
            };

            unsafe {
                windows_sys::Win32::Graphics::Gdi::SetBkMode(
                    hdc,
                    windows_sys::Win32::Graphics::Gdi::TRANSPARENT as i32,
                );

                if ctrl_id == ID_PID_LABEL_ELEVATED {
                    windows_sys::Win32::Graphics::Gdi::SetTextColor(hdc, COLOR_RED);
                }

                if ctrl_id == ID_PERMISSION_DENIED_LABEL {
                    windows_sys::Win32::Graphics::Gdi::SetTextColor(hdc, COLOR_RED);
                }

                if ctrl_id == ID_REG_STATUS_LABEL || ctrl_id == ID_TASK_STATUS_LABEL {
                    let reg_enabled = crate::utils::get_startup_command().is_some();
                    let task_enabled = crate::utils::is_task_enabled();

                    let is_enabled = if ctrl_id == ID_REG_STATUS_LABEL {
                        reg_enabled
                    } else {
                        task_enabled
                    };

                    if is_enabled {
                        windows_sys::Win32::Graphics::Gdi::SetTextColor(hdc, COLOR_GREEN);
                    }
                }

                if ctrl_id == ID_BOTTOM_TIP_LABEL {
                    windows_sys::Win32::Graphics::Gdi::SetTextColor(hdc, COLOR_TIP);
                }

                windows_sys::Win32::Graphics::Gdi::GetStockObject(
                    windows_sys::Win32::Graphics::Gdi::HOLLOW_BRUSH as i32,
                ) as LRESULT
            }
        }

        WM_COMMAND => {
            // LOWORD(wparam) is control/menu ID
            let id = wparam & 0xFFFF;

            match id {
                ID_STOP_BUTTON => {
                    if send_msg_to_instance(WM_CLOSE) == Some(true) {
                        create_alert_window(i18n.stop_signal_sent);
                        unsafe { DestroyWindow(hwnd) };
                    }
                }
                ID_RELOAD_BUTTON => {
                    if send_msg_to_instance(WM_RELOAD_CONFIG) == Some(true) {
                        create_alert_window(i18n.reload_signal_sent);
                    }
                }
                ID_EDIT_BUTTON => {
                    if let Err(e) = std::process::Command::new("notepad.exe")
                        .arg(get_config_path())
                        .spawn()
                    {
                        create_alert_window(&format!("Failed to open config: {}", e));
                    }
                }
                ID_ENABLE_REG_BUTTON => {
                    if let Err(e) = crate::utils::set_startup(true, false) {
                        create_alert_window(&e);
                    } else {
                        refresh_startup_ui(hwnd);
                        create_alert_window(i18n.start_on_login);
                    }
                }
                ID_ENABLE_TASK_BUTTON => {
                    if let Err(e) = crate::utils::set_startup(true, true) {
                        create_alert_window(&e);
                    } else {
                        refresh_startup_ui(hwnd);
                        create_alert_window(i18n.start_on_login);
                    }
                }
                ID_DISABLE_ALL_BUTTON => {
                    let mut errs = Vec::new();

                    if let Err(e) = crate::utils::set_startup(false, false) {
                        errs.push(e);
                    }
                    if let Err(e) = crate::utils::set_startup(false, true) {
                        errs.push(e);
                    }

                    if errs.is_empty() {
                        refresh_startup_ui(hwnd);
                        create_alert_window(i18n.no_longer_start_on_login);
                    } else {
                        create_alert_window(&errs.join("\n"));
                    }
                }
                _ => {}
            }

            0
        }

        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }

        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn create_alert_window(message: &str) {
    let title_w = encode_wide(get_i18n().title);
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

fn refresh_startup_ui(hwnd: HWND) {
    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let ids_to_remove = [
        ID_STARTUP_STATUS_LABEL,
        ID_REG_STATUS_LABEL,
        ID_TASK_STATUS_LABEL,
        ID_ENABLE_REG_BUTTON as isize,
        ID_ENABLE_TASK_BUTTON as isize,
        ID_DISABLE_ALL_BUTTON as isize,
        ID_BOTTOM_TIP_LABEL,
    ];

    for &id in &ids_to_remove {
        let child = unsafe { GetDlgItem(hwnd, id as i32) };
        if child != 0 {
            unsafe { DestroyWindow(child) };
        }
    }

    draw_startup_controls(hwnd, h_instance);

    unsafe {
        InvalidateRect(hwnd, null_mut(), 1);
    }
}

fn draw_startup_controls(hwnd: HWND, h_instance: isize) {
    let i18n = get_i18n();

    let is_elevated_now = is_elevated();
    let reg_enabled = crate::utils::get_startup_command().is_some();
    let task_enabled = crate::utils::is_task_enabled();

    let startup_status_text = encode_wide(i18n.startup_status);
    create_static(
        hwnd,
        h_instance,
        &startup_status_text,
        20,
        215,
        280,
        20,
        ID_STARTUP_STATUS_LABEL,
    );

    let reg_status = format!(
        "  - {}: {}",
        i18n.registry,
        if reg_enabled {
            i18n.enabled
        } else {
            i18n.disabled
        }
    );
    let reg_status_text = encode_wide(&reg_status);
    create_static(
        hwnd,
        h_instance,
        &reg_status_text,
        20,
        240,
        280,
        20,
        ID_REG_STATUS_LABEL,
    );

    let task_status = format!(
        "  - {}: {}",
        i18n.task_scheduler,
        if task_enabled {
            i18n.enabled
        } else {
            i18n.disabled
        }
    );
    let task_status_text = encode_wide(&task_status);
    create_static(
        hwnd,
        h_instance,
        &task_status_text,
        20,
        265,
        280,
        20,
        ID_TASK_STATUS_LABEL,
    );

    if !reg_enabled && !task_enabled {
        let enable_reg_text = encode_wide(i18n.enable_registry);
        create_button(
            hwnd,
            h_instance,
            &enable_reg_text,
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
            20,
            300,
            125,
            35,
            ID_ENABLE_REG_BUTTON as isize,
        );

        let enable_task_text = encode_wide(i18n.enable_task);
        let enable_task_style = if is_elevated_now {
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32
        } else {
            WS_CHILD
                | WS_VISIBLE
                | BS_PUSHBUTTON as u32
                | windows_sys::Win32::UI::WindowsAndMessaging::WS_DISABLED
        };
        create_button(
            hwnd,
            h_instance,
            &enable_task_text,
            enable_task_style,
            155,
            300,
            125,
            35,
            ID_ENABLE_TASK_BUTTON as isize,
        );
    } else {
        let disable_all_text = encode_wide(i18n.disable_all);
        let disable_all_style = if !task_enabled || is_elevated_now {
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32
        } else {
            WS_CHILD
                | WS_VISIBLE
                | BS_PUSHBUTTON as u32
                | windows_sys::Win32::UI::WindowsAndMessaging::WS_DISABLED
        };
        create_button(
            hwnd,
            h_instance,
            &disable_all_text,
            disable_all_style,
            20,
            300,
            260,
            35,
            ID_DISABLE_ALL_BUTTON as isize,
        );
    }

    let tip_text = encode_wide(if !is_elevated_now && task_enabled {
        i18n.permission_denied
    } else {
        i18n.registry_limit_tip
    });
    create_static(
        hwnd,
        h_instance,
        &tip_text,
        20,
        350,
        260,
        60,
        ID_BOTTOM_TIP_LABEL,
    );
}

fn create_static(
    parent: HWND,
    h_instance: isize,
    text: &[u16],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> HWND {
    let class = encode_wide("STATIC");
    unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            width,
            height,
            parent,
            id,
            h_instance,
            null_mut(),
        )
    }
}

fn create_button(
    parent: HWND,
    h_instance: isize,
    text: &[u16],
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: isize,
) -> HWND {
    let class = encode_wide("BUTTON");
    unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            parent,
            id,
            h_instance,
            null_mut(),
        )
    }
}

fn apply_default_gui_font(hwnd: HWND) {
    unsafe {
        use windows_sys::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT};
        use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};

        let font = GetStockObject(DEFAULT_GUI_FONT as i32);
        SendMessageW(hwnd, WM_SETFONT, font as usize, 1);
    }
}
