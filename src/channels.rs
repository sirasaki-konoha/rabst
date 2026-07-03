//! Shared constants: BMS channel -> physical lane mapping used by the
//! gameplay renderer and judgment system.

/// Beatmania-style 7-key + scratch lanes ordering (display left -> right):
///   Scratch, K1, K2, K3, K4, K5, K6, K7
pub const NUM_LANES: usize = 8;

/// Channel -> lane index. Returns None for non-player-visible channels.
pub fn lane_for_channel(ch: u32) -> Option<usize> {
    match ch {
        0x11 => Some(1), // K1
        0x12 => Some(2), // K2
        0x13 => Some(3), // K3
        0x14 => Some(4), // K4
        0x15 => Some(5), // K5
        0x16 => Some(6), // K6
        0x18 => Some(7), // K7
        0x19 => Some(0), // Scratch
        // 1C..1F Free zone -> ignore
        _ => None,
    }
}

pub const CHANNEL_BGM: u32 = 0x01;
pub const CHANNEL_BGA_BASE: u32 = 0x04;
pub const CHANNEL_BGA_LAYER: u32 = 0x07;
pub const CHANNEL_BGA_MISS: u32 = 0x06;
pub const CHANNEL_BPM_CHANGE_EXT: u32 = 0x08;
pub const CHANNEL_STOP: u32 = 0x09;
pub const CHANNEL_BPM_CHANGE: u32 = 0x03;
