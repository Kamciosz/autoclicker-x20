# Zasoby / Assets

Interfejs używa ikony `Check` i domyślnych zasobów GPUI Kit Assets 0.7.1.
The UI uses the `Check` icon and default GPUI Kit Assets 0.7.1 resources.

Źródło: pakiet `gpui-kit-assets` z crates.io, repozytorium [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit).
Ikony Lucide są na licencji ISC; odziedziczona ikona check ma też informację MIT projektu Feather.
Oryginalną treść zachowuje [LICENSE-LUCIDE](LICENSE-LUCIDE).
Source: the `gpui-kit-assets` crate from crates.io, [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit).
Lucide icons use ISC; the inherited check icon also carries Feather's MIT notice.
Original terms are preserved in [LICENSE-LUCIDE](LICENSE-LUCIDE).

Nie importujemy własnych fontów, obrazów ani ikon. Font interfejsu dobiera system i GPUI.
No custom fonts, images or icons are imported. The system and GPUI select the UI font.

[THIRD_PARTY_NOTICES.html](THIRD_PARTY_NOTICES.html) zawiera licencje zależności dla arm64 macOS.
Pakowanie dołącza oba pliki licencyjne do `Contents/Resources`.
[THIRD_PARTY_NOTICES.html](THIRD_PARTY_NOTICES.html) contains dependency notices for arm64 macOS.
Packaging includes both license files in `Contents/Resources`.

Odtworzenie / Regeneration:

```bash
cargo install cargo-about --version 0.9.2 --locked --features cli
cargo about generate --locked --fail --target aarch64-apple-darwin -o docs/THIRD_PARTY_NOTICES.html packaging/macos/notices.hbs
perl -pi -e 's/[ \t\r]+$//' docs/THIRD_PARTY_NOTICES.html
```
