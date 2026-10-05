//! PL: Kontrakty sesji i pojedynczy worker; nigdy nie wysyła kliknięć z UI.
//! EN: Session contracts and a single worker; never emits clicks from the UI.
//! @uses src/session.rs::Session
//! @used_by src/ui.rs::MainView

use crate::session::Session;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickKind {
    Single,
    Double,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitMode {
    Clicks(u64),
    Duration(Duration),
    Unlimited,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionConfig {
    pub interval: Duration,
    pub button: MouseButton,
    pub kind: ClickKind,
    pub limit: LimitMode,
}
impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            button: MouseButton::Left,
            kind: ClickKind::Single,
            limit: LimitMode::Clicks(100),
        }
    }
}
impl SessionConfig {
    /// PL: Sprawdza minimalny interwał i dodatni limit; nie zmienia danych.
    /// EN: Validates the minimum interval and a positive limit without mutation.
    pub fn validate(self) -> Result<Self, String> {
        if self.interval < Duration::from_millis(10) {
            return Err("Interwał musi wynosić co najmniej 10 ms".into());
        }
        match self.limit {
            LimitMode::Clicks(0) => Err("Liczba cykli musi być większa od zera".into()),
            LimitMode::Duration(value) if value.is_zero() => {
                Err("Czas sesji musi być większy od zera".into())
            }
            _ => Ok(self),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Countdown(u8),
    Running,
    Paused,
    Completed,
    Failed,
}
impl SessionStatus {
    pub fn active(self) -> bool {
        matches!(self, Self::Countdown(_) | Self::Running | Self::Paused)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineEvent {
    Status(SessionStatus),
    Progress { clicks: u64, elapsed: Duration },
    Error(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineCommand {
    Start(SessionConfig),
    ToggleStart(SessionConfig),
    TogglePause,
    Pause,
    Resume,
    Stop,
    Shutdown,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MousePosition {
    pub x: f64,
    pub y: f64,
}
/// PL: Każde wywołanie click wysyła kompletną parę down/up w podanej pozycji.
/// EN: Every click call sends a complete down/up pair at the supplied position.
pub trait ClickExecutor: Send + Sync + 'static {
    fn is_accessibility_trusted(&self) -> bool;
    fn position(&self) -> Result<MousePosition, String>;
    fn click(
        &self,
        button: MouseButton,
        position: MousePosition,
        click_state: i64,
    ) -> Result<(), String>;
}
#[derive(Clone)]
pub struct EngineCommands {
    sender: Sender<EngineCommand>,
    interrupted: Arc<AtomicBool>,
    pub ready: Arc<AtomicBool>,
    worker: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}
impl EngineCommands {
    /// PL: Zamknięcie czeka na parę down/up przed wyjściem procesu.
    /// EN: Closing waits for the down/up pair before the process exits.
    pub fn shutdown_and_wait(&self) {
        let _ = self.send(EngineCommand::Shutdown);
        let worker = self.worker.lock().ok().and_then(|mut worker| worker.take());
        if let Some(worker) = worker {
            let _ = worker.join();
        }
    }
    /// PL: Stop i pauza blokują kolejne zdarzenie również przed odczytem kanału.
    /// EN: Stop and pause inhibit the next event even before the channel is read.
    pub fn send(&self, command: EngineCommand) -> Result<(), mpsc::SendError<EngineCommand>> {
        if matches!(
            command,
            EngineCommand::Stop
                | EngineCommand::Shutdown
                | EngineCommand::TogglePause
                | EngineCommand::Pause
                | EngineCommand::ToggleStart(_)
        ) {
            self.interrupted.store(true, Ordering::SeqCst);
        }
        self.sender.send(command)
    }
}
pub struct EngineHandle {
    pub commands: EngineCommands,
    pub events: Receiver<EngineEvent>,
}
pub struct SessionEngine;
impl SessionEngine {
    /// PL: Tworzy jednego workera; ponowny Start nie tworzy następnej sesji.
    /// EN: Creates one worker; repeated Start never creates another session.
    pub fn spawn(executor: Arc<dyn ClickExecutor>) -> EngineHandle {
        let (sender, receiver) = mpsc::channel();
        let (event_sender, events) = mpsc::channel();
        let commands = EngineCommands {
            sender,
            interrupted: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            worker: Arc::new(Mutex::new(None)),
        };
        let interrupted = Arc::clone(&commands.interrupted);
        let ready = Arc::clone(&commands.ready);
        let worker = thread::Builder::new()
            .name("autoclicker-session".into())
            .spawn(move || worker(receiver, event_sender, executor, interrupted, ready))
            .expect("failed to spawn session worker");
        *commands.worker.lock().expect("worker lifetime lock") = Some(worker);
        EngineHandle { commands, events }
    }
}
fn worker(
    commands: Receiver<EngineCommand>,
    events: Sender<EngineEvent>,
    executor: Arc<dyn ClickExecutor>,
    interrupted: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
) {
    let epoch = Instant::now();
    let mut session = Session::default();
    let mut position = None;
    let mut next_progress = Duration::ZERO;
    loop {
        interrupted.store(false, Ordering::SeqCst);
        match commands.recv_timeout(Duration::from_millis(10)) {
            Ok(EngineCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(command) => {
                let can_start = ready.load(Ordering::SeqCst) && executor.is_accessibility_trusted();
                session.command(command, epoch.elapsed(), can_start);
                while let Ok(command) = commands.try_recv() {
                    if command == EngineCommand::Shutdown {
                        return;
                    }
                    let can_start =
                        ready.load(Ordering::SeqCst) && executor.is_accessibility_trusted();
                    session.command(command, epoch.elapsed(), can_start);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if session.status.active() && !executor.is_accessibility_trusted() {
            session.fail("Uprawnienie Dostępność zostało odebrane".into());
        }
        let now = epoch.elapsed();
        if !interrupted.load(Ordering::SeqCst)
            && let Some(click_state) = session.tick(now)
        {
            let result = if click_state == 1 {
                executor.position().inspect(|value| position = Some(*value))
            } else {
                position.ok_or_else(|| "Brak pozycji dwukliku".into())
            }
            .and_then(|position| {
                if interrupted.load(Ordering::SeqCst) {
                    return Ok(false);
                }
                executor
                    .click(session.config.button, position, click_state)
                    .map(|()| true)
            });
            match result {
                Ok(true) => session.clicked(epoch.elapsed(), click_state),
                Ok(false) => {}
                Err(error) => session.fail(error),
            }
        }
        for event in session.events.drain(..) {
            if events.send(event).is_err() {
                return;
            }
        }
        if now >= next_progress && session.status == SessionStatus::Running {
            let _ = events.send(EngineEvent::Progress {
                clicks: session.cycles,
                elapsed: session.elapsed(now),
            });
            next_progress = now + Duration::from_millis(100);
        }
    }
}

#[cfg(test)]
pub fn test_handle() -> (EngineHandle, Receiver<EngineCommand>) {
    let (sender, commands) = mpsc::channel();
    let (_, events) = mpsc::channel();
    (
        EngineHandle {
            commands: EngineCommands {
                sender,
                interrupted: Arc::new(AtomicBool::new(false)),
                ready: Arc::new(AtomicBool::new(true)),
                worker: Arc::new(Mutex::new(None)),
            },
            events,
        },
        commands,
    )
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
