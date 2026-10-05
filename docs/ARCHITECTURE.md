# Architektura

Autoclicker X20 jest jednopakietową aplikacją macOS. UI GPUI nie wysyła zdarzeń bezpośrednio:
przekazuje polecenia do silnika sesji, a silnik wywołuje wąską granicę integracji Core Graphics.

## Moduły

- `src/engine.rs` — konfiguracja, kanały i worker sesji;
- `src/session.rs` — automat z czasem monotonicznym przekazywanym przez workera lub test;
- `src/macos.rs` — Core Graphics, sprawdzanie Dostępności i globalne skróty;
- `src/ui.rs` — właściciel stanu widoku i kontrolki GPUI Kit;
- `src/main.rs` — bootstrap aplikacji, rejestracja skrótów i połączenie modułów.

## Przepływ

```text
GPUI Button / global hotkey
        ↓ EngineCommand
SessionEngine worker
        ↓ ClickExecutor
Core Graphics → bieżąca pozycja kursora
        ↓ EngineEvent
GPUI view status and counter
```

Worker jest pojedynczy i reaguje na `Stop` podczas odliczania, kliknięcia i oczekiwania. Czas
limitu mierzy tylko okres `Running`, a pauza nie zużywa limitu. Zdarzenia są wysyłane przez kanał,
który nie przechowuje żadnych danych użytkownika.

## Uprawnienia i ograniczenia

Core Graphics wymaga zgody Dostępność do publikowania zdarzeń myszy. Aplikacja sprawdza ją przed
każdą sesją oraz w workerze; brak zgody kończy sesję komunikatem. Aplikacja nie prosi o dostęp
automatycznie przy starcie i nie zapisuje danych w sieci.
