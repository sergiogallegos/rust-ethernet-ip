# Project Website

Static, dependency-free website for the stable `1.2.0` release line and the
`1.2.1` development preview. Open
`website/index.html` directly or serve the repository root for local review:

```bash
python3 -m http.server 8080
```

Then open `http://127.0.0.1:8080/website/`.

The site includes dedicated `privacy.html` and `license.html` pages. It ships
without analytics, cookies, forms, or third-party font requests. It stores one
local quick-start language preference and does not transmit it. The logo and
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

1. In Cloudflare, open **Workers & Pages**, choose **Create**, then **Pages** and
   **Connect to Git**.
2. Select the `sergiogallegos/rust-ethernet-ip` repository and production branch
   `main`.
3. Use framework preset `None`, leave the build command empty, and set the build
   output directory to `website`. The repository root stays at its default.
4. Deploy and verify the generated `*.pages.dev` preview before attaching DNS.
5. In the Pages project, open **Custom domains**, choose **Set up a domain**, and
   add `rustethernetip.com`. Cloudflare should configure the apex record because
   the domain uses Cloudflare DNS.
6. Add `www.rustethernetip.com` as a proxied DNS name and configure a permanent
   redirect to `https://rustethernetip.com` that preserves paths and query
   strings. Keep the apex domain as the only canonical URL.

Every push to `main` deploys the production site. Pull-request branches receive
preview deployments. The `_headers` file adds browser security headers; verify
them after deployment because they are applied by Cloudflare, not by a basic
local file server.

### Launch verification

- Confirm `/`, `/privacy`, `/license`, `/404.html`, `/robots.txt`,
  `/sitemap.xml`, and `/.well-known/security.txt` return successfully.
- Confirm HTTPS works without certificate warnings and HTTP redirects to HTTPS.
- Confirm `www` redirects to the apex while preserving a test path and query.
- Test the header, navigation, language tabs, outbound package links, sponsor
  link, and footer at desktop and mobile widths.
- Inspect response headers for the content security policy, HSTS, frame denial,
  MIME sniffing protection, referrer policy, and permissions policy.
- Keep analytics disabled unless the privacy policy is updated before enabling
  it.

### GitHub Pages

Publish the `website/` directory with a Pages workflow or copy it to a `gh-pages`
branch. The site uses absolute public URLs for project documentation and brand
assets, so it works at either a domain root or a project subpath.

## Maintenance

Update release numbers, hardware proof points, and package links during each
release. Keep the authoritative details in `README.md`, `docs/`, and validation
records; the website should summarize and link to those sources.
