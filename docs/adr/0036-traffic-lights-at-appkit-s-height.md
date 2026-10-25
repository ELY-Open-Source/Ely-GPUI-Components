# 0036 Traffic lights at AppKit's height

Date: 2026-10-20. Status: accepted.

The system traffic lights sit centered in the title bar at the height AppKit gives its window buttons, read through `NSWindow standardWindowButton:forStyleMask:` as a window opens: 16 tall on macOS 15, 14 on 26, so the fixed origin (14, 13) sat 2 points low on 15 and 1 on 26.
