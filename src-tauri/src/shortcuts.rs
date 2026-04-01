use parking_lot::RwLock;
use rdev::{Button, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

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
    SingleKey(Key),
    MouseButton(u8),
    Combo(Vec<Key>),
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

fn key_code_from_name(name: &str) -> Option<Key> {
    match name {
        "Alt_L" => Some(Key::Alt),
        "Alt_R" => Some(Key::AltGr),
        "Shift_L" => Some(Key::ShiftLeft),
        "Shift_R" => Some(Key::ShiftRight),
        "Control_L" => Some(Key::ControlLeft),
        "Control_R" => Some(Key::ControlRight),
        "Super_L" => Some(Key::MetaLeft),
        "Super_R" => Some(Key::MetaRight),
        "space" => Some(Key::Space),
        "Return" => Some(Key::Return),
        "Escape" => Some(Key::Escape),
        "BackSpace" => Some(Key::Backspace),
        "Tab" => Some(Key::Tab),
        "Caps_Lock" => Some(Key::CapsLock),
        "Delete" => Some(Key::Delete),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "a" => Some(Key::KeyA),
        "b" => Some(Key::KeyB),
        "c" => Some(Key::KeyC),
        "d" => Some(Key::KeyD),
        "e" => Some(Key::KeyE),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "i" => Some(Key::KeyI),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "m" => Some(Key::KeyM),
        "n" => Some(Key::KeyN),
        "o" => Some(Key::KeyO),
        "p" => Some(Key::KeyP),
        "q" => Some(Key::KeyQ),
        "r" => Some(Key::KeyR),
        "s" => Some(Key::KeyS),
        "t" => Some(Key::KeyT),
        "u" => Some(Key::KeyU),
        "v" => Some(Key::KeyV),
        "w" => Some(Key::KeyW),
        "x" => Some(Key::KeyX),
        "y" => Some(Key::KeyY),
        "z" => Some(Key::KeyZ),
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        _ => None,
    }
}

fn is_modifier(code: &Key) -> bool {
    matches!(
        code,
        Key::Alt
            | Key::AltGr
            | Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::MetaLeft
            | Key::MetaRight
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
        let keys: Vec<Key> = combo_str
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

fn keys_match_combo(pressed: &HashSet<Key>, combo: &[Key]) -> bool {
    combo.iter().all(|k| pressed.contains(k))
}

pub type GrabHandle = GrabHandleInner;

struct GrabState {
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
    pressed: HashSet<Key>,
    combo_hold_active: bool,
    combo_toggle_active: bool,
    combo_paste_active: bool,
    paste_pending: bool,
}

impl GrabState {
    fn process_key_press(&mut self, config: &ShortcutConfig, key: Key) -> bool {
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

    fn process_key_release(&mut self, config: &ShortcutConfig, key: Key) -> bool {
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

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let state = Arc::new(parking_lot::Mutex::new(GrabState {
            handle,
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
            let handle = &guard.handle;

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
                    guard.pressed.remove(&key);
                }
                return Some(event);
            }

            let config = handle.config.read().clone();

            match event.event_type {
                EventType::KeyPress(key) => {
                    if !guard.pressed.insert(key) {
                        return Some(event);
                    }
                    let suppress = guard.process_key_press(&config, key);
                    if suppress { None } else { Some(event) }
                }
                EventType::KeyRelease(key) => {
                    guard.pressed.remove(&key);
                    let suppress = guard.process_key_release(&config, key);
                    if suppress { None } else { Some(event) }
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

/// macOS implementation using CGEvent for mouse buttons and rdev for keyboard
#[cfg(target_os = "macos")]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField};
    
    std::thread::spawn(move || {
        let state = Arc::new(parking_lot::Mutex::new(GrabState {
            handle: handle.clone(),
            event_tx: event_tx.clone(),
            pressed: HashSet::new(),
            combo_hold_active: false,
            combo_toggle_active: false,
            combo_paste_active: false,
            paste_pending: false,
        }));

        // Start rdev in a separate thread for keyboard events
        let rdev_state = Arc::clone(&state);
        let rdev_handle = handle.clone();
        std::thread::spawn(move || {
            rdev::set_is_main_thread(false);
            
            let callback = move |event: Event| -> Option<Event> {
                // Only process keyboard events with rdev
                match event.event_type {
                    EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                        let mut guard = rdev_state.lock();
                        let config = rdev_handle.config.read().clone();
                        
                        match event.event_type {
                            EventType::KeyPress(key) => {
                                if !guard.pressed.insert(key) {
                                    return Some(event);
                                }
                                let suppress = guard.process_key_press(&config, key);
                                if suppress { None } else { Some(event) }
                            }
                            EventType::KeyRelease(key) => {
                                guard.pressed.remove(&key);
                                let suppress = guard.process_key_release(&config, key);
                                if suppress { None } else { Some(event) }
                            }
                            _ => Some(event)
                        }
                    }
                    _ => Some(event)
                }
            };
            
            if let Err(e) = rdev::grab(callback) {
                log::error!("[openbolo] rdev::grab failed: {:?}", e);
            }
        });

        // CGEvent tap for mouse buttons
        let event_types = vec![
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
            CGEventType::RightMouseDown,
            CGEventType::RightMouseUp,
            CGEventType::OtherMouseDown,
            CGEventType::OtherMouseUp,
        ];

        let cg_state = Arc::clone(&state);
        let cg_handle = handle.clone();
        
        let tap_callback = move |_proxy, event_type: CGEventType, event: &CGEvent| -> Option<CGEvent> {
            let mut guard = cg_state.lock();
            let button_num = event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
            let btn_code = button_num as u8;
            
            if cg_handle.capture_mode.load(Ordering::SeqCst) {
                if matches!(event_type, CGEventType::LeftMouseDown | CGEventType::RightMouseDown | CGEventType::OtherMouseDown) {
                    if btn_code != 0 {
                        let name = button_to_name(btn_code);
                        if let Some(tx) = cg_handle.capture_tx.lock().take() {
                            tx.send(name).ok();
                        }
                        return None;
                    }
                }
                return Some(event.to_owned());
            }
            
            if !cg_handle.enabled.load(Ordering::SeqCst) {
                return Some(event.to_owned());
            }
            
            let config = cg_handle.config.read().clone();
            
            let suppress = match event_type {
                CGEventType::LeftMouseDown | CGEventType::RightMouseDown | CGEventType::OtherMouseDown => {
                    guard.process_button_press(&config, btn_code)
                }
                CGEventType::LeftMouseUp | CGEventType::RightMouseUp | CGEventType::OtherMouseUp => {
                    guard.process_button_release(&config, btn_code)
                }
                _ => false
            };
            
            if suppress { None } else { Some(event.to_owned()) }
        };

        match CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            event_types,
            tap_callback,
        ) {
            Ok(tap) => {
                unsafe {
                    let run_loop_source = tap.mach_port.create_runloop_source(0).unwrap();
                    let run_loop = core_foundation::runloop::CFRunLoop::get_current();
                    run_loop.add_source(&run_loop_source, core_foundation::runloop::kCFRunLoopCommonModes);
                    tap.enable();
                    core_foundation::runloop::CFRunLoop::run_current();
                }
            }
            Err(e) => {
                log::error!("[openbolo] Failed to create CGEvent tap: {:?}", e);
                eprintln!("[openbolo] Failed to create CGEvent tap. Make sure Accessibility permissions are granted.");
            }
        }
    })
}

/// Windows does not support event suppression via rdev::grab; fall back to rdev::listen.
#[cfg(target_os = "windows")]
pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let state = Arc::new(parking_lot::Mutex::new(GrabState {
            handle,
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
            let handle = &guard.handle;

            // During capture, ignore left click so UI clicks are not bound as the shortcut.
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
                    guard.pressed.remove(&key);
                }
                return;
            }

            let config = handle.config.read().clone();

            match event.event_type {
                EventType::KeyPress(key) => {
                    if guard.pressed.insert(key) {
                        guard.process_key_press(&config, key);
                    }
                }
                EventType::KeyRelease(key) => {
                    guard.pressed.remove(&key);
                    guard.process_key_release(&config, key);
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
        s if s.len() == 1 => s.to_uppercase(),
        s => s.to_string(),
    }
}
