//! PL: Testy workera z atrapą myszy; nie wysyłają zdarzeń macOS.
//! EN: Worker tests with a fake mouse; never send macOS events.

use super::*;
use std::sync::{Mutex, atomic::AtomicUsize};

struct FakeMouse {
    trusted: AtomicBool,
    calls: Mutex<Vec<(MousePosition, i64)>>,
    concurrent: AtomicUsize,
    fail: bool,
}
impl ClickExecutor for FakeMouse {
    fn is_accessibility_trusted(&self) -> bool {
        self.trusted.load(Ordering::SeqCst)
    }
    fn position(&self) -> Result<MousePosition, String> {
        Ok(MousePosition { x: 50.0, y: 60.0 })
    }
    fn click(
        &self,
        _: MouseButton,
        position: MousePosition,
        click_state: i64,
    ) -> Result<(), String> {
        assert_eq!(self.concurrent.fetch_add(1, Ordering::SeqCst), 0);
        self.calls.lock().unwrap().push((position, click_state));
        self.concurrent.fetch_sub(1, Ordering::SeqCst);
        if self.fail {
            Err("fake mouse failed".into())
        } else {
            Ok(())
        }
    }
}
fn fake(trusted: bool, fail: bool) -> Arc<FakeMouse> {
    Arc::new(FakeMouse {
        trusted: AtomicBool::new(trusted),
        calls: Mutex::new(Vec::new()),
        concurrent: AtomicUsize::new(0),
        fail,
    })
}
fn receive_until(handle: &EngineHandle, status: SessionStatus) {
    loop {
        let event = handle.events.recv_timeout(Duration::from_secs(5)).unwrap();
        if event == EngineEvent::Status(status) {
            break;
        }
    }
}
#[test]
fn worker_double_click_uses_one_position_and_duplicate_start_is_ignored() {
    let mouse = fake(true, false);
    let handle = SessionEngine::spawn(mouse.clone());
    handle.commands.ready.store(true, Ordering::SeqCst);
    let config = SessionConfig {
        kind: ClickKind::Double,
        limit: LimitMode::Clicks(1),
        ..Default::default()
    };
    handle.commands.send(EngineCommand::Start(config)).unwrap();
    receive_until(&handle, SessionStatus::Countdown(3));
    handle.commands.send(EngineCommand::Start(config)).unwrap();
    receive_until(&handle, SessionStatus::Completed);
    let calls = mouse.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, calls[1].0);
    assert_eq!((calls[0].1, calls[1].1), (1, 2));
    handle.commands.send(EngineCommand::Shutdown).unwrap();
}
#[test]
fn worker_denied_permission_and_click_failure_are_reported() {
    for (trusted, fail) in [(false, false), (true, true)] {
        let mouse = fake(trusted, fail);
        let handle = SessionEngine::spawn(mouse.clone());
        handle.commands.ready.store(true, Ordering::SeqCst);
        handle
            .commands
            .send(EngineCommand::Start(SessionConfig::default()))
            .unwrap();
        receive_until(&handle, SessionStatus::Failed);
        assert!(matches!(
            handle.events.recv_timeout(Duration::from_secs(1)).unwrap(),
            EngineEvent::Error(_)
        ));
        assert_eq!(mouse.calls.lock().unwrap().len(), usize::from(trusted));
        handle.commands.send(EngineCommand::Shutdown).unwrap();
    }
}
#[test]
fn worker_stop_and_permission_loss_cancel_countdown() {
    for permission_loss in [false, true] {
        let mouse = fake(true, false);
        let handle = SessionEngine::spawn(mouse.clone());
        handle.commands.ready.store(true, Ordering::SeqCst);
        handle
            .commands
            .send(EngineCommand::Start(SessionConfig::default()))
            .unwrap();
        receive_until(&handle, SessionStatus::Countdown(3));
        if permission_loss {
            mouse.trusted.store(false, Ordering::SeqCst);
            receive_until(&handle, SessionStatus::Failed);
        } else {
            handle.commands.send(EngineCommand::Stop).unwrap();
            receive_until(&handle, SessionStatus::Idle);
        }
        assert!(mouse.calls.lock().unwrap().is_empty());
        handle.commands.send(EngineCommand::Shutdown).unwrap();
    }
}

#[test]
fn explicit_pause_and_resume_commands_are_idempotent() {
    let mouse = fake(true, false);
    let handle = SessionEngine::spawn(mouse.clone());
    handle.commands.ready.store(true, Ordering::SeqCst);
    handle
        .commands
        .send(EngineCommand::Start(SessionConfig::default()))
        .unwrap();
    receive_until(&handle, SessionStatus::Countdown(3));
    handle.commands.send(EngineCommand::Pause).unwrap();
    receive_until(&handle, SessionStatus::Paused);
    handle.commands.send(EngineCommand::Pause).unwrap();
    handle.commands.send(EngineCommand::Resume).unwrap();
    receive_until(&handle, SessionStatus::Countdown(3));
    handle.commands.shutdown_and_wait();
    assert!(mouse.calls.lock().unwrap().is_empty());
}

#[test]
fn shutdown_waits_for_inflight_mouse_pair() {
    struct HeldPair {
        entered: Sender<()>,
        released: Mutex<Receiver<()>>,
    }
    impl ClickExecutor for HeldPair {
        fn is_accessibility_trusted(&self) -> bool {
            true
        }
        fn position(&self) -> Result<MousePosition, String> {
            Ok(MousePosition { x: 0.0, y: 0.0 })
        }
        fn click(&self, _: MouseButton, _: MousePosition, _: i64) -> Result<(), String> {
            self.entered.send(()).unwrap();
            self.released.lock().unwrap().recv().unwrap();
            Ok(())
        }
    }
    let (entered_sender, entered) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let handle = SessionEngine::spawn(Arc::new(HeldPair {
        entered: entered_sender,
        released: Mutex::new(released),
    }));
    handle.commands.ready.store(true, Ordering::SeqCst);
    handle
        .commands
        .send(EngineCommand::Start(SessionConfig::default()))
        .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let (closing_sender, closing) = mpsc::channel();
    let (closed_sender, closed) = mpsc::channel();
    thread::spawn(move || {
        handle.commands.send(EngineCommand::Shutdown).unwrap();
        closing_sender.send(()).unwrap();
        handle.commands.shutdown_and_wait();
        closed_sender.send(()).unwrap();
    });
    closing.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(
        closed.recv_timeout(Duration::from_millis(20)),
        Err(mpsc::RecvTimeoutError::Timeout)
    );
    release.send(()).unwrap();
    closed.recv_timeout(Duration::from_secs(1)).unwrap();
}
