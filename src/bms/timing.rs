//! Timing computation: convert (measure, fraction) positions into wall-clock
//! seconds, accounting for `#BPMxx` change events and `#STOPxx` stop events.

use super::model::BmsData;

/// A precomputed timing table that maps every note to an absolute time in
/// seconds from song start.
#[derive(Debug, Clone)]
pub struct ChartTiming {
    /// (note index) -> seconds from song start.
    pub note_times: Vec<f64>,
    /// Sequence of BPM changes: (seconds, new_bpm).
    pub bpm_timeline: Vec<(f64, f64)>,
    pub total_seconds: f64,
}

impl ChartTiming {
    pub fn build(data: &BmsData) -> ChartTiming {
        // Determine, per measure *in order of appearance*, the BPM and the
        // length-of-measure (default = 4 beats). For this implementation we
        // assume 4/4 throughout (the common case) but support BPM changes and
        // STOP events at object granularity.

        // Gather all "events" sorted by position: BPM changes (channel 03 or
        // extended referent #BPMxx via channel 08), STOPs (channel 09), and
        // the regular notes (so their time gets computed alongside).
        let mut all_events: Vec<Event> = data
            .notes
            .iter()
            .enumerate()
            .map(|(i, n)| Event {
                measure: n.measure,
                fraction: n.fraction,
                kind: EventKind::Note(i),
            })
            .collect();

        // BPM change at channel 03 uses the hex value times BPMscale? Actually
        // channel 03 carries the BPM directly as a decimal in the obj id? No:
        // the obj id under channel 03 is interpreted as decimal BPM value if
        // under #BPMxx for channel 08 else the obj id encodes bpm? Per spec:
        // channel 03 -> the two-char hex object is the BPM (low-res, hex40=64..)
        // Wait actually the channel 03 carries a single base-36 hex that's the
        // bpm directly times... obsolete and rare. Channel 08 references #BPMxx.

        // Channel 08 (extended bpm change), channel 09 (stop), channel 03 (bpm)
        for n in data.notes.iter() {
            match n.channel {
                0x03 => {
                    let bpm = n.obj.0 as f64; // raw deprecated form
                    if (1.0_f64..).contains(&bpm) {
                        all_events.push(Event {
                            measure: n.measure,
                            fraction: n.fraction,
                            kind: EventKind::BpmChange(bpm),
                        });
                    }
                }
                0x08 => {
                    if let Some(&b) = data.bpm_changes.get(&n.obj) {
                        all_events.push(Event {
                            measure: n.measure,
                            fraction: n.fraction,
                            kind: EventKind::BpmChange(b),
                        });
                    }
                }
                0x09 => {
                    if let Some(&s) = data.stop_changes.get(&n.obj) {
                        all_events.push(Event {
                            measure: n.measure,
                            fraction: n.fraction,
                            kind: EventKind::Stop(s),
                        });
                    }
                }
                _ => {}
            }
        }

        all_events.sort_by(|a, b| {
            a.measure.cmp(&b.measure).then_with(|| {
                a.fraction
                    .partial_cmp(&b.fraction)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        let mut note_times = vec![0.0_f64; data.notes.len()];
        let mut bpm_timeline: Vec<(f64, f64)> = Vec::new();
        let mut current_bpm = data.base_bpm;
        let mut t = 0.0_f64;
        let mut last_measure = 0u32;
        let mut last_fraction = 0.0_f64;
        bpm_timeline.push((0.0, current_bpm));

        for ev in all_events {
            // advance time by the musical distance since the last event.
            if ev.measure >= last_measure {
                let beats_elapsed = (ev.measure as f64 - last_measure as f64) * 4.0
                    + (ev.fraction - last_fraction) * 4.0;
                // negative sample case -> avoid.
                let beats_elapsed = beats_elapsed.max(0.0);
                let beat_seconds = beat_seconds(beats_elapsed, current_bpm);
                t += beat_seconds;
            }
            last_measure = ev.measure;
            last_fraction = ev.fraction;

            match ev.kind {
                EventKind::Note(idx) => {
                    note_times[idx] = t;
                }
                EventKind::BpmChange(b) => {
                    current_bpm = b;
                    bpm_timeline.push((t, current_bpm));
                }
                EventKind::Stop(stop_16) => {
                    // a stop value of N stops for N/192 of a whole note? The
                    // standard: STOP value is in 1/16 notes? No, it's in
                    // 1/192 of a whole note (same default as beat resolution).
                    let stop_seconds = stop_16 / 192.0 * beat_seconds(4.0, current_bpm);
                    t += stop_seconds;
                }
            }
        }

        let total_seconds = note_times.iter().cloned().fold(0.0, f64::max) + 3.0;

        ChartTiming {
            note_times,
            bpm_timeline,
            total_seconds,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EventKind {
    Note(usize),
    BpmChange(f64),
    Stop(f64),
}
#[derive(Debug, Clone, Copy)]
struct Event {
    measure: u32,
    fraction: f64,
    kind: EventKind,
}

fn beat_seconds(beats: f64, bpm: f64) -> f64 {
    if bpm <= 0.0 {
        return beats;
    }
    (beats / bpm) * 60.0
}
