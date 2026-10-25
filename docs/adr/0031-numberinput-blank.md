# 0031 NumberInput::blank

Date: 2026-10-05. Status: accepted.

`NumberInput::blank` shows a placeholder, not a number, as for a mixed selection; Enter or blur on the empty field commits nothing, a typed number commits as usual. Before, a mixed field held a real number the owner had to invent, and Enter sent it. `ScrubInput` moved to `number/scrub.rs`.
