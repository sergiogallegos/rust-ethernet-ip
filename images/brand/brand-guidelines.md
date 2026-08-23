# Brand Package

This folder contains the brand assets actually referenced by the README and
`website/`. Unused crops and the original source image were removed to keep
this directory in sync with what's live. Restore the original source image
from repository history before generating a new variant.

## Files
- `logo-light.png`: Primary logo for light backgrounds (README, website footer)
- `logo-dark.png`: Primary logo for dark backgrounds (README dark-mode source)
- `package-preview.png`: Full package overview crop (website `og:image`)
- `favicon/icon-32.png`: Site favicon (website `<link rel="icon">`)

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
