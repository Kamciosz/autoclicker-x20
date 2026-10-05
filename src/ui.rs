//! PL: Jedno okno konfiguracji i statusu sesji kliknięć.
//! EN: Single-window configuration and click-session status view.
//! @uses src/engine.rs::EngineCommand
//! @used_by src/main.rs::main

use std::sync::{Arc, Mutex};

use gpui_kit::{
    component::{
        ActiveTheme, Disableable,
        button::{Button, ButtonVariants},
    },
    prelude::*,
    *,
};

use crate::engine::{
    ClickKind, EngineCommand, EngineCommands, EngineEvent, EngineHandle, LimitMode, MouseButton,
    SessionConfig, SessionStatus,
};

pub struct MainView {
    commands: EngineCommands,
    config: Arc<Mutex<SessionConfig>>,
    status: SessionStatus,
    clicks: u64,
    elapsed: std::time::Duration,
    message: String,
    focus: FocusHandle,
    _event_task: Task<()>,
}

impl MainView {
    pub fn new(
        handle: EngineHandle,
        config: Arc<Mutex<SessionConfig>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let events = Arc::new(Mutex::new(handle.events));
        let task_events = Arc::clone(&events);
        let event_task = cx.spawn(async move |this, cx| {
            loop {
                let receiver = Arc::clone(&task_events);
                let event = cx
                    .background_executor()
                    .spawn(
                        async move { receiver.lock().ok().and_then(|events| events.recv().ok()) },
                    )
                    .await;
                let Some(event) = event else { break };
                if this
                    .update(cx, |view, cx| view.apply_event(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            commands: handle.commands,
            config,
            status: SessionStatus::Idle,
            clicks: 0,
            elapsed: std::time::Duration::ZERO,
            message: "Gotowy".into(),
            focus,
            _event_task: event_task,
        }
    }

    pub fn apply_event(&mut self, event: EngineEvent, cx: &mut Context<Self>) {
        match event {
            EngineEvent::Status(status) => {
                self.status = status;
                self.message = status_message(status).into();
            }
            EngineEvent::Progress { clicks, elapsed } => {
                self.clicks = clicks;
                self.elapsed = elapsed;
            }
            EngineEvent::Error(error) => self.message = error,
        }
        cx.notify();
    }

    fn send(&self, command: EngineCommand) {
        let _ = self.commands.send(command);
    }

    fn update_config(&self, update: impl FnOnce(&mut SessionConfig)) {
        if self.status.active() {
            return;
        }
        if let Ok(mut config) = self.config.lock() {
            update(&mut config);
        }
    }

    fn start(&self) {
        let config = self.config.lock().map(|config| *config).unwrap_or_default();
        self.send(EngineCommand::Start(config));
    }

    fn config(&self) -> SessionConfig {
        self.config.lock().map(|config| *config).unwrap_or_default()
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = self.config();
        let active = matches!(
            self.status,
            SessionStatus::Countdown(_) | SessionStatus::Running | SessionStatus::Paused
        );
        let start_label = if active { "Stop" } else { "Start" };
        let limit_label = match config.limit {
            LimitMode::Clicks(value) => format!("Limit cykli: {value}"),
            LimitMode::Duration(value) => format!("Limit: {} s", value.as_secs()),
            LimitMode::Unlimited => "Bez limitu".into(),
        };
        let kind_label = match config.kind {
            ClickKind::Single => "Pojedynczy klik",
            ClickKind::Double => "Dwuklik",
        };
        let button_label = match config.button {
            MouseButton::Left => "Lewy przycisk",
            MouseButton::Right => "Prawy przycisk",
        };
        let elapsed = self.elapsed.as_secs_f32();
        let status = format!(
            "{} · cykle: {} · {:.1} s",
            self.message, self.clicks, elapsed
        );

        div()
            .id("main-view")
            .track_focus(&self.focus)
            .tab_group()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    this.send(EngineCommand::Stop);
                    cx.stop_propagation();
                }
            }))
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(div().text_xl().child("Autoclicker X20"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Klikanie w aktualnej pozycji kursora"),
            )
            .child(
                div()
                    .id("status")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(status.clone())
                    .child(status),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Interwał")
                    .child(
                        Button::new("interval-down")
                            .disabled(active)
                            .accessibility_label("Zmniejsz interwał o 10 ms")
                            .label("−")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_config(|config| {
                                    let milliseconds =
                                        config.interval.as_millis().saturating_sub(10).max(10);
                                    config.interval =
                                        std::time::Duration::from_millis(milliseconds as u64);
                                });
                                cx.notify();
                            })),
                    )
                    .child(format!("{} ms", config.interval.as_millis()))
                    .child(Button::new("interval-up").disabled(active).accessibility_label("Zwiększ interwał o 10 ms").label("+").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.update_config(|config| {
                                let milliseconds =
                                    config.interval.as_millis().saturating_add(10).min(60_000);
                                config.interval =
                                    std::time::Duration::from_millis(milliseconds as u64);
                            });
                            cx.notify();
                        },
                    ))),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("button-kind")
                            .disabled(active)
                            .label(button_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_config(|config| {
                                    config.button = match config.button {
                                        MouseButton::Left => MouseButton::Right,
                                        MouseButton::Right => MouseButton::Left,
                                    }
                                });
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("click-kind")
                            .disabled(active)
                            .label(kind_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_config(|config| {
                                    config.kind = match config.kind {
                                        ClickKind::Single => ClickKind::Double,
                                        ClickKind::Double => ClickKind::Single,
                                    }
                                });
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("limit")
                            .disabled(active)
                            .label(limit_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update_config(|config| {
                                    config.limit = match config.limit {
                                        LimitMode::Clicks(_) => {
                                            LimitMode::Duration(std::time::Duration::from_secs(30))
                                        }
                                        LimitMode::Duration(_) => LimitMode::Unlimited,
                                        LimitMode::Unlimited => LimitMode::Clicks(100),
                                    }
                                });
                                cx.notify();
                            })),
                    )
                    .when(!matches!(config.limit, LimitMode::Unlimited), |row| {
                        row.child(Button::new("limit-down").disabled(active).accessibility_label("Zmniejsz limit cykli").label("−").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.update_config(|config| {
                                    if let LimitMode::Clicks(value) = config.limit {
                                        config.limit =
                                            LimitMode::Clicks(value.saturating_sub(10).max(1));
                                    } else if let LimitMode::Duration(value) = config.limit {
                                        config.limit = LimitMode::Duration(std::time::Duration::from_secs(value.as_secs().saturating_sub(1).max(1)));
                                    }
                                });
                                cx.notify();
                            },
                        )))
                        .child(
                            Button::new("limit-up").disabled(active).accessibility_label("Zwiększ limit cykli").label("+").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.update_config(|config| {
                                        if let LimitMode::Clicks(value) = config.limit {
                                            config.limit = LimitMode::Clicks(
                                                value.saturating_add(10).min(1_000_000),
                                            );
                                        } else if let LimitMode::Duration(value) = config.limit {
                                            config.limit = LimitMode::Duration(std::time::Duration::from_secs(value.as_secs().saturating_add(1).min(86400)));
                                        }
                                    });
                                    cx.notify();
                                },
                            )),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("start")
                            .disabled(!active && !self.commands.ready.load(std::sync::atomic::Ordering::SeqCst))
                            .icon(gpui_kit::component::IconName::Check)
                            .primary()
                            .label(start_label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if active {
                                    this.send(EngineCommand::Stop);
                                } else {
                                    this.start();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("pause")
                            .disabled(!active)
                            .label(if self.status == SessionStatus::Paused {
                                "Wznów"
                            } else {
                                "Pauza"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.send(if this.status == SessionStatus::Paused {
                                    EngineCommand::Resume
                                } else { EngineCommand::Pause });
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("stop")
                            .disabled(!active)
                            .danger()
                            .label("Stop")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.send(EngineCommand::Stop);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("⌘⌥S start/stop · ⌘⌥P pauza/wznowienie · ⌘⌥X awaryjny stop"),
            )
            .child(Button::new("permission").label("Zezwól na wysyłanie kliknięć")
                .disabled(active)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.message = if crate::macos::request_access() { "Uprawnienie przyznane".into() } else {
                        "Zezwól na Dostępność w ustawieniach systemowych, a następnie kliknij Start".into()
                    };
                    cx.notify();
                })))
    }
}

fn status_message(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Idle => "Gotowy",
        SessionStatus::Countdown(value) => match value {
            1 => "Start za 1",
            2 => "Start za 2",
            _ => "Start za 3",
        },
        SessionStatus::Running => "Działa",
        SessionStatus::Paused => "Pauza",
        SessionStatus::Completed => "Ukończono",
        SessionStatus::Failed => "Błąd",
    }
}

impl Drop for MainView {
    fn drop(&mut self) {
        self.commands.shutdown_and_wait();
    }
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
