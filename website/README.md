# Project Website

Static, dependency-free website for the `1.2.0` release line. Open
`website/index.html` directly or serve the repository root for local review:

```bash
python3 -m http.server 8080
```

Then open `http://127.0.0.1:8080/website/`.

The site includes dedicated `privacy.html` and `license.html` pages. It ships
without analytics, cookies, forms, or third-party font requests. The logo and
favicon currently load from the public GitHub repository.

Navigation uses a CSS-rendered text wordmark so it stays crisp and compact at
every viewport size; the footer uses the full mascot logo at
`images/brand/logo-light.png`.

The “Where the library fits” flow distinguishes user interfaces and application
code from the driver, EtherNet/IP/CIP transport, and Logix controller. Keep the
web note intact: browser code should call a backend or edge service that owns
the PLC connection.

## Deployment

### Cloudflare Pages

- Connect the GitHub repository.
- Framework preset: `None`.
- Build command: leave empty.
- Build output directory: `website`.
- Add `rustethernetip.com` as the canonical custom domain after registration.
- Optionally register `rust-ethernet-ip.com` and configure a permanent redirect
  to the canonical unhyphenated domain.

### GitHub Pages

Publish the `website/` directory with a Pages workflow or copy it to a `gh-pages`
branch. The site uses absolute public URLs for project documentation and brand
assets, so it works at either a domain root or a project subpath.

## Maintenance

Update release numbers, hardware proof points, and package links during each
release. Keep the authoritative details in `README.md`, `docs/`, and validation
records; the website should summarize and link to those sources.
