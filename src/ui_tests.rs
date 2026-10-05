//! PL: Testy produkcyjnego widoku: dane, blokady, walidacja i klawiatura.
//! EN: Production-view tests: data, disabled state, validation, and keyboard.

use super::MainView;
use crate::engine::{self, EngineCommand, EngineEvent, SessionConfig, SessionStatus};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, TestAppContext, WindowOptions};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[gpui_kit::test]
fn controls_preserve_values_validate_and_lock(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let config = Arc::new(Mutex::new(SessionConfig::default()));
    let (handle, commands) = engine::test_handle();
    let (window, view) = cx.update(|cx| {
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| MainView::new(handle, Arc::clone(&config), window, cx))
        })
        .unwrap()
    });
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("interval-up", cx);
        window.render_frame(cx);
        assert_eq!(config.lock().unwrap().interval, Duration::from_millis(110));
        for _ in 0..20 {
            window.click("interval-down", cx);
        }
        assert_eq!(config.lock().unwrap().interval, Duration::from_millis(10));
        window.click("start", cx);
        assert!(matches!(
            commands.try_recv().unwrap(),
            EngineCommand::Start(_)
        ));
        view.update(cx, |view, cx| {
            view.apply_event(EngineEvent::Status(SessionStatus::Running), cx)
        });
        window.render_frame(cx);
        assert_eq!(window.find("interval-up").focused(), None);
        window.click("interval-up", cx);
        assert_eq!(config.lock().unwrap().interval, Duration::from_millis(10));
        window.click("pause", cx);
        assert_eq!(commands.try_recv().unwrap(), EngineCommand::Pause);
        window.press("escape", cx);
        assert_eq!(commands.try_recv().unwrap(), EngineCommand::Stop);
        view.update(cx, |view, cx| {
            view.apply_event(EngineEvent::Status(SessionStatus::Idle), cx)
        });
        window.render_frame(cx);
        window.click("pause", cx);
        assert!(commands.try_recv().is_err());
    })
    .unwrap();
}

#[gpui_kit::test]
fn keyboard_activation_and_start_gate(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (handle, commands) = engine::test_handle();
    let config = Arc::new(Mutex::new(SessionConfig::default()));
    let ready = Arc::clone(&handle.commands.ready);
    let (window, _) = cx.update(|cx| {
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| MainView::new(handle, Arc::clone(&config), window, cx))
        })
        .unwrap()
    });
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.press("tab", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("interval-down").focused(), Some(true));
        window.press("enter", cx);
        window.render_frame(cx);
        assert_eq!(config.lock().unwrap().interval, Duration::from_millis(90));
        ready.store(false, std::sync::atomic::Ordering::SeqCst);
        window.render_frame(cx);
        assert_eq!(window.find("start").focused(), None);
        window.click("start", cx);
        assert!(commands.try_recv().is_err());
    })
    .unwrap();
}
