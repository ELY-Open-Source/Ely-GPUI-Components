# 0022 The Showcase lives in its own repo

Date: 2026-10-04. Status: accepted, partly superseded by 0025.

The Showcase follows gpui-kit.com/apps: apps live in their own public repo and arrive by pull request. The site pins a commit of it in `frontend/showcase.lock` and moves only when someone bumps the pin, so stars refresh on the site with each bump. READMEs take Markdown and PNG pictures alone, no HTML or video, so the site needs no sanitizer. The catalog started empty.
