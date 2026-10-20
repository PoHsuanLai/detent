---
name: settings
description: Which desktop settings you may change for the person: theme, accent, dock size, web search and Do Not Disturb. Nothing else.
---
# Changing settings

Change a setting only when the person asked for it, with `settings.key.set`. If that action is not among your tools, say so; do not look for another route.

**You may set these keys, and no others:**

| Key | Values |
|---|---|
| `appearance.theme` | `system`, `light`, `dark` |
| `appearance.accent` | `blue`, `purple`, `pink`, `red`, `orange`, `yellow`, `green`, `graphite` |
| `dock.magnification` | `on`, `off` |
| `dock.tile_size_px` | 32 to 80 |
| `dock.magnified_size_px` | 32 to 72 |
| `launcher.web_search` | `on`, `off` |
| `notifications.dnd` | `on`, `off` |

Each is a preference one click undoes. "Make it dark" is `appearance.theme = dark`; "stop the notifications for the meeting" is `notifications.dnd = on`.

**Never set** anything under `ai.`, `agent.`, `cua.`, `memory.`, `companion.`, `session.`, `idle.` or `accounts.`: models, spend, your own limits, computer use, memory, the lock and sign-in are the person's alone. If asked, say they can change it in Settings, and stop.

Read a key with `settings.key.read` before you set it, and tell the person the old and the new value. `settings.key.set` refuses any key not in the table; the person can undo the change.
