mod disk;
mod session;

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

const USAGE: &str = "Usage: diskwatch [--session <id>]\n       diskwatch --hook | --help | --version";

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    match (args.next(), args.next(), args.next()) {
        (None, None, None) => show(None),
        (Some(flag), Some(id), None) if flag == "--session" => match id.to_str() {
            Some(id) if !id.is_empty() => show(Some(id)),
            _ => usage_error(),
        },
        (Some(flag), None, None) => {
            match flag.to_str() {
                Some("--hook") => session::hook(),
                Some("--help" | "-h") => {
                    let _ = writeln!(io::stdout().lock(), "{USAGE}");
                }
                Some("--version" | "-V") => {
                    let _ = writeln!(io::stdout().lock(), "diskwatch {}", env!("CARGO_PKG_VERSION"));
                }
                _ => return usage_error(),
            }
            ExitCode::SUCCESS
        }
        _ => usage_error(),
    }
}

fn usage_error() -> ExitCode {
    let _ = writeln!(io::stderr().lock(), "{USAGE}");
    ExitCode::from(2)
}

fn show(id: Option<&str>) -> ExitCode {
    let mut stdout = io::stdout().lock();
    match status_line(id) {
        Ok(line) => {
            let _ = writeln!(stdout, "{line}");
        }
        Err(reason) => {
            let _ = writeln!(stdout, "💾 diskwatch: {reason}");
        }
    }
    ExitCode::SUCCESS
}

fn status_line(id: Option<&str>) -> Result<String, &'static str> {
    let disk = disk::query()?;
    if disk.total == 0 {
        return Err("disk reports zero capacity");
    }

    let percent = u128::from(disk.free) * 100 / u128::from(disk.total);
    let warning = if percent < 10 { "⚠️ " } else { "" };
    let free = gib_tenths(disk.free);
    let mut line = format!("{warning}💾 {}.{} GiB free ({percent}%)", free / 10, free % 10);

    let environment_id = if id.is_none() { session::environment_id() } else { None };
    if let Some(id) = id.or(environment_id.as_deref()) {
        let baseline = session::baseline(id, disk.free).map_err(|_| "session baseline unavailable")?;
        if let Some(baseline) = baseline {
            let change = gib_tenths(disk.free.abs_diff(baseline));
            if change != 0 {
                use std::fmt::Write as _;
                let action = if disk.free > baseline { "freed" } else { "used" };
                let _ = write!(line, " · this session: {action} {}.{} GiB", change / 10, change % 10);
            }
        }
    }

    if percent < 10 {
        line.push_str(" · stop and tell Shane");
    }
    Ok(line)
}

fn gib_tenths(bytes: u64) -> u64 {
    const GIB: u128 = 1 << 30;
    ((u128::from(bytes) * 10 + GIB / 2) / GIB) as u64
}
