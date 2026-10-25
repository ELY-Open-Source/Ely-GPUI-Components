# 0033 The repos move to Gitea

Date: 2026-10-09. Status: superseded by 0034.

The repos moved from GitHub to Gitea at git.zacharyzhang.com; GitHub's copies are gone. The site links there, `pnpm showcase` downloads the catalog's archive from there, and `showcase.yml` runs on Gitea Actions, reading the catalog's head and combined commit status through Gitea's API. The catalog's stars still come from GitHub's API, signed in only with its `STARS_GITHUB_TOKEN` secret, since Gitea's own token means nothing there. TokenLedger, merged on GitHub after the copy was taken, was restored from the site's copy.
