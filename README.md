# Autoclicker X20

Autoclicker dla macOS, napisany w Rust z GPUI Kit. Klika w aktualnej pozycji kursora.
Bez nagrywania makr, profili, sieci i telemetrii.

## Instalacja

Rozpakuj `Autoclicker-X20-0.1.0-macos-arm64.zip` i przenieś `Autoclicker X20.app`
do folderu Aplikacje. Pakiet jest przeznaczony dla Apple Silicon i macOS 15 lub nowszego.
Sprawdzony system oraz pozostałe ograniczenia opisuje [raport weryfikacji](docs/VERIFICATION.md).

Podpis jest lokalny, ad hoc. To nie jest podpis Developer ID ani notaryzacja Apple.
Gatekeeper może wymagać zatwierdzenia w ustawieniach bezpieczeństwa.
Nie wyłączaj ochrony systemu. Jeżeli system odrzuca paczkę, zbuduj ją ze źródeł.

## Obsługa

Domyślnie: 100 ms, lewy przycisk, pojedynczy klik, 100 cykli. Minimalny interwał to 10 ms.
Interwał oznacza przerwę po ukończeniu cyklu, więc rzeczywiste tempo zależy też od systemu.
Dwuklik to dwie pary naciśnięcie/zwolnienie w jednej pozycji, liczone jako jeden cykl.

Wybierz limit cykli, czas aktywnego klikania albo tryb bez limitu.
Przyciski −/+ zmieniają interwał i wybrany limit. Konfiguracja jest zablokowana podczas sesji,
również w pauzie. Start i wznowienie poprzedza trzysekundowe odliczanie.
Pauza nie zużywa limitu czasu. Stop kończy sesję; kolejny Start zeruje liczniki.
Przerwanie częściowego dwukliku nie zalicza cyklu.

| Skrót | Działanie |
| --- | --- |
| `⌘⌥S` | Start albo Stop |
| `⌘⌥P` | Pauza albo wznowienie |
| `⌘⌥X` | Awaryjny Stop |
| `Escape` | Stop w aktywnym oknie |

Konflikt skrótów blokuje Start i pokazuje błąd. Zamknięcie okna, uśpienie i utrata zgody
kończą sesję bez samoczynnego wznowienia. Nie ma automatycznego startu.

## Uprawnienia

Kliknij „Zezwól na wysyłanie kliknięć”, aby świadomie wywołać prośbę systemową.
Zgodę nadajesz samodzielnie w `Ustawienia systemowe → Prywatność i ochrona → Dostępność`.
Aplikacja nie zmienia tych ustawień i nie monitoruje wpisywanego tekstu.
Sprawdza możliwość wysyłania zdarzeń przed Startem oraz w trakcie sesji.

## Budowanie ze źródeł

Wymagane są Rust 1.97.1 i Xcode Command Line Tools na macOS.

```bash
git clone https://github.com/Kamciosz/autoclicker-x20.git
cd autoclicker-x20
cargo run --locked
cargo build --release --locked
./packaging/macos/build-app.sh
```

Skrypt tworzy `dist/Autoclicker X20.app` i ZIP w `outputs/`.
Odmawia zastąpienia istniejącej aplikacji: przenieś poprzedni pakiet przed ponownym pakowaniem.
Sprawdza architekturę arm64, plist i podpis. Licencje zależności trafiają do zasobów aplikacji.

## Testy

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
~/engineering-standard/bin/quality-audit .
```

Testy automatu używają jawnego zegara. Testy workera używają atrapy myszy.
Testy GPUI renderują produkcyjny widok, sprawdzają zmianę danych, walidację, blokady,
Tab/Enter oraz Escape. Nie zastępują testów natywnych kliknięć ani VoiceOver.

GPUI Kit 0.7.1 nie udostępnia natywnej flagi `disabled` dla użytych przycisków.
Testy sprawdzają odrzucanie interakcji i brak fokusu zablokowanej kontrolki.

## Dokumentacja

- [Architektura](docs/ARCHITECTURE.md)
- [Mapa projektu PL+EN](docs/PROJECT_MAP.md)
- [Decyzje](docs/adr/0001-struktura-projektu.md)
- [Zasoby i licencje](docs/ASSETS.md)
- [Weryfikacja i ograniczenia](docs/VERIFICATION.md)
- [Współpraca](CONTRIBUTING.md)

## Licencja

Kod projektu: MIT, zobacz [LICENSE](LICENSE).
Zależności mają własne licencje: [Third-party notices](docs/THIRD_PARTY_NOTICES.html).
