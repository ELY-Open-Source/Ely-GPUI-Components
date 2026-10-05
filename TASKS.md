# Tasks

Progress truth. Work runs top to bottom. No item is skipped.
Each `T` item ends with a review in a herdr split pane, three rounds at most: codex (`gpt-6-astra`, effort `max`) through T24d round 2; from T24d round 3, after codex ran out of quota on 2026-09-26, a Claude session on Fable 5.1 at max effort, at the user's word; from T44a round 2, codex again, as the user asked; from T45d round 2, Claude Code on Fable 5.1 at high effort, at the user's word; from T50a, codex `gpt-6.1-sol` at `max`, at the user's word.

Per-component lines live in `tasks/`. Tags there:

| Tag | Meaning |
|-----|---------|
| `→ path` | Alias or duplicate. One home, at `path`. |
| `native` | gpui ships it. The gallery shows the native call. |
| `host` | Ely draws the UI. The host app supplies the engine data. |
| `prove` | Needs a crate or platform API not yet proven with gpui. |
| `blocked` | Cannot be real on gpui. Reason given. |
| `(Txx)` | Built early in task `Txx` as a dependency. |

## Items

- [x] T00 Foundation: package, theme, motion, assets, Icon, Button, gallery shell, docs
- [x] T01 Primitives — `tasks/ch01-10.md`
- [x] T02 Typography
- [x] T03 Layout
- [x] T04 Window & Shell
- [x] T05 Buttons & Actions
- [x] T06a Forms · Text
- [x] T06b Forms · Selection
- [x] T06c Forms · Date & Time
- [x] T06d Forms · Color
- [x] T06e Forms · Files
- [x] T06f Forms · Other
- [x] T06g Forms · Structure
- [x] T07a Navigation · Tabs & paths
- [x] T07b Navigation · Places
- [x] T07c Navigation · Palettes & tour
- [x] T08a Menus · Menu & hosts
- [x] T08b Menus · Bar, search & pie
- [x] T09a Overlays · Popovers & dialogs
- [x] T09b Overlays · Guides & floats
- [x] T10a Feedback · Messages
- [x] T10b Feedback · States
- [x] T11a Loading · Spinners & skeletons — `tasks/ch11-20.md`
- [x] T11b Loading · Progress & states
- [x] T11c Motion · Presence & order
- [x] T11d Motion · Effects
- [x] T12a Data Display · People & numbers
- [x] T12b Data Display · Records
- [x] T12c Data Display · Media
- [x] T12d Data Display · Codes & measures
- [x] T13a Lists & Trees · Lists
- [x] T13b Lists & Trees · Trees
- [x] T13c Lists & Trees · Files
- [x] T14a Tables · Core
- [x] T14b Tables · Columns & rows
- [x] T14c Tables · Grids
- [x] T15a Charts · Cartesian
- [x] T15b Charts · Parts and spreads
- [x] T15c Charts · Flows and relations
- [x] T16a Finance · Market charts
- [x] T16b Finance · Technical analysis
- [x] T16c Finance · Quotes & book
- [x] T16d Finance · Trading
- [x] T16e Finance · Markets & assets
- [x] T17a Editor · Core
- [x] T17b Editor · Intelligence
- [x] T17c Editor · Search
- [x] T17d Editor · Panels
- [x] T17e Editor · Status items
- [x] T18 Terminal
- [x] T19 Git
- [x] T20 Debug
- [x] T21a Documents · Editing — `tasks/ch21-30.md`
- [x] T21b Documents · Reading
- [x] T21c Documents · Knowledge
- [x] T22 Collaboration
- [x] T23a AI Chat · Messages
- [x] T23b AI Chat · Citations
- [x] T23c AI Chat · Input
- [x] T23d AI Chat · Conversations
- [x] T23e AI Chat · Welcome
- [x] T24a Agent · Tool calls
- [x] T24b Agent · Progress
- [x] T24c Agent · Changes
- [x] T24d Agent · Previews
- [x] T24e Agent · Environment
- [x] T24f Agent · Control and cost
- [x] T25a Generative · Prompt and settings
- [x] T25b Generative · Queue and results
- [x] T25c Generative · Canvas
- [x] T25d Generative · Sound and motion
- [x] T25e Generative · Models
- [x] T25f Generative · Data and prompts
- [x] T26a Media · Images
- [x] T26b Media · Crop and annotate
- [x] T26c Media · Video
- [x] T26d Media · Audio and controls
- [x] T26e Media · Capture and devices
- [x] T27a Files · Items
- [x] T27b Files · Explorer
- [x] T27c Files · Find and look
- [x] T27d Files · Transfers
- [x] T28a Messaging · Channels
- [x] T28b Messaging · Messages
- [x] T28c Messaging · People
- [x] T28d Messaging · Calls
- [x] T28e Messaging · Stickers and GIFs
- [x] T29a Mail · Boxes and lists
- [x] T29b Mail · Reading
- [x] T29c Mail · Writing
- [x] T29d Mail · Sorting
- [x] T30a Calendar · Months
- [x] T30b Calendar · Time grid
- [x] T30c Calendar · Agenda and people
- [x] T30d Calendar · Editing
- [x] T31a Project · Tasks — `tasks/ch31-43.md`
- [x] T31b Project · Issues and boards
- [x] T31c Project · Plans
- [x] T31d Project · Time and databases
- [x] T32a Canvas · Plane — `tasks/ch31-43.md`
- [x] T32b Canvas · Tools — `tasks/ch31-43.md`
- [x] T32c Canvas · Panels — `tasks/ch31-43.md`
- [x] T32d Canvas · Graphs and boards — `tasks/ch31-43.md`
- [x] T33a DB & Dev Tools · Databases — `tasks/ch31-43.md`
- [x] T33b DB & Dev Tools · Data formats — `tasks/ch31-43.md`
- [x] T33c DB & Dev Tools · APIs — `tasks/ch31-43.md`
- [x] T33d DB & Dev Tools · Tools and machines — `tasks/ch31-43.md`
- [x] T34 Dashboard — `tasks/ch31-43.md` (its two maps lines wait for T41)
- [x] T35 Settings — `tasks/ch31-43.md`
- [x] T36a Account · Sign in — `tasks/ch31-43.md`
- [x] T36b Account · Profile and access — `tasks/ch31-43.md`
- [x] T36c Account · Billing and team — `tasks/ch31-43.md`
- [x] T37a Onboarding · Flows and highlights — `tasks/ch31-43.md`
- [x] T37b Onboarding · Help — `tasks/ch31-43.md`
- [x] T38a Interaction · Pointer — `tasks/ch31-43.md`
- [x] T38b Interaction · Keys, scroll and focus — `tasks/ch31-43.md`
- [x] T39a Theme · Editing — `tasks/ch31-43.md`
- [x] T39b Theme · Importing — `tasks/ch31-43.md`
- [x] T39c Theme · Platform — `tasks/ch31-43.md`
- [x] T40a i18n — `tasks/ch31-43.md`
- [x] T40b a11y — `tasks/ch31-43.md`
- [x] T41a Maps · View — `tasks/ch31-43.md`
- [x] T41b Maps · Layers — `tasks/ch31-43.md`
- [x] T41c Maps · World — `tasks/ch31-43.md`
- [x] T42a Misc · Time — `tasks/ch31-43.md`
- [x] T42b Misc · Numbers — `tasks/ch31-43.md`
- [x] T42c Misc · Codes and checks — `tasks/ch31-43.md`
- [x] T42d Misc · Asking and learning — `tasks/ch31-43.md`
- [x] T42e Misc · Terms and notices — `tasks/ch31-43.md`
- [x] T42f Misc · Web — `tasks/ch31-43.md`
- [x] T42g Misc · Export and import — `tasks/ch31-43.md`
- [x] T43a Tooling · Workbench — `tasks/ch31-43.md`
- [x] T43b Tooling · Inspection — `tasks/ch31-43.md`
- [x] T43c Tooling · Catalogs — `tasks/ch31-43.md`
- [x] T44a Capture · Roots: roots still on `w_full` drop it, checked at 280px and in padded cards
- [x] T44b Port · Build: gpui and gpui_platform from Zed's repo by git at 1a28cff; the crate, its tests and the gallery build, and check.sh passes
- [x] T44c Port · Parity: every page and scripted state captured light and dark and matched against T44a's shots; AGENTS.md's gpui 0.2.2 and taffy 0.9 rules re-tested at 1a28cff and rewritten
- [x] T44d Web · Stories: the gallery builds for wasm32 through gpui_web; one story per component, chosen by URL, themed by the host page; what cannot run in a browser says why
- [x] T45 Website: `frontend/` rebuilt after `frontend-reference/` (serro.ai's language: heavy titles, thin faint text, hairlines, one accent), Vite 8, pnpm, React 19, TanStack Router, light and dark; isometric line miniatures that grow cell by cell on a marimba beat; every story live, a native capture where a browser cannot run it; no box inside a box, no subheadings; run locally, not published
- [x] T45a Site · Stack: the old `frontend/` replaced by Vite 8, React 19 and TanStack Router under pnpm; routes `/`, `/components/`, `/components/$page/$story`; theme, fonts, the stories manifest
- [x] T45b Site · Iso: isometric line miniatures on a canvas, no perspective, hidden lines removed, cells growing on a sixteenth-note marimba beat, playable under the pointer, light and dark, still under reduced motion
- [x] T45c Site · Home: a full-screen hero and full-screen sections, each a miniature beside its live component, counts, the chapters as a city of plinths, a closing call; motion from scroll and route changes
- [x] T45d Site · Components: the chapter index and the story page redesigned; live stories without frames, native captures for what a browser cannot run, search, keys
- [x] T45e Site · Local: the gallery bundle built for localhost and the site served for the owner to see; not published
- [x] T46 E2E: Playwright against the built site
- [x] T46a Site · Owner's notes: the city title whole, header icons on one line, less space above a story, no visible scroll bars, each story named by its first component, every page laid out at phone, tablet and desktop widths
- [x] T47 Ship: GitHub repo (public), CI, Cloudflare deploy through `npx wrangler`
- [x] T48 Docs: a Docs tab after gpui-kit.com/docs, a rail of guides, an on-this-page list, previous and next; every claim checked against the code, every full Rust sample an example `scripts/check.sh` compiles and the page imports as text
- [x] T48a Docs · Frame: the Docs tab, `/docs/` and `/docs/$page/`, Markdown pages, the rail, the on-this-page list, previous and next, code blocks, light and dark, phone to desktop
- [x] T48b Docs · Introduction: what Ely is, the order to read, what is stable, a first example, license
- [x] T48c Docs · Installation: Rust, the git dependencies, gpui_platform's fonts, runtime shaders, the gallery as a check, what `init` refuses
- [x] T48d Docs · Getting started: an app from `init` to a window under a root `FocusScope`, with a Button
- [x] T48e Docs · Theme: modes, tokens through `cx.theme()`, high contrast, color-blind safe charts, an owner's palette
- [x] T48f Docs · Focus and keyboard: Tab and Shift-Tab, the root scope, tab stops, the focus ring, overlays that take and give back focus
- [x] T48g Docs · Motion: durations, reduced motion, springs, motion from a change
- [x] T48h Docs · Icons, fonts and assets: `Assets`, `IconName`, adding icons, the embedded fonts, file icons
- [x] T48i Docs · Overlays: what draws over the page, dialogs, popovers, toasts
- [x] T48j Docs · Internationalization: the `I18n` global, catalogs, right-to-left
- [x] T48k Docs · WebAssembly: the web gallery build, its URL, the host protocol, what stays native
- [x] T48l Docs · Testing: the `test-support` feature, frames, clocks and reduced motion in tests
- [x] T48m Docs · E2E: the Docs routes, rail, list, previous and next, and code blocks under Playwright
- [ ] T49 Acceptance: live URL and MVP checklist
- [x] T50a Rendering · Plan: chapter 44 in the PRD, `tasks/ch44.md`, these lines
- [x] T50b Rendering · Surface: `rendering::RenderSurface`, tested — `tasks/ch44.md`
- [x] T50c Rendering · Gallery: a Rendering page, its story, `stories.json`; the gallery runs on Linux
- [x] T50d Rendering · Web: the story runs live on the site from the wasm gallery, checked in a browser
- [x] T50e Rendering · Platforms: CI builds and tests the crate on Linux, macOS and Windows, arm64 and x86_64
- [x] T50f Rendering · Docs: a guide with an example `scripts/check.sh` compiles; AGENTS.md
- [x] T50g Rendering · Acceptance: the checklist, item by item
- [x] T51 Showcase · an apps page fed by a public repo — `tasks/showcase.md`
- [x] T52 Logo · an isometric mark for the header, favicon and OG image — `tasks/logo.md`

## T00 detail

- [x] git, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE`
- [x] `Cargo.toml`: gpui 0.2.2 with `runtime_shaders`, lints
- [x] Theme: light and dark palettes, high contrast, tokens, `ActiveTheme`, animated switch
- [x] Motion: durations, easings, spring
- [x] Assets: Lucide icons, Inter, JetBrains Mono, `AssetSource`
- [x] Icon, Button, IconButton
- [x] Gallery shell: chapter nav, theme toggle, self-capture
- [x] `AGENTS.md`, `README.md`

## Review log

Every round's verdict lives in `tasks/reviews.md`.
