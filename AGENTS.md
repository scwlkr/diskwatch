# diskwatch

A tiny Rust CLI that prints one line about the main disk, so coding agents stay
mindful of the space their work uses. It is not a monitor: it glances once and exits.

## Rules for this repo

- Ship over test. No test suite, no test CI, no quality gates. If it breaks in
  real use, the agent that hit it files an issue here and it gets fixed.
- Zero dependencies. Every kilobyte of the binary matters; build with size-first
  release settings.
- Never scan folders. One free-space query to the OS, nothing else.
- Nothing stays running. No daemon, config file, history, JSON or self-update.
- Nobody builds by hand. GitHub Actions compiles release binaries for macOS
  (arm64), Linux (x86_64, arm64) and Windows (x86_64) on every push to main;
  machines download the finished binary. The build is a copier, not a gate.

## Output

    💾 65.4 GiB free (15%) · this session: used 0.3 GiB

- "freed" when free space went up; the second half is dropped when unchanged
  or when there is no starting number.
- Under 10% free: starts with ⚠️ and ends with "stop and tell Shane".
- The change is whole-machine since the session started, so it includes other
  threads' work. Agents clean up only what their own task created.
