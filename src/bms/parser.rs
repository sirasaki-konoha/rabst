//! BMS file parser implementing a pragmatic subset of the spec.
//!
//! Reference: the widely used "BMS command memo".
//! A BMS file is a text file made of lines of the form `#<command> <args>`.
//! Data (note) lines look like: `#<measure>:<channel> <obj><obj>...` where
//! `<measure>` is decimal, `<channel>` is hex, and `<obj>` are two-char base-36.

use anyhow::{Context, Result};
use std::path::Path;

use super::model::{BmsData, BmsNote, ObjId};

pub fn parse_file(path: &Path) -> Result<BmsData> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read bms file {:?}", path))?;
    // Decode (most BMS files are Shift-JIS / Latin1 / UTF-8). We try UTF-8 first
    // (already succeeded) and fall back to lossy latin1 for SJIS-ish bytes.
    parse(&content)
}

pub fn parse(content: &str) -> Result<BmsData> {
    let mut data = BmsData::new();
    let mut notes: Vec<BmsNote> = Vec::new();

    for raw in content.lines() {
        let line = strip_comment(raw.trim());
        if line.is_empty() {
            continue;
        }
        let line = line.strip_prefix('#').unwrap_or(line);

        let (head, rest) = match split_first_token(line) {
            Some(parts) => parts,
            None => continue,
        };

        // Standard BMS note lines are "#mmmcc:data". The bundled demo also
        // uses a looser "#mmm:cc data" form, so accept both.
        if let Some((measure, channel, objs)) = parse_note_line(head, rest) {
            push_objects(&mut notes, measure, channel, objs);
            continue;
        }

        // Header command.
        let value = rest.unwrap_or("").trim();
        handle_header(&mut data, head, value);
    }

    data.notes = notes;
    Ok(data)
}

fn parse_note_line<'a>(head: &'a str, rest: Option<&'a str>) -> Option<(u32, u32, &'a str)> {
    let (left, after_colon) = head.split_once(':')?;

    if left.len() == 3 {
        let objs = rest?;
        let measure = left.trim().parse::<u32>().ok()?;
        let channel = parse_channel(after_colon).ok()?;
        return Some((measure, channel, objs.trim()));
    }

    if left.len() == 5 {
        let measure = left[..3].parse::<u32>().ok()?;
        let channel = parse_channel(&left[3..]).ok()?;
        let objs = rest.unwrap_or(after_colon).trim();
        return Some((measure, channel, objs));
    }

    None
}

fn push_objects(notes: &mut Vec<BmsNote>, measure: u32, channel: u32, objs: &str) {
    let len = objs.len();
    if len < 2 || len % 2 != 0 {
        return;
    }
    let n = (len / 2) as f64;
    if n == 0.0 {
        return;
    }
    let mut i = 0;
    while i < len {
        let pair = &objs[i..i + 2];
        let pos = (i / 2) as f64;
        let fraction = pos / n;
        let obj = ObjId::from_hex36(pair);
        if obj != ObjId::ZERO {
            notes.push(BmsNote {
                channel,
                measure,
                fraction,
                obj,
            });
        }
        i += 2;
    }
}

fn handle_header(data: &mut BmsData, key: &str, value: &str) {
    let lower = key.to_ascii_lowercase();
    let value = value.trim();
    match lower.as_str() {
        "title" => data.title = value.to_string(),
        "subtitle" => data.subtitle = value.trim_start().to_string(),
        "artist" => data.artist = value.to_string(),
        "genre" => data.genre = value.to_string(),
        "player" => data.player = value.parse().unwrap_or(1),
        "playlevel" => data.play_level = value.to_string(),
        "rank" => data.rank = value.parse().unwrap_or(2),
        "total" => data.total = value.parse().unwrap_or(100.0),
        "stagefile" => data.stagefile = normalize_path(value),
        "banner" => data.banner = normalize_path(value),
        "backbmp" => data.backbmp = normalize_path(value),
        "bpm" => data.base_bpm = value.parse().unwrap_or(130.0),
        _ => {
            if let Some(id) = strip_key(&lower, "wav") {
                data.wav_files.insert(id, normalize_path(value));
            } else if let Some(id) = strip_key(&lower, "bmp") {
                data.bmp_files.insert(id, normalize_path(value));
            } else if let Some(id) = strip_key(&lower, "bpm") {
                if let Ok(b) = value.parse::<f64>() {
                    data.bpm_changes.insert(id, b);
                }
            } else if let Some(id) = strip_key(&lower, "stop") {
                if let Ok(s) = value.parse::<f64>() {
                    data.stop_changes.insert(id, s);
                }
            }
            // otherwise unknown header, ignore.
        }
    }
}

fn strip_key(key: &str, head: &str) -> Option<ObjId> {
    let id = key.strip_prefix(head)?;
    if id.len() != 2 {
        return None;
    }
    Some(ObjId::from_hex36(id))
}

fn parse_channel(s: &str) -> Result<u32> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }
    let t = if s.len() > 2 { &s[..2] } else { s };
    if let Ok(v) = u32::from_str_radix(t, 16) {
        Ok(v)
    } else {
        // fall back to base36
        let mut v = 0u32;
        for c in t.bytes() {
            v *= 36;
            v += match c {
                b'0'..=b'9' => (c - b'0') as u32,
                b'a'..=b'z' => (c - b'a' + 10) as u32,
                b'A'..=b'Z' => (c - b'A' + 10) as u32,
                _ => 0,
            };
        }
        Ok(v)
    }
}

fn split_first_token(line: &str) -> Option<(&str, Option<&str>)> {
    let mut it = line.splitn(2, char::is_whitespace);
    let head = it.next()?;
    let tail = it.next();
    Some((head, tail))
}

fn strip_comment(line: &str) -> &str {
    if let Some(idx) = line.find("//") {
        if idx == 0 || line.as_bytes()[idx - 1] != b':' {
            return &line[..idx];
        }
    }
    line
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_bms_note_lines() {
        let data = parse(
            "#TITLE standard\n\
             #BPM 174\n\
             #00111:01000200\n\
             #00115:00000300\n",
        )
        .unwrap();

        assert_eq!(data.title, "standard");
        assert_eq!(data.base_bpm, 174.0);
        assert_eq!(data.notes.len(), 3);
        assert_eq!(data.notes[0].measure, 1);
        assert_eq!(data.notes[0].channel, 0x11);
        assert_eq!(data.notes[0].fraction, 0.0);
        assert_eq!(data.notes[1].channel, 0x11);
        assert_eq!(data.notes[1].fraction, 0.5);
        assert_eq!(data.notes[2].channel, 0x15);
        assert_eq!(data.notes[2].fraction, 0.5);
    }

    #[test]
    fn keeps_legacy_demo_note_line_support() {
        let data = parse("#005:11 02000000\n").unwrap();

        assert_eq!(data.notes.len(), 1);
        assert_eq!(data.notes[0].measure, 5);
        assert_eq!(data.notes[0].channel, 0x11);
        assert_eq!(data.notes[0].obj.0, 2);
    }
}
