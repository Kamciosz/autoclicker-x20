//! PL: Testy automatu z jawnym zegarem i potwierdzaniem myszy.
//! EN: State-machine tests with an explicit clock and mouse acknowledgements.
use super::*;
fn time(milliseconds: u64) -> Duration {
    Duration::from_millis(milliseconds)
}
fn started(config: SessionConfig) -> Session {
    let mut session = Session::default();
    session.command(EngineCommand::Start(config), time(0), true);
    session
}
#[test]
fn defaults_and_validation() {
    assert_eq!(SessionConfig::default().interval, time(100));
    for config in [
        SessionConfig {
            interval: time(9),
            ..Default::default()
        },
        SessionConfig {
            limit: LimitMode::Clicks(0),
            ..Default::default()
        },
        SessionConfig {
            limit: LimitMode::Duration(time(0)),
            ..Default::default()
        },
    ] {
        assert!(config.validate().is_err());
    }
}
#[test]
fn countdown_and_duplicate_start_cannot_accelerate() {
    let mut session = started(SessionConfig::default());
    session.command(
        EngineCommand::Start(SessionConfig::default()),
        time(900),
        true,
    );
    assert_eq!(session.tick(time(2999)), None);
    assert_eq!(session.tick(time(3000)), Some(1));
    assert_eq!(session.cycles, 0);
}
#[test]
fn cycles_complete_and_restart_resets() {
    let mut session = started(SessionConfig {
        limit: LimitMode::Clicks(1),
        ..Default::default()
    });
    assert_eq!(session.tick(time(3000)), Some(1));
    session.clicked(time(3000), 1);
    assert_eq!(session.status, SessionStatus::Completed);
    assert_eq!(session.cycles, 1);
    session.command(
        EngineCommand::Start(SessionConfig::default()),
        time(5000),
        true,
    );
    assert_eq!(session.cycles, 0);
    assert_eq!(session.elapsed(time(5000)), time(0));
}
#[test]
fn pause_waits_for_explicit_resume_and_excludes_countdown() {
    let mut session = started(SessionConfig::default());
    session.tick(time(3000));
    session.clicked(time(3000), 1);
    session.command(EngineCommand::TogglePause, time(3050), true);
    assert_eq!(session.tick(time(9000)), None);
    assert_eq!(session.status, SessionStatus::Paused);
    assert_eq!(session.elapsed(time(9000)), time(50));
    session.command(EngineCommand::TogglePause, time(9000), true);
    assert_eq!(session.tick(time(11999)), None);
    assert_eq!(session.tick(time(12000)), Some(1));
    assert_eq!(session.elapsed(time(12000)), time(50));
}
#[test]
fn stop_cancels_countdown_wait_and_second_click() {
    for stop_time in [time(500), time(3010), time(4000)] {
        let mut session = started(SessionConfig {
            kind: ClickKind::Double,
            ..Default::default()
        });
        if stop_time >= time(3000) {
            session.tick(time(3000));
            session.clicked(time(3000), 1);
        }
        session.command(EngineCommand::Stop, stop_time, true);
        assert_eq!(session.tick(time(10000)), None);
        assert_eq!(session.status, SessionStatus::Idle);
        assert_eq!(session.cycles, 0);
    }
}
#[test]
fn double_click_counts_one_cycle_and_waits_after_completion() {
    let mut session = started(SessionConfig {
        kind: ClickKind::Double,
        ..Default::default()
    });
    assert_eq!(session.tick(time(3000)), Some(1));
    session.clicked(time(3005), 1);
    assert_eq!(session.tick(time(3044)), None);
    assert_eq!(session.tick(time(3045)), Some(2));
    session.clicked(time(3050), 2);
    assert_eq!(session.cycles, 1);
    assert_eq!(session.tick(time(3149)), None);
    assert_eq!(session.tick(time(3150)), Some(1));
}
#[test]
fn time_limit_interrupts_long_wait_and_unlimited_does_not_complete() {
    let mut session = started(SessionConfig {
        interval: time(60000),
        limit: LimitMode::Duration(time(20)),
        ..Default::default()
    });
    session.tick(time(3000));
    session.clicked(time(3000), 1);
    assert_eq!(session.tick(time(3020)), None);
    assert_eq!(session.status, SessionStatus::Completed);
    let mut unlimited = started(SessionConfig {
        limit: LimitMode::Unlimited,
        ..Default::default()
    });
    assert_eq!(unlimited.tick(time(3000)), Some(1));
    unlimited.clicked(time(3000), 1);
    assert_eq!(unlimited.tick(time(999999)), Some(1));
    unlimited.clicked(time(999999), 1);
    assert_eq!(unlimited.tick(time(999999)), None);
}
#[test]
fn permission_failure_and_click_error_never_restart() {
    let mut session = Session::default();
    session.command(
        EngineCommand::Start(SessionConfig::default()),
        time(0),
        false,
    );
    assert_eq!(session.status, SessionStatus::Failed);
    assert_eq!(session.tick(time(3000)), None);
    session.command(
        EngineCommand::Start(SessionConfig::default()),
        time(4000),
        true,
    );
    session.tick(time(7000));
    session.fail("mouse error".into());
    assert_eq!(session.tick(time(8000)), None);
    assert!(matches!(session.events.last(), Some(EngineEvent::Error(_))));
}
#[test]
fn toggled_start_stops_and_countdown_can_be_paused() {
    let mut session = started(SessionConfig::default());
    session.command(EngineCommand::TogglePause, time(1000), true);
    assert_eq!(session.status, SessionStatus::Paused);
    session.command(EngineCommand::TogglePause, time(5000), true);
    assert_eq!(session.tick(time(7999)), None);
    session.command(
        EngineCommand::ToggleStart(SessionConfig::default()),
        time(8000),
        true,
    );
    assert_eq!(session.tick(time(9000)), None);
    assert_eq!(session.status, SessionStatus::Idle);
}
