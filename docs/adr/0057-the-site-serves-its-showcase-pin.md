# 0057 The site serves its showcase pin

Date: 2026-10-26. Status: accepted.

Since 0054, `showcase.yml` cannot push its pin to `main`: the workflow's token is no admin, GitHub takes no Actions bypass for a repository ruleset, and deploy keys are off for this repository. So the deployed site serves the catalog commit it shows at `/showcase.lock`, and the next run compares the catalog's main with that; a site that names none, its page answering the path, goes through the catalog's checks as a new commit would. `frontend/showcase.lock` in `main` stays the pin for local builds, moved by hand. This replaces 0025's commit of the pin.
