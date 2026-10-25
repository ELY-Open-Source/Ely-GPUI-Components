# 0021 The web gallery on one thread

Date: 2026-09-29. Status: accepted.

The web gallery runs on one thread, since a static site sends no cross-origin isolation, and draws through WebGL2 where WebGPU is missing. Its pictures are served beside the wasm, not embedded: the wasm stays under Cloudflare's 25 MiB, and a story fetches only its own. `wasm-opt -Oz` takes the wasm from 23.0 to 20.0 MB; gzip and brotli stay near 7.9 and 5.5 MB.
