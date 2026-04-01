use parking_lot::RwLock;
#[cfg(not(target_os = "macos"))]
use rdev::{Button, Event, EventType};
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Alt,
    AltGr,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    MetaLeft,
    MetaRight,
    Space,
    Return,
    Escape,
    Backspace,
    Tab,
    CapsLock,
    Delete,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
    KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
    KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    PageDown, PageUp, Home, End,
    UpArrow, DownArrow, LeftArrow, RightArrow,
}

#[derive(Debug, Clone)]
pub enum ShortcutEvent {
    HoldPress,
    HoldRelease,
    TogglePress,
    PastePress,
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutConfig {
    pub hold: ParsedShortcut,
    pub toggle: ParsedShortcut,
    pub paste: ParsedShortcut,
}

#[derive(Debug, Clone)]
pub enum ParsedShortcut {
    None,
    SingleKey(KeyCode),
    MouseButton(u8),
    Combo(Vec<KeyCode>),
}

impl Default for ParsedShortcut {
    fn default() -> Self {
        ParsedShortcut::None
    }
}

#[derive(Clone)]
pub struct GrabHandleInner {
    pub enabled: Arc<AtomicBool>,
    pub config: Arc<RwLock<ShortcutConfig>>,
    pub capture_mode: Arc<AtomicBool>,
    pub capture_tx: Arc<parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
}

fn key_code_from_name(name: &str) -> Option<KeyCode> {
    match name {
        "Alt_L" => Some(KeyCode::Alt),
        "Alt_R" => Some(KeyCode::AltGr),
        "Shift_L" => Some(KeyCode::ShiftLeft),
        "Shift_R" => Some(KeyCode::ShiftRight),
        "Control_L" => Some(KeyCode::ControlLeft),
        "Control_R" => Some(KeyCode::ControlRight),
        "Super_L" => Some(KeyCode::MetaLeft),
        "Super_R" => Some(KeyCode::MetaRight),
        "space" => Some(KeyCode::Space),
        "Return" => Some(KeyCode::Return),
        "Escape" => Some(KeyCode::Escape),
        "BackSpace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "Caps_Lock" => Some(KeyCode::CapsLock),
        "Delete" => Some(KeyCode::Delete),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "a" => Some(KeyCode::KeyA),
        "b" => Some(KeyCode::KeyB),
        "c" => Some(KeyCode::KeyC),
        "d" => Some(KeyCode::KeyD),
        "e" => Some(KeyCode::KeyE),
        "f" => Some(KeyCode::KeyF),
        "g" => Some(KeyCode::KeyG),
        "h" => Some(KeyCode::KeyH),
        "i" => Some(KeyCode::KeyI),
        "j" => Some(KeyCode::KeyJ),
        "k" => Some(KeyCode::KeyK),
        "l" => Some(KeyCode::KeyL),
        "m" => Some(KeyCode::KeyM),
        "n" => Some(KeyCode::KeyN),
        "o" => Some(KeyCode::KeyO),
        "p" => Some(KeyCode::KeyP),
        "q" => Some(KeyCode::KeyQ),
        "r" => Some(KeyCode::KeyR),
        "s" => Some(KeyCode::KeyS),
        "t" => Some(KeyCode::KeyT),
        "u" => Some(KeyCode::KeyU),
        "v" => Some(KeyCode::KeyV),
        "w" => Some(KeyCode::KeyW),
        "x" => Some(KeyCode::KeyX),
        "y" => Some(KeyCode::KeyY),
        "z" => Some(KeyCode::KeyZ),
        "0" => Some(KeyCode::Num0),
        "1" => Some(KeyCode::Num1),
        "2" => Some(KeyCode::Num2),
        "3" => Some(KeyCode::Num3),
        "4" => Some(KeyCode::Num4),
        "5" => Some(KeyCode::Num5),
        "6" => Some(KeyCode::Num6),
        "7" => Some(KeyCode::Num7),
        "8" => Some(KeyCode::Num8),
        "9" => Some(KeyCode::Num9),
        "Page_Down" => Some(KeyCode::PageDown),
        "Page_Up" => Some(KeyCode::PageUp),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "Up" => Some(KeyCode::UpArrow),
        "Down" => Some(KeyCode::DownArrow),
        "Left" => Some(KeyCode::LeftArrow),
        "Right" => Some(KeyCode::RightArrow),
        _ => None,
    }
}

fn is_modifier(code: &KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Alt
            | KeyCode::AltGr
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::MetaLeft
            | KeyCode::MetaRight
    )
}

fn button_code_from_name(name: &str) -> Option<u8> {
    match name {
        "left" => Some(0),
        "right" => Some(1),
        "middle" => Some(2),
        "back" => Some(3),
        "forward" => Some(4),
        _ => None,
    }
}

fn button_name_from_code(code: u8) -> &'static str {
    match code {
        0 => "left",
        1 => "right",
        2 => "middle",
        3 => "back",
        4 => "forward",
        _ => "unknown",
    }
}

#[cfg(not(target_os = "macos"))]
fn button_from_rdev(btn: &Button) -> u8 {
    match btn {
        Button::Left => 0,
        Button::Right => 1,
        Button::Middle => 2,
        Button::Unknown(3) => 3,
        Button::Unknown(4) => 4,
        Button::Unknown(8) => 3,
        Button::Unknown(9) => 4,
        _ => 255,
    }
}

#[cfg(not(target_os = "macos"))]
fn key_from_rdev(key: &rdev::Key) -> Option<KeyCode> {
    use rdev::Key;
    match key {
        Key::Alt => Some(KeyCode::Alt),
        Key::AltGr => Some(KeyCode::AltGr),
        Key::ShiftLeft => Some(KeyCode::ShiftLeft),
        Key::ShiftRight => Some(KeyCode::ShiftRight),
        Key::ControlLeft => Some(KeyCode::ControlLeft),
        Key::ControlRight => Some(KeyCode::ControlRight),
        Key::MetaLeft => Some(KeyCode::MetaLeft),
        Key::MetaRight => Some(KeyCode::MetaRight),
        Key::Space => Some(KeyCode::Space),
        Key::Return => Some(KeyCode::Return),
        Key::Escape => Some(KeyCode::Escape),
        Key::Backspace => Some(KeyCode::Backspace),
        Key::Tab => Some(KeyCode::Tab),
        Key::CapsLock => Some(KeyCode::CapsLock),
        Key::Delete => Some(KeyCode::Delete),
        Key::F1 => Some(KeyCode::F1),
        Key::F2 => Some(KeyCode::F2),
        Key::F3 => Some(KeyCode::F3),
        Key::F4 => Some(KeyCode::F4),
        Key::F5 => Some(KeyCode::F5),
        Key::F6 => Some(KeyCode::F6),
        Key::F7 => Some(KeyCode::F7),
        Key::F8 => Some(KeyCode::F8),
        Key::F9 => Some(KeyCode::F9),
        Key::F10 => Some(KeyCode::F10),
        Key::F11 => Some(KeyCode::F11),
        Key::F12 => Some(KeyCode::F12),
        Key::KeyA => Some(KeyCode::KeyA),
        Key::KeyB => Some(KeyCode::KeyB),
        Key::KeyC => Some(KeyCode::KeyC),
        Key::KeyD => Some(KeyCode::KeyD),
        Key::KeyE => Some(KeyCode::KeyE),
        Key::KeyF => Some(KeyCode::KeyF),
        Key::KeyG => Some(KeyCode::KeyG),
        Key::KeyH => Some(KeyCode::KeyH),
        Key::KeyI => Some(KeyCode::KeyI),
        Key::KeyJ => Some(KeyCode::KeyJ),
        Key::KeyK => Some(KeyCode::KeyK),
        Key::KeyL => Some(KeyCode::KeyL),
        Key::KeyM => Some(KeyCode::KeyM),
        Key::KeyN => Some(KeyCode::KeyN),
        Key::KeyO => Some(KeyCode::KeyO),
        Key::KeyP => Some(KeyCode::KeyP),
        Key::KeyQ => Some(KeyCode::KeyQ),
        Key::KeyR => Some(KeyCode::KeyR),
        Key::KeyS => Some(KeyCode::KeyS),
        Key::KeyT => Some(KeyCode::KeyT),
        Key::KeyU => Some(KeyCode::KeyU),
        Key::KeyV => Some(KeyCode::KeyV),
        Key::KeyW => Some(KeyCode::KeyW),
        Key::KeyX => Some(KeyCode::KeyX),
        Key::KeyY => Some(KeyCode::KeyY),
        Key::KeyZ => Some(KeyCode::KeyZ),
        Key::Num0 => Some(KeyCode::Num0),
        Key::Num1 => Some(KeyCode::Num1),
        Key::Num2 => Some(KeyCode::Num2),
        Key::Num3 => Some(KeyCode::Num3),
        Key::Num4 => Some(KeyCode::Num4),
        Key::Num5 => Some(KeyCode::Num5),
        Key::Num6 => Some(KeyCode::Num6),
        Key::Num7 => Some(KeyCode::Num7),
        Key::Num8 => Some(KeyCode::Num8),
        Key::Num9 => Some(KeyCode::Num9),
        Key::PageDown => Some(KeyCode::PageDown),
        Key::PageUp => Some(KeyCode::PageUp),
        Key::Home => Some(KeyCode::Home),
        Key::End => Some(KeyCode::End),
        Key::UpArrow => Some(KeyCode::UpArrow),
        Key::DownArrow => Some(KeyCode::DownArrow),
        Key::LeftArrow => Some(KeyCode::LeftArrow),
        Key::RightArrow => Some(KeyCode::RightArrow),
        _ => None,
    }
}

pub fn parse_shortcut(s: &str) -> ParsedShortcut {
    if s.is_empty() {
        return ParsedShortcut::None;
    }
    if let Some(mouse) = s.strip_prefix("mouse:") {
        return match button_code_from_name(mouse) {
            Some(b) => ParsedShortcut::MouseButton(b),
            None => ParsedShortcut::None,
        };
    }
    if let Some(combo_str) = s.strip_prefix("combo:") {
        let keys: Vec<KeyCode> = combo_str
            .split('+')
            .filter_map(key_code_from_name)
            .collect();
        if keys.len() >= 2 {
            return ParsedShortcut::Combo(keys);
        }
        return ParsedShortcut::None;
    }
    if let Some(key_name) = s.strip_prefix("key:") {
        return match key_code_from_name(key_name) {
            Some(k) => ParsedShortcut::SingleKey(k),
            None => ParsedShortcut::None,
        };
    }
    ParsedShortcut::None
}

pub fn button_to_name(code: u8) -> String {
    format!("mouse:{}", button_name_from_code(code))
}

fn keys_match_combo(pressed: &HashSet<KeyCode>, combo: &[KeyCode]) -> bool {
    combo.iter().all(|k| pressed.contains(k))
}

pub type GrabHandle = GrabHandleInner;

struct GrabState {
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
    pressed: HashSet<KeyCode>,
    combo_hold_active: bool,
    combo_toggle_active: bool,
    combo_paste_active: bool,
    paste_pending: bool,
}

impl GrabState {
    fn process_key_press(&mut self, config: &ShortcutConfig, key: KeyCode) -> bool {
        let mut suppress = false;

        if let ParsedShortcut::SingleKey(k) = &config.hold {
            if key == *k {
                self.event_tx.send(ShortcutEvent::HoldPress).ok();
                suppress = true;
            }
        }
        if let ParsedShortcut::SingleKey(k) = &config.toggle {
            if key == *k {
                self.event_tx.send(ShortcutEvent::TogglePress).ok();
                suppress = true;
            }
        }
        if let ParsedShortcut::SingleKey(k) = &config.paste {
            if key == *k {
                self.paste_pending = true;
                suppress = true;
            }
        }

        if let ParsedShortcut::Combo(ref combo) = config.hold {
            if !self.combo_hold_active && keys_match_combo(&self.pressed, combo) {
                self.combo_hold_active = true;
                self.event_tx.send(ShortcutEvent::HoldPress).ok();
                if !is_modifier(&key) {
                    suppress = true;
                }
            }
        }
        if let ParsedShortcut::Combo(ref combo) = config.toggle {
            if !self.combo_toggle_active && keys_match_combo(&self.pressed, combo) {
                self.combo_toggle_active = true;
                self.event_tx.send(ShortcutEvent::TogglePress).ok();
                if !is_modifier(&key) {
                    suppress = true;
                }
            }
        }
        if let ParsedShortcut::Combo(ref combo) = config.paste {
            if !self.combo_paste_active && keys_match_combo(&self.pressed, combo) {
                self.combo_paste_active = true;
                self.paste_pending = true;
                if !is_modifier(&key) {
                    suppress = true;
                }
            }
        }

        suppress
    }

    fn process_key_release(&mut self, config: &ShortcutConfig, key: KeyCode) -> bool {
        let mut suppress = false;

        if let ParsedShortcut::SingleKey(k) = &config.hold {
            if key == *k {
                self.event_tx.send(ShortcutEvent::HoldRelease).ok();
                suppress = true;
            }
        }

        if let ParsedShortcut::SingleKey(k) = &config.paste {
            if key == *k && self.paste_pending {
                self.paste_pending = false;
                self.event_tx.send(ShortcutEvent::PastePress).ok();
                suppress = true;
            }
        }

        if self.combo_hold_active {
            if let ParsedShortcut::Combo(ref combo) = config.hold {
                if !keys_match_combo(&self.pressed, combo) {
                    self.combo_hold_active = false;
                    self.event_tx.send(ShortcutEvent::HoldRelease).ok();
                }
            }
        }
        if self.combo_toggle_active {
            if let ParsedShortcut::Combo(ref combo) = config.toggle {
                if !keys_match_combo(&self.pressed, combo) {
                    self.combo_toggle_active = false;
                }
            }
        }
        if self.combo_paste_active {
            if let ParsedShortcut::Combo(ref combo) = config.paste {
                if !keys_match_combo(&self.pressed, combo) {
                    self.combo_paste_active = false;
                    if self.paste_pending {
                        self.paste_pending = false;
                        self.event_tx.send(ShortcutEvent::PastePress).ok();
                    }
                }
            }
        }

        suppress
    }

    fn process_button_press(&mut self, config: &ShortcutConfig, btn: u8) -> bool {
        let mut suppress = false;
        
        if let ParsedShortcut::MouseButton(b) = &config.hold {
            if btn == *b {
                self.event_tx.send(ShortcutEvent::HoldPress).ok();
                suppress = true;
            }
        }
        if let ParsedShortcut::MouseButton(b) = &config.toggle {
            if btn == *b {
                self.event_tx.send(ShortcutEvent::TogglePress).ok();
                suppress = true;
            }
        }
        if let ParsedShortcut::MouseButton(b) = &config.paste {
            if btn == *b {
                self.paste_pending = true;
                suppress = true;
            }
        }
        suppress
    }

    fn process_button_release(&mut self, config: &ShortcutConfig, btn: u8) -> bool {
        let mut suppress = false;
        
        if let ParsedShortcut::MouseButton(b) = &config.hold {
            if btn == *b {
                self.event_tx.send(ShortcutEvent::HoldRelease).ok();
                suppress = true;
            }
        }
        if let ParsedShortcut::MouseButton(b) = &config.paste {
            if btn == *b && self.paste_pending {
                self.paste_pending = false;
                self.event_tx.send(ShortcutEvent::PastePress).ok();
                suppress = true;
            }
        }
        suppress
    }
}

// ---------------------------------------------------------------------------
// Linux
// ---------------------------------------------------------------------------
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let state = Arc::new(parking_lot::Mutex::new(GrabState {
            event_tx,
            pressed: HashSet::new(),
            combo_hold_active: false,
            combo_toggle_active: false,
            combo_paste_active: false,
            paste_pending: false,
        }));

        let callback_state = Arc::clone(&state);
        let callback = move |event: Event| -> Option<Event> {
            let mut guard = callback_state.lock();

            if handle.capture_mode.load(Ordering::SeqCst) {
                if let EventType::ButtonPress(btn) = event.event_type {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code != 255 && btn_code != 0 {
                        let name = button_to_name(btn_code);
                        if let Some(tx) = handle.capture_tx.lock().take() {
                            tx.send(name).ok();
                        }
                        return None;
                    }
                }
                return Some(event);
            }

            if !handle.enabled.load(Ordering::SeqCst) {
                if let EventType::KeyRelease(key) = event.event_type {
                    if let Some(kc) = key_from_rdev(&key) {
                        guard.pressed.remove(&kc);
                    }
                }
                return Some(event);
            }

            let config = handle.config.read().clone();

            match event.event_type {
                EventType::KeyPress(key) => {
                    if let Some(kc) = key_from_rdev(&key) {
                        if !guard.pressed.insert(kc) {
                            return Some(event);
                        }
                        let suppress = guard.process_key_press(&config, kc);
                        if suppress { None } else { Some(event) }
                    } else {
                        Some(event)
                    }
                }
                EventType::KeyRelease(key) => {
                    if let Some(kc) = key_from_rdev(&key) {
                        guard.pressed.remove(&kc);
                        let suppress = guard.process_key_release(&config, kc);
                        if suppress { None } else { Some(event) }
                    } else {
                        Some(event)
                    }
                }
                EventType::ButtonPress(btn) => {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code == 255 {
                        return Some(event);
                    }
                    let suppress = guard.process_button_press(&config, btn_code);
                    if suppress { None } else { Some(event) }
                }
                EventType::ButtonRelease(btn) => {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code == 255 {
                        return Some(event);
                    }
                    let suppress = guard.process_button_release(&config, btn_code);
                    if suppress { None } else { Some(event) }
                }
                _ => Some(event),
            }
        };

        if let Err(e) = rdev::grab(callback) {
            log::error!("[openbolo] Failed to grab input events: {:?}", e);
            eprintln!("[openbolo] Failed to grab input events: {:?}", e);
            eprintln!("[openbolo] On Linux: Ensure you're in the 'input' group or run with appropriate permissions.");
        }
    })
}

// ---------------------------------------------------------------------------
// macOS – raw CGEventTap via C API (no Rust wrappers)
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
fn key_from_cg_keycode(code: i64) -> Option<KeyCode> {
    match code {
        0x00 => Some(KeyCode::KeyA),
        0x01 => Some(KeyCode::KeyS),
        0x02 => Some(KeyCode::KeyD),
        0x03 => Some(KeyCode::KeyF),
        0x04 => Some(KeyCode::KeyH),
        0x05 => Some(KeyCode::KeyG),
        0x06 => Some(KeyCode::KeyZ),
        0x07 => Some(KeyCode::KeyX),
        0x08 => Some(KeyCode::KeyC),
        0x09 => Some(KeyCode::KeyV),
        0x0B => Some(KeyCode::KeyB),
        0x0C => Some(KeyCode::KeyQ),
        0x0D => Some(KeyCode::KeyW),
        0x0E => Some(KeyCode::KeyE),
        0x0F => Some(KeyCode::KeyR),
        0x10 => Some(KeyCode::KeyY),
        0x11 => Some(KeyCode::KeyT),
        0x12 => Some(KeyCode::Num1),
        0x13 => Some(KeyCode::Num2),
        0x14 => Some(KeyCode::Num3),
        0x15 => Some(KeyCode::Num4),
        0x16 => Some(KeyCode::Num6),
        0x17 => Some(KeyCode::Num5),
        0x19 => Some(KeyCode::Num9),
        0x1A => Some(KeyCode::Num7),
        0x1C => Some(KeyCode::Num8),
        0x1D => Some(KeyCode::Num0),
        0x1F => Some(KeyCode::KeyO),
        0x20 => Some(KeyCode::KeyU),
        0x22 => Some(KeyCode::KeyI),
        0x23 => Some(KeyCode::KeyP),
        0x24 => Some(KeyCode::Return),
        0x25 => Some(KeyCode::KeyL),
        0x26 => Some(KeyCode::KeyJ),
        0x28 => Some(KeyCode::KeyK),
        0x2D => Some(KeyCode::KeyN),
        0x2E => Some(KeyCode::KeyM),
        0x30 => Some(KeyCode::Tab),
        0x31 => Some(KeyCode::Space),
        0x33 => Some(KeyCode::Backspace),
        0x35 => Some(KeyCode::Escape),
        0x36 => Some(KeyCode::MetaRight),
        0x37 => Some(KeyCode::MetaLeft),
        0x38 => Some(KeyCode::ShiftLeft),
        0x39 => Some(KeyCode::CapsLock),
        0x3A => Some(KeyCode::Alt),
        0x3B => Some(KeyCode::ControlLeft),
        0x3C => Some(KeyCode::ShiftRight),
        0x3D => Some(KeyCode::AltGr),
        0x3E => Some(KeyCode::ControlRight),
        0x60 => Some(KeyCode::F5),
        0x61 => Some(KeyCode::F6),
        0x62 => Some(KeyCode::F7),
        0x63 => Some(KeyCode::F3),
        0x64 => Some(KeyCode::F8),
        0x65 => Some(KeyCode::F9),
        0x67 => Some(KeyCode::F11),
        0x6D => Some(KeyCode::F10),
        0x6F => Some(KeyCode::F12),
        0x73 => Some(KeyCode::Home),
        0x74 => Some(KeyCode::PageUp),
        0x75 => Some(KeyCode::Delete),
        0x76 => Some(KeyCode::F4),
        0x77 => Some(KeyCode::End),
        0x78 => Some(KeyCode::F2),
        0x79 => Some(KeyCode::PageDown),
        0x7A => Some(KeyCode::F1),
        0x7B => Some(KeyCode::LeftArrow),
        0x7C => Some(KeyCode::RightArrow),
        0x7D => Some(KeyCode::DownArrow),
        0x7E => Some(KeyCode::UpArrow),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
mod cg_raw {
    use std::ffi::c_void;
    pub type CGEventRef = *mut c_void;
    pub type CFMachPortRef = *mut c_void;
    pub type CGEventTapProxy = *mut c_void;
    pub type CFAllocatorRef = *const c_void;
    pub type CFRunLoopSourceRef = *mut c_void;
    pub type CFRunLoopRef = *mut c_void;
    pub type CFStringRef = *const c_void;

    pub type CGEventTapCallBack = unsafe extern "C" fn(
        proxy: CGEventTapProxy,
        event_type: u32,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef;

    pub const K_CG_HID_EVENT_TAP: u32 = 0;
    pub const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
    pub const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

    pub const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
    pub const K_CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
    pub const K_CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
    pub const K_CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
    pub const K_CG_EVENT_KEY_DOWN: u32 = 10;
    pub const K_CG_EVENT_KEY_UP: u32 = 11;
    pub const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
    pub const K_CG_EVENT_OTHER_MOUSE_DOWN: u32 = 25;
    pub const K_CG_EVENT_OTHER_MOUSE_UP: u32 = 26;

    pub const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
    pub const K_CG_MOUSE_EVENT_BUTTON_NUMBER: u32 = 87;

    pub fn event_mask(types: &[u32]) -> u64 {
        types.iter().fold(0u64, |mask, &t| mask | (1u64 << t))
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events_of_interest: u64,
            callback: CGEventTapCallBack,
            user_info: *mut c_void,
        ) -> CFMachPortRef;

        pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

        pub fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub static kCFAllocatorDefault: CFAllocatorRef;

        pub fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: i64,
        ) -> CFRunLoopSourceRef;

        pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;

        pub fn CFRunLoopAddSource(
            rl: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );

        pub fn CFRunLoopRun();

        pub static kCFRunLoopCommonModes: CFStringRef;
    }
}

#[cfg(target_os = "macos")]
struct TapContext {
    handle: GrabHandle,
    state: parking_lot::Mutex<GrabState>,
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn macos_tap_callback(
    _proxy: cg_raw::CGEventTapProxy,
    event_type: u32,
    event: cg_raw::CGEventRef,
    user_info: *mut std::ffi::c_void,
) -> cg_raw::CGEventRef {
    use cg_raw::*;

    if event.is_null() || user_info.is_null() {
        return event;
    }

    // Tap-disabled notification (event_type ~0u32 on some macOS versions)
    if event_type > 100 {
        return event;
    }

    let ctx = &*(user_info as *const TapContext);
    let mut guard = ctx.state.lock();

    // --- capture mode (mouse-only, for settings UI) ---
    if ctx.handle.capture_mode.load(Ordering::SeqCst) {
        match event_type {
            K_CG_EVENT_LEFT_MOUSE_DOWN
            | K_CG_EVENT_RIGHT_MOUSE_DOWN
            | K_CG_EVENT_OTHER_MOUSE_DOWN => {
                let btn_code =
                    CGEventGetIntegerValueField(event, K_CG_MOUSE_EVENT_BUTTON_NUMBER) as u8;
                if btn_code != 0 {
                    let name = button_to_name(btn_code);
                    if let Some(tx) = ctx.handle.capture_tx.lock().take() {
                        tx.send(name).ok();
                    }
                    return std::ptr::null_mut();
                }
            }
            _ => {}
        }
        return event;
    }

    // --- shortcuts disabled ---
    if !ctx.handle.enabled.load(Ordering::SeqCst) {
        match event_type {
            K_CG_EVENT_KEY_UP => {
                let kc = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);
                if let Some(key) = key_from_cg_keycode(kc) {
                    guard.pressed.remove(&key);
                }
            }
            K_CG_EVENT_FLAGS_CHANGED => {
                let kc = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);
                if let Some(key) = key_from_cg_keycode(kc) {
                    guard.pressed.remove(&key);
                }
            }
            _ => {}
        }
        return event;
    }

    let config = ctx.handle.config.read().clone();

    match event_type {
        K_CG_EVENT_KEY_DOWN => {
            let kc = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);
            if let Some(key) = key_from_cg_keycode(kc) {
                if !guard.pressed.insert(key) {
                    return event; // auto-repeat
                }
                if guard.process_key_press(&config, key) {
                    return std::ptr::null_mut();
                }
            }
            event
        }
        K_CG_EVENT_KEY_UP => {
            let kc = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);
            if let Some(key) = key_from_cg_keycode(kc) {
                guard.pressed.remove(&key);
                if guard.process_key_release(&config, key) {
                    return std::ptr::null_mut();
                }
            }
            event
        }
        K_CG_EVENT_FLAGS_CHANGED => {
            let kc = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);
            if let Some(key) = key_from_cg_keycode(kc) {
                if guard.pressed.contains(&key) {
                    guard.pressed.remove(&key);
                    if guard.process_key_release(&config, key) {
                        return std::ptr::null_mut();
                    }
                } else {
                    guard.pressed.insert(key);
                    if guard.process_key_press(&config, key) {
                        return std::ptr::null_mut();
                    }
                }
            }
            event
        }
        K_CG_EVENT_LEFT_MOUSE_DOWN
        | K_CG_EVENT_RIGHT_MOUSE_DOWN
        | K_CG_EVENT_OTHER_MOUSE_DOWN => {
            let btn_code =
                CGEventGetIntegerValueField(event, K_CG_MOUSE_EVENT_BUTTON_NUMBER) as u8;
            if guard.process_button_press(&config, btn_code) {
                std::ptr::null_mut()
            } else {
                event
            }
        }
        K_CG_EVENT_LEFT_MOUSE_UP
        | K_CG_EVENT_RIGHT_MOUSE_UP
        | K_CG_EVENT_OTHER_MOUSE_UP => {
            let btn_code =
                CGEventGetIntegerValueField(event, K_CG_MOUSE_EVENT_BUTTON_NUMBER) as u8;
            if guard.process_button_release(&config, btn_code) {
                std::ptr::null_mut()
            } else {
                event
            }
        }
        _ => event,
    }
}

#[cfg(target_os = "macos")]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let ctx = Box::new(TapContext {
            handle,
            state: parking_lot::Mutex::new(GrabState {
                event_tx,
                pressed: HashSet::new(),
                combo_hold_active: false,
                combo_toggle_active: false,
                combo_paste_active: false,
                paste_pending: false,
            }),
        });
        let ctx_ptr = Box::into_raw(ctx) as *mut std::ffi::c_void;

        let mask = cg_raw::event_mask(&[
            cg_raw::K_CG_EVENT_KEY_DOWN,
            cg_raw::K_CG_EVENT_KEY_UP,
            cg_raw::K_CG_EVENT_FLAGS_CHANGED,
            cg_raw::K_CG_EVENT_LEFT_MOUSE_DOWN,
            cg_raw::K_CG_EVENT_LEFT_MOUSE_UP,
            cg_raw::K_CG_EVENT_RIGHT_MOUSE_DOWN,
            cg_raw::K_CG_EVENT_RIGHT_MOUSE_UP,
            cg_raw::K_CG_EVENT_OTHER_MOUSE_DOWN,
            cg_raw::K_CG_EVENT_OTHER_MOUSE_UP,
        ]);

        unsafe {
            eprintln!("[openbolo] Creating CGEventTap with mask {:#x}", mask);

            let tap = cg_raw::CGEventTapCreate(
                cg_raw::K_CG_HID_EVENT_TAP,
                cg_raw::K_CG_HEAD_INSERT_EVENT_TAP,
                cg_raw::K_CG_EVENT_TAP_OPTION_DEFAULT,
                mask,
                macos_tap_callback,
                ctx_ptr,
            );

            if tap.is_null() {
                eprintln!("[openbolo] CGEventTapCreate FAILED — grant Accessibility permission.");
                return;
            }
            eprintln!("[openbolo] CGEventTap created successfully.");

            cg_raw::CGEventTapEnable(tap, true);

            let source = cg_raw::CFMachPortCreateRunLoopSource(
                cg_raw::kCFAllocatorDefault,
                tap,
                0,
            );
            if source.is_null() {
                eprintln!("[openbolo] CFMachPortCreateRunLoopSource FAILED.");
                return;
            }

            let rl = cg_raw::CFRunLoopGetCurrent();
            cg_raw::CFRunLoopAddSource(rl, source, cg_raw::kCFRunLoopCommonModes);
            eprintln!("[openbolo] CGEventTap added to run loop, starting...");

            // Watchdog: re-enable tap every 2s in case macOS disabled it
            let tap_usize = tap as usize;
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                cg_raw::CGEventTapEnable(tap_usize as *mut std::ffi::c_void, true);
            });

            cg_raw::CFRunLoopRun();
        }
    })
}

// ---------------------------------------------------------------------------
// Windows – rdev::listen (no suppression support)
// ---------------------------------------------------------------------------
#[cfg(target_os = "windows")]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let state = Arc::new(parking_lot::Mutex::new(GrabState {
            event_tx,
            pressed: HashSet::new(),
            combo_hold_active: false,
            combo_toggle_active: false,
            combo_paste_active: false,
            paste_pending: false,
        }));

        let callback_state = Arc::clone(&state);
        let callback = move |event: Event| {
            let mut guard = callback_state.lock();

            if handle.capture_mode.load(Ordering::SeqCst) {
                if let EventType::ButtonPress(btn) = event.event_type {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code != 255 && btn_code != 0 {
                        let name = button_to_name(btn_code);
                        if let Some(tx) = handle.capture_tx.lock().take() {
                            tx.send(name).ok();
                        }
                    }
                }
                return;
            }

            if !handle.enabled.load(Ordering::SeqCst) {
                if let EventType::KeyRelease(key) = event.event_type {
                    if let Some(kc) = key_from_rdev(&key) {
                        guard.pressed.remove(&kc);
                    }
                }
                return;
            }

            let config = handle.config.read().clone();

            match event.event_type {
                EventType::KeyPress(key) => {
                    if let Some(kc) = key_from_rdev(&key) {
                        if guard.pressed.insert(kc) {
                            guard.process_key_press(&config, kc);
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    if let Some(kc) = key_from_rdev(&key) {
                        guard.pressed.remove(&kc);
                        guard.process_key_release(&config, kc);
                    }
                }
                EventType::ButtonPress(btn) => {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code != 255 {
                        guard.process_button_press(&config, btn_code);
                    }
                }
                EventType::ButtonRelease(btn) => {
                    let btn_code = button_from_rdev(&btn);
                    if btn_code != 255 {
                        guard.process_button_release(&config, btn_code);
                    }
                }
                _ => {}
            }
        };

        if let Err(e) = rdev::listen(callback) {
            eprintln!("[openbolo] Failed to listen to input events: {:?}", e);
            eprintln!("[openbolo] On Windows: Run as administrator if needed.");
        }
    })
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------
fn get_os() -> &'static str {
    std::env::consts::OS
}

pub fn shortcut_display(s: &str) -> String {
    if s.is_empty() {
        return "Not set".into();
    }
    if let Some(mouse) = s.strip_prefix("mouse:") {
        return match mouse {
            "left" => "Left Click",
            "right" => "Right Click",
            "middle" => "Middle Click",
            "back" => "Back Button",
            "forward" => "Forward Button",
            _ => mouse,
        }
        .to_string();
    }
    if let Some(combo) = s.strip_prefix("combo:") {
        return combo
            .split('+')
            .map(key_display_name)
            .collect::<Vec<_>>()
            .join(" + ");
    }
    if let Some(key_name) = s.strip_prefix("key:") {
        return key_display_name(key_name);
    }
    s.to_string()
}

fn key_display_name(name: &str) -> String {
    let os = get_os();
    match name {
        "Alt_L" => {
            if os == "macos" {
                "Left Option \u{2325}".into()
            } else {
                "Left Alt".into()
            }
        }
        "Alt_R" => {
            if os == "macos" {
                "Right Option \u{2325}".into()
            } else {
                "Right Alt".into()
            }
        }
        "Shift_L" => {
            if os == "macos" {
                "Left Shift \u{21e7}".into()
            } else {
                "Left Shift".into()
            }
        }
        "Shift_R" => {
            if os == "macos" {
                "Right Shift \u{21e7}".into()
            } else {
                "Right Shift".into()
            }
        }
        "Control_L" => {
            if os == "macos" {
                "Left Control \u{2303}".into()
            } else {
                "Left Ctrl".into()
            }
        }
        "Control_R" => {
            if os == "macos" {
                "Right Control \u{2303}".into()
            } else {
                "Right Ctrl".into()
            }
        }
        "Super_L" => {
            if os == "macos" {
                "Left Command \u{2318}".into()
            } else if os == "windows" {
                "Left Win".into()
            } else {
                "Left Super".into()
            }
        }
        "Super_R" => {
            if os == "macos" {
                "Right Command \u{2318}".into()
            } else if os == "windows" {
                "Right Win".into()
            } else {
                "Right Super".into()
            }
        }
        "space" => "Space".into(),
        "Return" => {
            if os == "macos" {
                "Return \u{21a9}".into()
            } else {
                "Enter".into()
            }
        }
        "Escape" => "Esc".into(),
        "BackSpace" => {
            if os == "macos" {
                "Delete \u{232b}".into()
            } else {
                "Backspace".into()
            }
        }
        "Tab" => {
            if os == "macos" {
                "Tab \u{21e5}".into()
            } else {
                "Tab".into()
            }
        }
        "Page_Down" => "Page Down".into(),
        "Page_Up" => "Page Up".into(),
        "Home" => "Home".into(),
        "End" => "End".into(),
        "Up" => "Up Arrow".into(),
        "Down" => "Down Arrow".into(),
        "Left" => "Left Arrow".into(),
        "Right" => "Right Arrow".into(),
        s if s.len() == 1 => s.to_uppercase(),
        s => s.to_string(),
    }
}
