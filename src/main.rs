//! PL: Punkt wejścia aplikacji, składanie natywnych adapterów i widoku GPUI.
//! EN: Application entry point that composes native adapters and the GPUI view.
//! @uses src/engine.rs::SessionEngine
//! @uses src/macos.rs::HotkeyRegistration
//! @uses src/ui.rs::MainView

mod engine;
mod macos;
mod session;
mod ui;

use std::sync::{Arc, Mutex};

use engine::{SessionConfig, SessionEngine};
use gpui_kit::{
    AppContext, Bounds, Global, TitlebarOptions, WindowBounds, WindowOptions, application, assets,
    init, open_window, px, size,
};
use macos::{HotkeyRegistration, MacClickExecutor, SleepObserver};
use ui::MainView;

struct NativeLifetime {
    _hotkeys: Option<HotkeyRegistration>,
    _sleep: SleepObserver,
}
impl Global for NativeLifetime {}

fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);
        gpui_kit::component::Theme::sync_system_appearance(None, cx);
        let executor = Arc::new(MacClickExecutor);
        let handle = SessionEngine::spawn(executor);
        let config = Arc::new(Mutex::new(SessionConfig::default()));
        let hotkeys = HotkeyRegistration::register(handle.commands.clone(), Arc::clone(&config));
        let startup_message = hotkeys.as_ref().err().cloned();
        handle
            .commands
            .ready
            .store(hotkeys.is_ok(), std::sync::atomic::Ordering::SeqCst);
        let commands = handle.commands.clone();
        cx.set_global(NativeLifetime {
            _hotkeys: hotkeys.ok(),
            _sleep: SleepObserver::register(commands.clone()),
        });
        cx.on_app_quit({
            let commands = commands.clone();
            move |_| {
                commands.shutdown_and_wait();
                async {}
            }
        })
        .detach();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(640.), px(540.)),
                cx,
            ))),
            window_min_size: Some(size(px(600.), px(500.))),
            titlebar: Some(TitlebarOptions {
                title: Some("Autoclicker X20".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (_, view) = open_window(options, cx, |window, cx| {
            window
                .observe_window_appearance(|window, cx| {
                    gpui_kit::component::Theme::sync_system_appearance(Some(window), cx);
                })
                .detach();
            window.on_window_should_close(cx, move |_, cx| {
                commands.shutdown_and_wait();
                cx.quit();
                true
            });
            cx.new(|cx| MainView::new(handle, config, window, cx))
        })
        .expect("failed to open Autoclicker X20 window");
        if let Some(error) = startup_message {
            view.update(cx, |view, cx| {
                view.apply_event(
                    engine::EngineEvent::Error(format!("Skróty niedostępne: {error}")),
                    cx,
                );
            });
        }
        let _ = view;
    });
}
