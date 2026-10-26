# 0025 The showcase pin moves by itself

Date: 2026-10-05. Status: accepted, partly superseded by 0057.

The pin moves by itself: `.github/workflows/showcase.yml` looks at the catalog's main every 30 minutes and, once that commit's checks pass, builds the gallery and the site, deploys through `CLOUDFLARE_API_TOKEN` (secret) and `CLOUDFLARE_ACCOUNT_ID` (variable), then commits the pin; a run with `deploy` set redeploys at the current pin. A catalog commit that fails its checks keeps the pin. The catalog runs its checks on every pull request without approval, save from accounts new to GitHub; they get no secrets and a read-only token.
