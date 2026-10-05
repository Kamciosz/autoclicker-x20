# ADR 0002: GPUI Kit i natywna integracja macOS

## Status

Zaakceptowana

## Decyzja

Używamy GPUI Kit 0.7.1 jako warstwy UI oraz Core Graphics 0.25.0 jako wąskiej granicy wysyłania
zdarzeń myszy. Globalne skróty zapewnia `global-hotkey` 0.8.0.
Zależności są przypięte dokładnie; Rust to 1.97.1, a graf zależności utrwala Cargo.lock.
Istniejące w grafie `objc2`, `objc2-app-kit`, `objc2-foundation` i `block2` obsługują
powiadomienie NSWorkspaceWillSleep bez ręcznego definiowania klas Objective-C.

## Uzasadnienie

GPUI Kit daje kontrolki z obsługą fokusu i dostępności, a Core Graphics umożliwia zachowanie
macOS-owego modelu kliknięć bez wprowadzania backendów dla innych systemów. Worker sesji pozostaje
niezależny od UI, dzięki czemu automat można testować bez wysyłania prawdziwych zdarzeń.

## Konsekwencje

Użytkownik musi nadać aplikacji uprawnienie Dostępność. Paczki są budowane na macOS; inne systemy
nie są obsługiwanym celem tej wersji.
