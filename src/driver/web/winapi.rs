use lazy_static::lazy_static;
use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse;

const WAIT_TIME: std::time::Duration = std::time::Duration::from_millis(10);

#[derive(Debug)]
pub struct Key {
    virtual_key_code: u16,
    scan_code: u16,
}

lazy_static! {
    pub static ref KEYS: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        // From https://docs.google.com/spreadsheets/d/1GSj0gKDxyWAecB3SIyEZ2ssPETZkkxn67gdIwL1zFUs/edit#gid=824607963
        m.insert("LButton", Key {virtual_key_code: 0x01, scan_code: 0x000 });
        m.insert("RButton", Key {virtual_key_code: 0x02, scan_code: 0x000 });
        m.insert("CtrlBreak", Key {virtual_key_code: 0x03, scan_code: 0x146 });
        m.insert("MButton", Key {virtual_key_code: 0x04, scan_code: 0x000 });
        m.insert("XButton1", Key {virtual_key_code: 0x05, scan_code: 0x000 });
        m.insert("XButton2", Key {virtual_key_code: 0x06, scan_code: 0x000 });
        m.insert("Backspace", Key {virtual_key_code: 0x08, scan_code: 0x00E });
        m.insert("Tab", Key {virtual_key_code: 0x09, scan_code: 0x00F });
        m.insert("NumpadClear", Key {virtual_key_code: 0x0C, scan_code: 0x04C });
        m.insert("Enter", Key {virtual_key_code: 0x0D, scan_code: 0x01C });
        m.insert("Shift", Key {virtual_key_code: 0x10, scan_code: 0x02A });
        m.insert("Control", Key {virtual_key_code: 0x11, scan_code: 0x01D });
        m.insert("Alt", Key {virtual_key_code: 0x12, scan_code: 0x038 });
        m.insert("Pause", Key {virtual_key_code: 0x13, scan_code: 0x045 });
        m.insert("CapsLock", Key {virtual_key_code: 0x14, scan_code: 0x03A });
        m.insert("Escape", Key {virtual_key_code: 0x1B, scan_code: 0x001 });
        m.insert("Space", Key {virtual_key_code: 0x20, scan_code: 0x039 });
        m.insert("NumpadPgUp", Key {virtual_key_code: 0x21, scan_code: 0x049 });
        m.insert("NumpadPgDn", Key {virtual_key_code: 0x22, scan_code: 0x051 });
        m.insert("NumpadEnd", Key {virtual_key_code: 0x23, scan_code: 0x04F });
        m.insert("NumpadHome", Key {virtual_key_code: 0x24, scan_code: 0x047 });
        m.insert("NumpadLeft", Key {virtual_key_code: 0x25, scan_code: 0x04B });
        m.insert("NumpadUp", Key {virtual_key_code: 0x26, scan_code: 0x048 });
        m.insert("NumpadRight", Key {virtual_key_code: 0x27, scan_code: 0x04D });
        m.insert("NumpadDown", Key {virtual_key_code: 0x28, scan_code: 0x050 });
        m.insert("PrintScreen", Key {virtual_key_code: 0x2C, scan_code: 0x154 });
        m.insert("NumpadIns", Key {virtual_key_code: 0x2D, scan_code: 0x052 });
        m.insert("NumpadDel", Key {virtual_key_code: 0x2E, scan_code: 0x053 });
        m.insert("Help", Key {virtual_key_code: 0x2F, scan_code: 0x063 });
        m.insert("0", Key {virtual_key_code: 0x30, scan_code: 0x00B });
        m.insert("1", Key {virtual_key_code: 0x31, scan_code: 0x002 });
        m.insert("2", Key {virtual_key_code: 0x32, scan_code: 0x003 });
        m.insert("3", Key {virtual_key_code: 0x33, scan_code: 0x004 });
        m.insert("4", Key {virtual_key_code: 0x34, scan_code: 0x005 });
        m.insert("5", Key {virtual_key_code: 0x35, scan_code: 0x006 });
        m.insert("6", Key {virtual_key_code: 0x36, scan_code: 0x007 });
        m.insert("7", Key {virtual_key_code: 0x37, scan_code: 0x008 });
        m.insert("8", Key {virtual_key_code: 0x38, scan_code: 0x009 });
        m.insert("9", Key {virtual_key_code: 0x39, scan_code: 0x00A });
        m.insert("a", Key {virtual_key_code: 0x41, scan_code: 0x01E });
        m.insert("b", Key {virtual_key_code: 0x42, scan_code: 0x030 });
        m.insert("c", Key {virtual_key_code: 0x43, scan_code: 0x02E });
        m.insert("d", Key {virtual_key_code: 0x44, scan_code: 0x020 });
        m.insert("e", Key {virtual_key_code: 0x45, scan_code: 0x012 });
        m.insert("f", Key {virtual_key_code: 0x46, scan_code: 0x021 });
        m.insert("g", Key {virtual_key_code: 0x47, scan_code: 0x022 });
        m.insert("h", Key {virtual_key_code: 0x48, scan_code: 0x023 });
        m.insert("i", Key {virtual_key_code: 0x49, scan_code: 0x017 });
        m.insert("j", Key {virtual_key_code: 0x4A, scan_code: 0x024 });
        m.insert("k", Key {virtual_key_code: 0x4B, scan_code: 0x025 });
        m.insert("l", Key {virtual_key_code: 0x4C, scan_code: 0x026 });
        m.insert("m", Key {virtual_key_code: 0x4D, scan_code: 0x032 });
        m.insert("n", Key {virtual_key_code: 0x4E, scan_code: 0x031 });
        m.insert("o", Key {virtual_key_code: 0x4F, scan_code: 0x018 });
        m.insert("p", Key {virtual_key_code: 0x50, scan_code: 0x019 });
        m.insert("q", Key {virtual_key_code: 0x51, scan_code: 0x010 });
        m.insert("r", Key {virtual_key_code: 0x52, scan_code: 0x013 });
        m.insert("s", Key {virtual_key_code: 0x53, scan_code: 0x01F });
        m.insert("t", Key {virtual_key_code: 0x54, scan_code: 0x014 });
        m.insert("u", Key {virtual_key_code: 0x55, scan_code: 0x016 });
        m.insert("v", Key {virtual_key_code: 0x56, scan_code: 0x02F });
        m.insert("w", Key {virtual_key_code: 0x57, scan_code: 0x011 });
        m.insert("x", Key {virtual_key_code: 0x58, scan_code: 0x02D });
        m.insert("y", Key {virtual_key_code: 0x59, scan_code: 0x015 });
        m.insert("z", Key {virtual_key_code: 0x5A, scan_code: 0x02C });
        m.insert("LWin", Key {virtual_key_code: 0x5B, scan_code: 0x15B });
        m.insert("RWin", Key {virtual_key_code: 0x5C, scan_code: 0x15C });
        m.insert("AppsKey", Key {virtual_key_code: 0x5D, scan_code: 0x15D });
        m.insert("Sleep", Key {virtual_key_code: 0x5F, scan_code: 0x05F });
        m.insert("Numpad0", Key {virtual_key_code: 0x60, scan_code: 0x052 });
        m.insert("Numpad1", Key {virtual_key_code: 0x61, scan_code: 0x04F });
        m.insert("Numpad2", Key {virtual_key_code: 0x62, scan_code: 0x050 });
        m.insert("Numpad3", Key {virtual_key_code: 0x63, scan_code: 0x051 });
        m.insert("Numpad4", Key {virtual_key_code: 0x64, scan_code: 0x04B });
        m.insert("Numpad5", Key {virtual_key_code: 0x65, scan_code: 0x04C });
        m.insert("Numpad6", Key {virtual_key_code: 0x66, scan_code: 0x04D });
        m.insert("Numpad7", Key {virtual_key_code: 0x67, scan_code: 0x047 });
        m.insert("Numpad8", Key {virtual_key_code: 0x68, scan_code: 0x048 });
        m.insert("Numpad9", Key {virtual_key_code: 0x69, scan_code: 0x049 });
        m.insert("NumpadMult", Key {virtual_key_code: 0x6A, scan_code: 0x037 });
        m.insert("NumpadAdd", Key {virtual_key_code: 0x6B, scan_code: 0x04E });
        m.insert("NumpadSub", Key {virtual_key_code: 0x6D, scan_code: 0x04A });
        m.insert("NumpadDot", Key {virtual_key_code: 0x6E, scan_code: 0x053 });
        m.insert("NumpadDiv", Key {virtual_key_code: 0x6F, scan_code: 0x135 });
        m.insert("F1", Key {virtual_key_code: 0x70, scan_code: 0x03B });
        m.insert("F2", Key {virtual_key_code: 0x71, scan_code: 0x03C });
        m.insert("F3", Key {virtual_key_code: 0x72, scan_code: 0x03D });
        m.insert("F4", Key {virtual_key_code: 0x73, scan_code: 0x03E });
        m.insert("F5", Key {virtual_key_code: 0x74, scan_code: 0x03F });
        m.insert("F6", Key {virtual_key_code: 0x75, scan_code: 0x040 });
        m.insert("F7", Key {virtual_key_code: 0x76, scan_code: 0x041 });
        m.insert("F8", Key {virtual_key_code: 0x77, scan_code: 0x042 });
        m.insert("F9", Key {virtual_key_code: 0x78, scan_code: 0x043 });
        m.insert("F10", Key {virtual_key_code: 0x79, scan_code: 0x044 });
        m.insert("F11", Key {virtual_key_code: 0x7A, scan_code: 0x057 });
        m.insert("F12", Key {virtual_key_code: 0x7B, scan_code: 0x058 });
        m.insert("F13", Key {virtual_key_code: 0x7C, scan_code: 0x064 });
        m.insert("F14", Key {virtual_key_code: 0x7D, scan_code: 0x065 });
        m.insert("F15", Key {virtual_key_code: 0x7E, scan_code: 0x066 });
        m.insert("F16", Key {virtual_key_code: 0x7F, scan_code: 0x067 });
        m.insert("F17", Key {virtual_key_code: 0x80, scan_code: 0x068 });
        m.insert("F18", Key {virtual_key_code: 0x81, scan_code: 0x069 });
        m.insert("F19", Key {virtual_key_code: 0x82, scan_code: 0x06A });
        m.insert("F20", Key {virtual_key_code: 0x83, scan_code: 0x06B });
        m.insert("F21", Key {virtual_key_code: 0x84, scan_code: 0x06B });
        m.insert("F22", Key {virtual_key_code: 0x85, scan_code: 0x06D });
        m.insert("F23", Key {virtual_key_code: 0x86, scan_code: 0x06E });
        m.insert("F24", Key {virtual_key_code: 0x87, scan_code: 0x076 });
        m.insert("Numlock", Key {virtual_key_code: 0x90, scan_code: 0x145 });
        m.insert("ScrollLock", Key {virtual_key_code: 0x91, scan_code: 0x046 });
        m.insert("WheelLeft", Key {virtual_key_code: 0x9C, scan_code: 0x001 });
        m.insert("WheelRight", Key {virtual_key_code: 0x9D, scan_code: 0x001 });
        m.insert("WheelDown", Key {virtual_key_code: 0x9E, scan_code: 0x001 });
        m.insert("WheelUp", Key {virtual_key_code: 0x9F, scan_code: 0x001 });
        m.insert("LShift", Key {virtual_key_code: 0xA0, scan_code: 0x02A });
        m.insert("RShift", Key {virtual_key_code: 0xA1, scan_code: 0x136 });
        m.insert("LControl", Key {virtual_key_code: 0xA2, scan_code: 0x01D });
        m.insert("RControl", Key {virtual_key_code: 0xA3, scan_code: 0x11D });
        m.insert("LAlt", Key {virtual_key_code: 0xA4, scan_code: 0x038 });
        m.insert("RAlt", Key {virtual_key_code: 0xA5, scan_code: 0x138 });
        m.insert("Browser_Back", Key {virtual_key_code: 0xA6, scan_code: 0x16A });
        m.insert("Browser_Forward", Key {virtual_key_code: 0xA7, scan_code: 0x169 });
        m.insert("Browser_Refresh", Key {virtual_key_code: 0xA8, scan_code: 0x167 });
        m.insert("Browser_Stop", Key {virtual_key_code: 0xA9, scan_code: 0x168 });
        m.insert("Browser_Search", Key {virtual_key_code: 0xAA, scan_code: 0x165 });
        m.insert("Browser_Favorites", Key {virtual_key_code: 0xAB, scan_code: 0x166 });
        m.insert("Browser_Home", Key {virtual_key_code: 0xAC, scan_code: 0x132 });
        m.insert("Volume_Mute", Key {virtual_key_code: 0xAD, scan_code: 0x120 });
        m.insert("Volume_Down", Key {virtual_key_code: 0xAE, scan_code: 0x12E });
        m.insert("Volume_Up", Key {virtual_key_code: 0xAF, scan_code: 0x130 });
        m.insert("Media_Next", Key {virtual_key_code: 0xB0, scan_code: 0x119 });
        m.insert("Media_Prev", Key {virtual_key_code: 0xB1, scan_code: 0x110 });
        m.insert("Media_Stop", Key {virtual_key_code: 0xB2, scan_code: 0x124 });
        m.insert("Media_Play_Pause", Key {virtual_key_code: 0xB3, scan_code: 0x122 });
        m.insert("Launch_Mail", Key {virtual_key_code: 0xB4, scan_code: 0x16C });
        m.insert("Launch_Media", Key {virtual_key_code: 0xB5, scan_code: 0x16D });
        m.insert("Launch_App1", Key {virtual_key_code: 0xB6, scan_code: 0x16B });
        m.insert("Launch_App2", Key {virtual_key_code: 0xB7, scan_code: 0x121 });
        m.insert(";", Key {virtual_key_code: 0xBA, scan_code: 0x027 });
        m.insert("=", Key {virtual_key_code: 0xBB, scan_code: 0x00D });
        m.insert(",", Key {virtual_key_code: 0xBC, scan_code: 0x033 });
        m.insert("-", Key {virtual_key_code: 0xBD, scan_code: 0x00C });
        m.insert(".", Key {virtual_key_code: 0xBE, scan_code: 0x034 });
        m.insert("/", Key {virtual_key_code: 0xBF, scan_code: 0x035 });
        m.insert("`", Key {virtual_key_code: 0xC0, scan_code: 0x029 });
        m.insert("[", Key {virtual_key_code: 0xDB, scan_code: 0x01A });
        m.insert("\\", Key {virtual_key_code: 0xDC, scan_code: 0x02B });
        m.insert("]", Key {virtual_key_code: 0xDD, scan_code: 0x01B });
        m.insert("'", Key {virtual_key_code: 0xDE, scan_code: 0x028 });
        m.insert("\\", Key {virtual_key_code: 0xE2, scan_code: 0x056 });
        m
    };
}

/// Press and immediately release a key.
pub fn press_and_release_key(key: &Key) {
    press_key(key);
    release_key(key);
}

/// Send a key press to the active window.
pub fn press_key(key: &Key) {
    let input = KeyboardAndMouse::INPUT {
        r#type: KeyboardAndMouse::INPUT_KEYBOARD,
        Anonymous: KeyboardAndMouse::INPUT_0 {
            ki: KeyboardAndMouse::KEYBDINPUT {
                wVk: KeyboardAndMouse::VIRTUAL_KEY(key.virtual_key_code),
                wScan: key.scan_code,
                dwFlags: KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        KeyboardAndMouse::SendInput(
            &[input],
            std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
        );
    }
    std::thread::sleep(WAIT_TIME);
}

/// Send a key release to the active window.
#[allow(dead_code)]
pub fn release_key(key: &Key) {
    let input = KeyboardAndMouse::INPUT {
        r#type: KeyboardAndMouse::INPUT_KEYBOARD,
        Anonymous: KeyboardAndMouse::INPUT_0 {
            ki: KeyboardAndMouse::KEYBDINPUT {
                wVk: KeyboardAndMouse::VIRTUAL_KEY(key.virtual_key_code),
                wScan: key.scan_code,
                dwFlags: KeyboardAndMouse::KEYBD_EVENT_FLAGS(KeyboardAndMouse::KEYEVENTF_KEYUP.0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        KeyboardAndMouse::SendInput(
            &[input],
            std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32,
        );
    }
    std::thread::sleep(WAIT_TIME);
}

#[cfg(test)]
mod tests {
    use super::{press_and_release_key, press_key, release_key, KEYS};
    use crate::{
        driver::{web::WebDriver, Driver},
        solver::Solver,
    };

    #[test]
    #[ignore]
    fn enter_text() {
        let solver = Solver::default();
        let driver = WebDriver::new(solver).unwrap();
        assert!(driver.get_password().unwrap().is_empty());

        press_and_release_key(KEYS.get("f").unwrap());
        press_and_release_key(KEYS.get("o").unwrap());
        press_and_release_key(KEYS.get("o").unwrap());
        assert_eq!(driver.get_password().unwrap(), "foo");
    }

    #[test]
    #[ignore]
    fn select_text() {
        let solver = Solver::default();
        let driver = WebDriver::new(solver).unwrap();
        assert!(driver.get_password().unwrap().is_empty());

        press_and_release_key(KEYS.get("f").unwrap());
        press_and_release_key(KEYS.get("o").unwrap());
        press_and_release_key(KEYS.get("o").unwrap());
        assert_eq!(driver.get_password().unwrap(), "foo");

        for _ in 0..3 {
            press_and_release_key(KEYS.get("NumpadLeft").unwrap());
        }
        press_key(KEYS.get("Shift").unwrap());
        press_key(KEYS.get("RShift").unwrap());
        for _ in 0..3 {
            press_and_release_key(KEYS.get("NumpadRight").unwrap());
        }
        release_key(KEYS.get("Shift").unwrap());
        release_key(KEYS.get("RShift").unwrap());

        press_and_release_key(KEYS.get("b").unwrap());
        press_and_release_key(KEYS.get("a").unwrap());
        press_and_release_key(KEYS.get("r").unwrap());
        assert_eq!(driver.get_password().unwrap(), "bar");
    }
}
