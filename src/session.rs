//! PL: Deterministyczny automat; czas monotoniczny dostarcza worker albo test.
//! EN: Deterministic state machine; monotonic time comes from the worker or test.
//! @uses src/engine.rs::EngineCommand
//! @used_by src/engine.rs::SessionEngine

use crate::engine::{
    ClickKind, EngineCommand, EngineEvent, LimitMode, SessionConfig, SessionStatus,
};
use std::time::Duration;

pub struct Session {
    pub config: SessionConfig,
    pub status: SessionStatus,
    pub cycles: u64,
    pub events: Vec<EngineEvent>,
    active_time: Duration,
    running_since: Duration,
    countdown_end: Duration,
    next_click: Duration,
    second_click: bool,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            config: SessionConfig::default(),
            status: SessionStatus::Idle,
            cycles: 0,
            events: vec![EngineEvent::Status(SessionStatus::Idle)],
            active_time: Duration::ZERO,
            running_since: Duration::ZERO,
            countdown_end: Duration::ZERO,
            next_click: Duration::ZERO,
            second_click: false,
        }
    }
}
impl Session {
    /// PL: Wykonuje polecenie w podanym czasie; nie dotyka myszy ani zegara.
    /// EN: Applies a command at the supplied time; touches neither mouse nor clock.
    pub fn command(&mut self, command: EngineCommand, now: Duration, can_start: bool) {
        match command {
            EngineCommand::ToggleStart(_) if self.status.active() => self.stop(now),
            EngineCommand::Start(config) | EngineCommand::ToggleStart(config)
                if !self.status.active() =>
            {
                if let Err(error) = config.validate() {
                    self.fail(error);
                    return;
                }
                if !can_start {
                    self.fail("Start zablokowany: sprawdź skróty i uprawnienie Dostępność".into());
                    return;
                }
                self.config = config;
                self.cycles = 0;
                self.active_time = Duration::ZERO;
                self.second_click = false;
                self.progress(now);
                self.countdown(now);
            }
            EngineCommand::TogglePause if self.status == SessionStatus::Paused => self.resume(now),
            EngineCommand::TogglePause | EngineCommand::Pause => self.pause(now),
            EngineCommand::Resume => self.resume(now),
            EngineCommand::Stop | EngineCommand::Shutdown => self.stop(now),
            _ => {}
        }
    }
    fn pause(&mut self, now: Duration) {
        if matches!(
            self.status,
            SessionStatus::Running | SessionStatus::Countdown(_)
        ) {
            self.freeze(now);
            self.second_click = false;
            self.set_status(SessionStatus::Paused);
            self.progress(now);
        }
    }
    fn resume(&mut self, now: Duration) {
        if self.status == SessionStatus::Paused {
            self.countdown(now);
        }
    }
    fn countdown(&mut self, now: Duration) {
        self.countdown_end = now + Duration::from_secs(3);
        self.set_status(SessionStatus::Countdown(3));
    }
    fn freeze(&mut self, now: Duration) {
        self.active_time = self.elapsed(now);
    }
    fn stop(&mut self, now: Duration) {
        self.freeze(now);
        self.second_click = false;
        self.set_status(SessionStatus::Idle);
        self.progress(now);
    }
    /// PL: Zwraca co najwyżej jeden click state; bez nadrabiania zaległych kliknięć.
    /// EN: Returns at most one click state; never catches up with click bursts.
    pub fn tick(&mut self, now: Duration) -> Option<i64> {
        if matches!(self.status, SessionStatus::Countdown(_)) {
            if now < self.countdown_end {
                let remaining = (self.countdown_end - now)
                    .as_secs()
                    .saturating_add(1)
                    .min(3) as u8;
                self.set_status(SessionStatus::Countdown(remaining));
                return None;
            }
            self.running_since = now;
            self.next_click = now;
            self.set_status(SessionStatus::Running);
        }
        if self.status != SessionStatus::Running {
            return None;
        }
        if self.limit_reached(now) {
            self.freeze(now);
            self.set_status(SessionStatus::Completed);
            self.progress(now);
            return None;
        }
        (now >= self.next_click).then_some(if self.second_click { 2 } else { 1 })
    }
    /// PL: Potwierdza parę down/up; dwuklik zalicza po drugiej parze.
    /// EN: Acknowledges a down/up pair; double clicks count after the second pair.
    pub fn clicked(&mut self, now: Duration, click_state: i64) {
        if self.config.kind == ClickKind::Double && click_state == 1 {
            self.second_click = true;
            self.next_click = now + Duration::from_millis(40);
            return;
        }
        self.second_click = false;
        self.cycles += 1;
        self.next_click = now + self.config.interval;
        self.progress(now);
        if self.limit_reached(now) {
            self.freeze(now);
            self.set_status(SessionStatus::Completed);
        }
    }
    fn limit_reached(&self, now: Duration) -> bool {
        match self.config.limit {
            LimitMode::Clicks(limit) => self.cycles >= limit,
            LimitMode::Duration(limit) => self.elapsed(now) >= limit,
            LimitMode::Unlimited => false,
        }
    }
    pub fn elapsed(&self, now: Duration) -> Duration {
        self.active_time
            + if self.status == SessionStatus::Running {
                now.saturating_sub(self.running_since)
            } else {
                Duration::ZERO
            }
    }
    pub fn fail(&mut self, error: String) {
        self.second_click = false;
        self.set_status(SessionStatus::Failed);
        self.events.push(EngineEvent::Error(error));
    }
    fn progress(&mut self, now: Duration) {
        self.events.push(EngineEvent::Progress {
            clicks: self.cycles,
            elapsed: self.elapsed(now),
        });
    }
    fn set_status(&mut self, status: SessionStatus) {
        if self.status != status {
            self.status = status;
            self.events.push(EngineEvent::Status(status));
        }
    }
}
#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
