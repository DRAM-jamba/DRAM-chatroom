use std::sync::Mutex;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use tauri::{AppHandle, Emitter};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

static HOOK_HANDLE: Mutex<Option<HHOOK>> = Mutex::new(None);
static HOOK_THREAD_ID: Mutex<Option<u32>> = Mutex::new(None);
static MIC_KEY: Mutex<Option<u32>> = Mutex::new(None);
static HEADPHONES_KEY: Mutex<Option<u32>> = Mutex::new(None);
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam as u32 == WM_KEYDOWN {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;

        let mic_key = MIC_KEY.lock().unwrap().clone();
        let headphones_key = HEADPHONES_KEY.lock().unwrap().clone();
        let app = APP_HANDLE.lock().unwrap().clone();

       if let Some(app_handle) = app {
            if Some(vk) == mic_key {
                let ah = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = ah.emit_to("main", "global_mic_hotkey", ());
                });
            }
            if Some(vk) == headphones_key {
                let ah = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = ah.emit_to("main", "global_headphones_hotkey", ());
                });
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}

pub fn register_hooks(app: AppHandle, mic_key: Option<u32>, headphones_key: Option<u32>) {
    unregister_hooks();

    *MIC_KEY.lock().unwrap() = mic_key;
    *HEADPHONES_KEY.lock().unwrap() = headphones_key;
    *APP_HANDLE.lock().unwrap() = Some(app);

    std::thread::spawn(|| unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), 0, 0);
        *HOOK_HANDLE.lock().unwrap() = Some(hook);

        let thread_id = GetCurrentThreadId();
        *HOOK_THREAD_ID.lock().unwrap() = Some(thread_id);

        let mut msg = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut msg, 0, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        *HOOK_THREAD_ID.lock().unwrap() = None;
    });
}

pub fn unregister_hooks() {
    let hook = HOOK_HANDLE.lock().unwrap().take();
    if let Some(h) = hook {
        unsafe { UnhookWindowsHookEx(h) };
    }

    let thread_id = HOOK_THREAD_ID.lock().unwrap().take();
    if let Some(tid) = thread_id {
        unsafe { PostThreadMessageW(tid, WM_QUIT, 0, 0) };
    }

    *MIC_KEY.lock().unwrap() = None;
    *HEADPHONES_KEY.lock().unwrap() = None;
    *APP_HANDLE.lock().unwrap() = None;
}

pub fn key_str_to_vk(key: &str) -> Option<u32> {
    match key.to_uppercase().as_str() {
        "A" => Some(0x41), "B" => Some(0x42), "C" => Some(0x43),
        "D" => Some(0x44), "E" => Some(0x45), "F" => Some(0x46),
        "G" => Some(0x47), "H" => Some(0x48), "I" => Some(0x49),
        "J" => Some(0x4A), "K" => Some(0x4B), "L" => Some(0x4C),
        "M" => Some(0x4D), "N" => Some(0x4E), "O" => Some(0x4F),
        "P" => Some(0x50), "Q" => Some(0x51), "R" => Some(0x52),
        "S" => Some(0x53), "T" => Some(0x54), "U" => Some(0x55),
        "V" => Some(0x56), "W" => Some(0x57), "X" => Some(0x58),
        "Y" => Some(0x59), "Z" => Some(0x5A),
        // Numbers
        "0" => Some(0x30), "1" => Some(0x31), "2" => Some(0x32),
        "3" => Some(0x33), "4" => Some(0x34), "5" => Some(0x35),
        "6" => Some(0x36), "7" => Some(0x37), "8" => Some(0x38),
        "9" => Some(0x39),
        // F keys
        "F1" => Some(0x70), "F2" => Some(0x71), "F3" => Some(0x72),
        "F4" => Some(0x73), "F5" => Some(0x74), "F6" => Some(0x75),
        "F7" => Some(0x76), "F8" => Some(0x77), "F9" => Some(0x78),
        "F10" => Some(0x79), "F11" => Some(0x7A), "F12" => Some(0x7B),
        // Special keys
        "SPACE" => Some(0x20),
        "ENTER" => Some(0x0D),
        "TAB" => Some(0x09),
        "ESCAPE" | "ESC" => Some(0x1B),
        "BACKSPACE" => Some(0x08),
        "DELETE" | "DEL" => Some(0x2E),
        "INSERT" | "INS" => Some(0x2D),
        "HOME" => Some(0x24),
        "END" => Some(0x23),
        "PAGEUP" => Some(0x21),
        "PAGEDOWN" => Some(0x22),
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),
        // Numpad
        "NUM0" => Some(0x60), "NUM1" => Some(0x61), "NUM2" => Some(0x62),
        "NUM3" => Some(0x63), "NUM4" => Some(0x64), "NUM5" => Some(0x65),
        "NUM6" => Some(0x66), "NUM7" => Some(0x67), "NUM8" => Some(0x68),
        "NUM9" => Some(0x69),
        // Punctuation & symbols
        "+" | "=" => Some(0xBB),
        "-" | "_" => Some(0xBD),
        "*" => Some(0x6A), 
        "/" | "?" => Some(0xBF),
        "\\" | "|" => Some(0xDC),
        "[" | "{" => Some(0xDB),
        "]" | "}" => Some(0xDD),
        ";" | ":" => Some(0xBA),
        "'" | "\"" => Some(0xDE),
        "," | "<" => Some(0xBC),
        "." | ">" => Some(0xBE),
        "`" | "~" => Some(0xC0),
        "NUM+" => Some(0x6B),
        "NUM-" => Some(0x6D),
        "NUM*" => Some(0x6A),
        "NUM/" => Some(0x6F),
        "NUM." => Some(0x6E),
        "NUMENTER" => Some(0x6C),
        _ => None,
    }
}