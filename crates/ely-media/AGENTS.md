# AGENTS.md: ely-media

Rules for `ely-media`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- gpui paints pictures without a transform, so a picture turns by its pixels: `media::turn` turns decoded frames by quarter turns off the main thread, an asset keyed by the picture and the turn. gpui's loader turns a photo upright by its EXIF orientation first, so `media::ImageViewer` turns from the upright frame and `media::ImageUpload` measures that frame. An svg turns to any angle through its `Transformation`, so a control that turns shows one (`interaction::Rotatable`).
- A tip over a track splits the free width around it by the pointer's share: it stays inside the track and under the pointer, and nothing is measured (`media::scrubber::time_tip`, for the scrubber and the waveform).
