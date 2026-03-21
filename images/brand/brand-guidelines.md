# Brand Package

This folder contains extracted brand assets derived from:
- `images/ChatGPT Image Mar 21, 2026, 03_22_22 PM.png`

## Files
- `logo-light.png`: Primary logo for light backgrounds
- `logo-dark.png`: Primary logo for dark backgrounds
- `logo-horizontal.png`: Horizontal wordmark variant
- `logo-stacked.png`: Compact stacked variant
- `icon.png`: Icon/mark-only style crop
- `palette-guide.png`: Color/type reference panel crop
- `package-preview.png`: Full package overview crop
- `favicon/`: Generated favicon/app icon set from `icon.png`
  - `icon-16.png`, `icon-32.png`, `icon-64.png`
  - `icon-180.png` (Apple touch icon)
  - `icon-192.png`, `icon-512.png` (PWA/manifest)
  - `favicon.ico`

## Suggested Tokens
- Rust Orange: `#E4572E`
- Dark Rust (as shown in concept board): `#B3A1E` (verify before final production tokenization)
- Dark Text: `#2E2E2E`
- Light Text: `#EDEDED`

## Usage Notes
- Prefer `logo-light.png` on white/light surfaces.
- Prefer `logo-dark.png` on dark/navy surfaces.
- In README, use a `<picture>` element for theme switching.
- These are raster crops (PNG), not true vector source files.
