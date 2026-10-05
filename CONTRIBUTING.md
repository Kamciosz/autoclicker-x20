# Współpraca

## Uruchomienie

```bash
git clone https://github.com/Kamciosz/autoclicker-x20.git
cd autoclicker-x20
git config core.hooksPath .githooks
cargo run --locked
```

Wymagania: macOS, Apple Silicon, Rust 1.97.1 i Xcode Command Line Tools.
Nie zmieniaj globalnej tożsamości Git.

## Zmiany

Pracuj na gałęziach `feat/`, `fix/` lub `docs/`.
Pierwsza publikacja nowego repo inicjuje `main`; kolejne zmiany trafiają przez PR.
Nie przepisuj współdzielonej historii.

Commity są atomowe i mają format Conventional Commits.
Opisy zapisujemy po polsku w trybie rozkazującym, np. `feat(engine): dodaj limity sesji`.
Hook sprawdza format i długość tematu.

## Weryfikacja

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
~/engineering-standard/bin/quality-audit .
```

Formatter to rustfmt; linter to clippy. Testy nie wysyłają prawdziwych kliknięć.
Testy natywne wykonuj wyłącznie na neutralnym celu, bez przycisków płatności, usuwania lub wysyłania.

Zmiany zachowania wymagają aktualizacji README i changelogu.
Zmiany modułów wymagają aktualizacji [mapy](docs/PROJECT_MAP.md).
Decyzje zapisuj w `docs/adr/`. Zależności i ich licencje sprawdzaj razem z lockfile.
Nie opisuj nieprzeprowadzonych testów jako zaliczonych.
