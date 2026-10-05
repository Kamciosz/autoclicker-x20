# AGENTS.md — zasady pracy w tym repo (dla ludzi i agentów AI)

## Definition of done
Zmiana jest gotowa dopiero, gdy wszystkie poniższe komendy przechodzą:

```bash
# format
cargo fmt --check
# build
cargo check --locked
# testy
cargo test --locked
# lint
cargo clippy --locked --all-targets -- -D warnings
```

Nie zgłaszaj „gotowe" bez uruchomienia tych komend i wklejenia wyniku.

## Kod
- Małe funkcje i moduły: plik > 600 linii = sygnał do podziału, funkcja > ~50 linii = sygnał do rozbicia.
- Żadnego martwego kodu: nie komentujemy — usuwamy (historia gita pamięta).
- Żadnych debug-printów w kodzie produkcyjnym (logger tak).
- Komentarze tylko tam, gdzie nie da się tego wyczytać z kodu (dlaczego, nie co). Konwencja: PL+EN, tagi `@uses/@used_by/@param` — patrz skill `code-docs-master`.
- Nazwy po angielsku, opisowe; brak skrótów typu `tmp2`, `data1`.

## Struktura
- `src/` — kod i testy modułów, `tests/` — testy metadanych, `docs/` — dokumentacja, `docs/adr/` — decyzje. Mapa: `docs/PROJECT_MAP.md`.
- Nowa funkcja = nowy plik/moduł, nie doklejka do istniejącego molocha.
- Zależności: najpierw to, co już jest w projekcie; nowa biblioteka = wpis w `docs/adr/` (dlaczego).

## Git
- Jeden commit = jedna logiczna zmiana; commity atomowe, opisowe (Conventional Commits). Hook `commit-msg` pilnuje formatu.
- Gałęzie: `feat/nazwa`, `fix/nazwa`, `docs/nazwa`. Pierwsza publikacja inicjuje `main`; następne zmiany trafiają przez PR.
- Nie przepisuj historii wspólnych gałęzi (`--force`); na własnej gałęzi — `--force-with-lease`.
- PR: opis co/po co/jak testowane/ryzyka (`.github/pull_request_template.md`).
- Przed PR: `git fetch && git rebase origin/main` (nie merge z main).

## Dokumentacja
- Zmiana zachowania → aktualizacja `README.md` i `CHANGELOG.md` (sekcja `Unreleased`).
- Decyzja architektoniczna → `docs/adr/NNNN-tytul.md`.
- Nowy moduł → dopisz do `docs/ARCHITECTURE.md`.

## Higiena repo
- Zero plików tymczasowych w repo; `.gitignore` pokrywa artefakty stacku.
- Nie commituj sekretów (`.env`, klucze). Sekret w historii = rotacja klucza, nie tylko usunięcie pliku.
