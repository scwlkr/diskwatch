use std::borrow::Cow;
use std::env;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Read, Write};

pub fn environment_id() -> Option<String> {
    ["DISKWATCH_SESSION", "CLAUDE_CODE_SESSION_ID", "CODEX_THREAD_ID", "CODEX_SESSION_ID"]
        .into_iter()
        .find_map(|name| env::var(name).ok().filter(|id| !id.is_empty()))
}

pub fn hook() {
    let mut input = String::new();
    let id = io::stdin()
        .lock()
        .read_to_string(&mut input)
        .ok()
        .and_then(|_| hook_id(&input))
        .filter(|id| !id.is_empty())
        .or_else(environment_id);

    let Some(id) = id else { return };
    if let Ok(disk) = crate::disk::query() {
        let _ = baseline(&id, disk.free);
    }
}

pub fn baseline(id: &str, free: u64) -> io::Result<Option<u64>> {
    let dir = env::temp_dir().join("diskwatch");
    let name = filename(id);
    let path = dir.join(name.as_ref());

    let stored = match fs::read_to_string(&path) {
        Ok(stored) => stored,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut builder = DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(dir)?;

            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    if let Err(error) = file.write_all(free.to_string().as_bytes()) {
                        drop(file);
                        let _ = fs::remove_file(path);
                        return Err(error);
                    }
                    return Ok(None);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    fs::read_to_string(&path)?
                }
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    };

    // Another invocation may have just created the file and still be writing it.
    if stored.is_empty() {
        return Ok(None);
    }
    stored
        .trim()
        .parse()
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn filename(id: &str) -> Cow<'_, str> {
    let device_name = matches!(id, "aux" | "con" | "nul" | "prn")
        || matches!(
            id.as_bytes(),
            [b'c', b'o', b'm', b'0'..=b'9'] | [b'l', b'p', b't', b'0'..=b'9']
        );

    // Preserve ordinary agent IDs and reserve the double hyphen for encoded IDs.
    // Lowercase names also keep distinct IDs apart on case-insensitive filesystems.
    if id.len() <= 100
        && !id.starts_with("--")
        && !device_name
        && id.bytes().all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
    {
        return Cow::Borrowed(id);
    }
    if id.len() <= 48 {
        use std::fmt::Write as _;
        let mut encoded = String::from("--x");
        for byte in id.bytes() {
            let _ = write!(encoded, "{byte:02x}");
        }
        return Cow::Owned(encoded);
    }

    // Use stable FNV-1a for IDs too long to encode within the filename limit.
    const OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let hash = id.bytes().fold(OFFSET, |hash, byte| (hash ^ u128::from(byte)).wrapping_mul(PRIME));
    Cow::Owned(format!("--h{hash:032x}"))
}

fn hook_id(input: &str) -> Option<String> {
    let input = input.trim_start().strip_prefix('{')?;
    let mut chars = input.char_indices();
    let mut depth = 0;
    while let Some((start, ch)) = chars.next() {
        match ch {
            '"' => {
                let end = string_end(&mut chars)?;
                if depth == 0 {
                    if let Some(value) = input[end + 1..].trim_start().strip_prefix(':') {
                        if decode_string(&input[start + 1..end]).as_deref() == Some("session_id") {
                            let value = value.trim_start().strip_prefix('"')?;
                            let end = string_end(&mut value.char_indices())?;
                            return decode_string(&value[..end]);
                        }
                    }
                }
            }
            '{' | '[' => depth += 1,
            '}' if depth == 0 => return None,
            '}' | ']' if depth > 0 => depth -= 1,
            _ => {}
        }
    }
    None
}

fn string_end(chars: &mut std::str::CharIndices<'_>) -> Option<usize> {
    while let Some((index, ch)) = chars.next() {
        match ch {
            '"' => return Some(index),
            '\\' => {
                chars.next()?;
            }
            _ => {}
        }
    }
    None
}

fn decode_string(raw: &str) -> Option<String> {
    let mut decoded = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        let value = match ch {
            '\\' => match chars.next()? {
                ch @ ('"' | '\\' | '/') => ch,
                'b' => '\x08',
                'f' => '\x0c',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    let high = hex_unit(&mut chars)?;
                    if (0xd800..=0xdbff).contains(&high) {
                        if chars.next()? != '\\' || chars.next()? != 'u' {
                            return None;
                        }
                        let low = hex_unit(&mut chars)?;
                        char::decode_utf16([high, low]).next()?.ok()?
                    } else {
                        char::from_u32(u32::from(high))?
                    }
                }
                _ => return None,
            },
            ch if ch < ' ' => return None,
            ch => ch,
        };
        decoded.push(value);
    }
    Some(decoded)
}

fn hex_unit(chars: &mut std::str::Chars<'_>) -> Option<u16> {
    (0..4).try_fold(0, |value, _| Some(value * 16 + chars.next()?.to_digit(16)? as u16))
}
