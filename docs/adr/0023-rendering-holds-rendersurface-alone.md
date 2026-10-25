# 0023 Rendering holds RenderSurface alone

Date: 2026-10-04. Status: accepted.

Chapter 44, Rendering, holds `RenderSurface` alone. gpui at 1a28cff paints `RenderImage`s everywhere, `CVPixelBuffer`s (NV12) on macOS's Metal renderer alone, and shares no device or shader hook, so shader, camera and GPU views are the host's, feeding the surface, and zero-copy or Metal, OpenGL, Vulkan and Direct3D surfaces are blocked (`tasks/ch44.md`). CI tests the crate and builds the gallery on Linux, Windows and macOS, arm64 and x86_64.
