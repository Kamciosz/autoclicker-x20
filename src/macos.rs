//! PL: Natywna granica kliknięć, zgody, skrótów i uśpienia macOS.
//! EN: Native boundary for clicks, consent, hotkeys, and macOS sleep.
//! @uses src/engine.rs::ClickExecutor
//! @used_by src/main.rs::main

use crate::engine::{
    ClickExecutor, EngineCommand, EngineCommands, MouseButton, MousePosition, SessionConfig,
};
use block2::RcBlock;
use core_graphics::{
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, EventField},
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSWorkspace, NSWorkspaceWillSleepNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObjectProtocol};
use std::{
    ptr::NonNull,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct MacClickExecutor;
impl ClickExecutor for MacClickExecutor {
    fn is_accessibility_trusted(&self) -> bool {
        unsafe { CGPreflightPostEventAccess() }
    }
    fn position(&self) -> Result<MousePosition, String> {
        let source = source()?;
        let event = CGEvent::new(source)
            .map_err(|_| "Nie udało się odczytać pozycji kursora".to_string())?;
        let location = event.location();
        Ok(MousePosition {
            x: location.x,
            y: location.y,
        })
    }
    fn click(
        &self,
        button: MouseButton,
        position: MousePosition,
        click_state: i64,
    ) -> Result<(), String> {
        if !self.is_accessibility_trusted() {
            return Err("Włącz uprawnienie Dostępność dla tej aplikacji".into());
        }
        let source = source()?;
        let location = CGPoint::new(position.x, position.y);
        let (button, down, up) = match button {
            MouseButton::Left => (
                CGMouseButton::Left,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
            ),
            MouseButton::Right => (
                CGMouseButton::Right,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
            ),
        };
        // PL: Oba zdarzenia powstają przed down; błąd alokacji nie zostawia wciśniętej myszy.
        // EN: Allocate both events before down; allocation failure cannot leave the mouse held.
        let down = mouse_event(&source, down, location, button, click_state)?;
        let up = mouse_event(&source, up, location, button, click_state)?;
        down.post(CGEventTapLocation::Session);
        up.post(CGEventTapLocation::Session);
        Ok(())
    }
}
fn source() -> Result<CGEventSource, String> {
    CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| "Nie udało się utworzyć źródła zdarzeń myszy".to_string())
}
fn mouse_event(
    source: &CGEventSource,
    event_type: CGEventType,
    location: CGPoint,
    button: CGMouseButton,
    click_state: i64,
) -> Result<CGEvent, String> {
    let event = CGEvent::new_mouse_event(source.clone(), event_type, location, button)
        .map_err(|_| "Nie udało się utworzyć zdarzenia myszy".to_string())?;
    event.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, click_state);
    Ok(event)
}
/// PL: Wywoływane tylko z przycisku użytkownika; otwiera systemową prośbę o zgodę.
/// EN: Called only from the user's button; opens the system consent request.
pub fn request_access() -> bool {
    unsafe { CGRequestPostEventAccess() }
}

pub struct HotkeyRegistration {
    manager: GlobalHotKeyManager,
    hotkeys: [HotKey; 3],
}
impl HotkeyRegistration {
    /// PL: Rejestruje na głównym wątku; błąd wycofuje częściową rejestrację.
    /// EN: Registers on the main thread; failure rolls back partial registration.
    pub fn register(
        commands: EngineCommands,
        config: Arc<Mutex<SessionConfig>>,
    ) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|error| error.to_string())?;
        let hotkeys = [
            HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyS),
            HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyP),
            HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyX),
        ];
        for (index, hotkey) in hotkeys.iter().enumerate() {
            if let Err(error) = manager.register(*hotkey) {
                let _ = manager.unregister_all(&hotkeys[..index]);
                return Err(error.to_string());
            }
        }
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state != HotKeyState::Pressed {
                return;
            }
            let command = match event.id {
                id if id == hotkeys[0].id() => {
                    let Ok(config) = config.lock() else {
                        return;
                    };
                    EngineCommand::ToggleStart(*config)
                }
                id if id == hotkeys[1].id() => EngineCommand::TogglePause,
                id if id == hotkeys[2].id() => EngineCommand::Stop,
                _ => return,
            };
            let _ = commands.send(command);
        }));
        Ok(Self { manager, hotkeys })
    }
}
impl Drop for HotkeyRegistration {
    fn drop(&mut self) {
        let _ = self.manager.unregister_all(&self.hotkeys);
    }
}

pub struct SleepObserver {
    center: Retained<NSNotificationCenter>,
    observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}
impl SleepObserver {
    /// PL: WillSleep kończy sesję; nie rejestrujemy wznowienia po wybudzeniu.
    /// EN: WillSleep ends the session; no automatic wake-resume is registered.
    pub fn register(commands: EngineCommands) -> Self {
        let center = NSWorkspace::sharedWorkspace().notificationCenter();
        let block = RcBlock::new(move |_: NonNull<NSNotification>| {
            let _ = commands.send(EngineCommand::Stop);
        });
        // PL: Blok przechwytuje wyłącznie sendowalny kanał; token pozostaje przy życiu.
        // EN: The block captures only a sendable channel; its token remains alive.
        let observer = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceWillSleepNotification),
                None,
                None,
                &block,
            )
        };
        Self { center, observer }
    }
}
impl Drop for SleepObserver {
    fn drop(&mut self) {
        unsafe {
            self.center.removeObserver((*self.observer).as_ref());
        }
    }
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}
