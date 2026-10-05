# ADR 0001: Jeden pakiet i osobny automat sesji

- Data: 2026-10-05
- Status: zaakceptowana
- Autor: Szymon Sosnowski

## Kontekst

Autoclicker potrzebuje UI, globalnych skrótów i przerwania sesji bez blokowania głównego wątku.
Testy nie mogą klikać w pulpit użytkownika.

## Decyzja

Używamy jednego pakietu Rust. UI i integracja macOS komunikują się z pojedynczym workerem.
Automat sesji otrzymuje czas monotoniczny jako parametr, więc można go testować deterministycznie.
Adapter myszy jest atrapą w testach workera.

## Rozważane opcje

Klikanie na wątku UI blokowałoby interakcje. Osobny worker dla każdego Startu umożliwiałby
równoległe sesje. Wybrany worker obsługuje wszystkie sesje po kolei.

## Konsekwencje

Testy automatu nie potrzebują uprawnień. Testy prawdziwych zdarzeń oraz skrótów wymagają macOS
i neutralnego celu. UI nie jest źródłem prawdy o stanie sesji.
