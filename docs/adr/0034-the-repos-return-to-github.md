# 0034 The repos return to GitHub

Date: 2026-10-19. Status: accepted.

Gitea retires; the repos live on GitHub under `ELY-Open-Source`. The site links there with GitHub's mark, `pnpm showcase` downloads the catalog from codeload.github.com, and `showcase.yml` reads the catalog's head and its check runs (and any commit statuses) through GitHub's API. CI runs on GitHub's hosted runners, where the Linux leg is already a plain user. The catalog's stars sign in with its workflow's `GITHUB_TOKEN`; since a push with that token starts no workflow, the stars job dispatches the catalog's checks on the commit it pushes, or the pin would wait on it.
