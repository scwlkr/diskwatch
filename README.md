# diskwatch

`diskwatch` prints how much space is free on the main disk and how much that
changed during the current agent session.

```text
💾 65.4 GiB free (15%) · this session: used 0.3 GiB
```

Download the binary from the [latest release](https://github.com/scwlkr/diskwatch/releases/tag/latest).

| Platform | Asset |
| --- | --- |
| macOS (Apple silicon) | `diskwatch-macos-arm64` |
| Linux (x86-64) | `diskwatch-linux-x86_64` |
| Linux (ARM64) | `diskwatch-linux-arm64` |
| Windows (x86-64) | `diskwatch-windows-x86_64.exe` |

On macOS and Linux, save it as `~/.local/bin/diskwatch` and run
`chmod +x ~/.local/bin/diskwatch`. On Windows, save it as `diskwatch.exe` in a
folder on your `PATH`.

- `diskwatch` prints the disk status. The first call records the starting point
  when a session ID is available.
- `diskwatch --session <id>` supplies a session ID explicitly.
- `diskwatch --hook` reads the session-start payload from standard input and
  records the starting point silently.

Broke? [File an issue here](https://github.com/scwlkr/diskwatch/issues/new).
