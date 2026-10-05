<!-- code-docs: lang=PL+EN map=docs/PROJECT_MAP.md owner=Szymon assumed=no -->

# Mapa projektu / Project map

Autoclicker X20 wysyła kliknięcia użytkownika na macOS.
Autoclicker X20 sends user-controlled clicks on macOS.

## Wejścia / Entry points

[main.rs](../src/main.rs) uruchamia jedno okno i utrzymuje natywne adaptery.
[main.rs](../src/main.rs) opens one window and retains the native adapters.

`cargo run --locked` uruchamia aplikację; `cargo test --locked` uruchamia testy.
`cargo run --locked` runs the app; `cargo test --locked` runs tests.
`cargo build --release --locked` i [skrypt pakowania](../packaging/macos/build-app.sh) tworzą paczkę.
`cargo build --release --locked` and the [packaging script](../packaging/macos/build-app.sh) build the bundle.

## Moduły / Modules

| Plik / File | Cel PL | Purpose EN |
| --- | --- | --- |
| [engine.rs](../src/engine.rs) | Kontrakty, kanały i jeden worker | Contracts, channels and one worker |
| [session.rs](../src/session.rs) | Automat z jawnym zegarem | State machine with an explicit clock |
| [macos.rs](../src/macos.rs) | Core Graphics, zgoda, skróty, uśpienie | Core Graphics, consent, hotkeys, sleep |
| [ui.rs](../src/ui.rs) | Kontrolki i stan widoku | Controls and view state |
| [engine_tests.rs](../src/engine_tests.rs) | Atrapa myszy i testy workera | Fake mouse and worker tests |
| [session_tests.rs](../src/session_tests.rs) | Testy deterministycznego czasu | Deterministic-time tests |
| [ui_tests.rs](../src/ui_tests.rs) | Testy produkcyjnego widoku | Production-view tests |

## Połączenia / Connections

`src/main.rs::main` → `src/engine.rs::SessionEngine`, `src/macos.rs::HotkeyRegistration`,
`src/macos.rs::SleepObserver`, `src/ui.rs::MainView`.
`src/engine.rs::SessionEngine` → `src/session.rs::Session`, `src/engine.rs::ClickExecutor`.
`src/macos.rs::MacClickExecutor` implementuje / implements `src/engine.rs::ClickExecutor`.

## Przepływ / Flow

UI i skróty → EngineCommand → worker → Session → ClickExecutor → Core Graphics.
UI and hotkeys → EngineCommand → worker → Session → ClickExecutor → Core Graphics.
EngineEvent wraca do widoku. Uśpienie i zamknięcie wysyłają Stop/Shutdown.
EngineEvent returns to the view. Sleep and close send Stop/Shutdown.

## Konfiguracja / Configuration

Brak zmiennych środowiska w aplikacji. Stały bundle ID: `pl.kamciosz.autoclicker-x20`.
The application reads no environment variables. Fixed bundle ID: `pl.kamciosz.autoclicker-x20`.
[Rust](../rust-toolchain.toml), [zależności](../Cargo.toml), [CI](../.github/workflows/ci.yml).
[Rust](../rust-toolchain.toml), [dependencies](../Cargo.toml), [CI](../.github/workflows/ci.yml).

## Konwencje i odpowiedzialność / Conventions and ownership

Komentarze PL+EN; tagi `@uses`, `@used_by`, `@invariant`. Mapa opisuje kod tego repo.
Comments use PL+EN and `@uses`, `@used_by`, `@invariant`. This map describes this repository's code.
Nie dodawaj nowych autorów. Istniejąca tożsamość Git to Szymon Sosnowski.
Do not add authors. The existing Git identity is Szymon Sosnowski.
Decyzje: [ADR 0001](adr/0001-struktura-projektu.md), [ADR 0002](adr/0002-stos-gpui-i-integracja-macos.md).
Decisions: [ADR 0001](adr/0001-struktura-projektu.md), [ADR 0002](adr/0002-stos-gpui-i-integracja-macos.md).
