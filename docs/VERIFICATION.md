# Weryfikacja 0.1.0

## Środowisko

Testy lokalne wykonano na macOS 26.6.2, arm64, kompilatorem Rust 1.97.1
(commit 8bab26f4f). Lokalny alias rustup `1.97.1` ma niekompletny sysroot.
Sprawdzony alias `stable` raportuje dokładnie 1.97.1 i służył do lokalnych bramek:
`RUSTUP_TOOLCHAIN=stable cargo ...`. Repo i CI pozostają przypięte do 1.97.1.

## Wyniki

- PASS: fmt, check, test i clippy z lockfile oraz `-D warnings`.
- PASS: automat z kontrolowanym zegarem, atrapa myszy i produkcyjny widok GPUI.
- PASS: release arm64, plist, bundle ID i weryfikacja podpisu ad hoc.
- PASS: uruchomienie paczki z `dist/`, bez zależności od katalogu `target/`.
- PASS: Start bez zgody systemowej pokazuje blokadę i pozostawia licznik równy zero.
- PASS: zamknięcie jedynego okna kończy proces.
- PASS: cargo-audit nie zgłosił znanych podatności w lockfile.

Cargo-audit zgłosił nieutrzymywane zależności pośrednie: instant, paste, rustls-pemfile,
rustybuzz i ttf-parser. Kompilator zgłasza przyszłą niezgodność zależności block 0.1.6.
Nie są to wyciszone ostrzeżenia kodu aplikacji.

## UNVERIFIED

- Rzeczywisty lewy/prawy/dwuklik na neutralnym celu: paczka nie ma zgody Dostępność.
- Globalne skróty z aktywnym i nieaktywnym oknem: próba przez narzędzie UI nie wykazała
  zmiany stanu. Rejestracja nie zwróciła błędu, ale to nie potwierdza działania skrótów.
- Natywne Escape: test GPUI przechodzi; próba narzędziem UI nie potwierdziła zmiany stanu.
- Zatrzymanie aktywnego klikania przy prawdziwym uśpieniu lub odebraniu zgody.
- Konflikt skrótów z inną aplikacją; lokalna rejestracja zakończyła się bez błędu.
- VoiceOver, Intel oraz starsze wersje macOS.
- Developer ID i notaryzacja nie są częścią tego wydania.

## Procedura natywna

Na pustym celu testowym nadaj zgodę systemową i ustaw limit 1 cyklu.
Sprawdź lewy, prawy i dwuklik. Dwuklik powinien dać dwie pary zdarzeń i jeden cykl.
Sprawdź Start, pauzę, wznowienie i Stop, trzysekundowe odliczanie oraz Escape.
Przenieś fokus do innej aplikacji i sprawdź wszystkie trzy skróty.
Sprawdź uśpienie, odebranie zgody i zamknięcie podczas sesji: żaden przypadek nie może
samoczynnie wznowić klikania. Nie używaj przycisków wysyłania, płatności ani kasowania.

To wydanie ma nieweryfikowane ścieżki natywne; nie należy traktować bramek kompilacji
ani rejestracji skrótów jako dowodu pełnego działania.
