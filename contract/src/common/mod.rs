use crate::*;

mod asserts;

pub(crate) fn unix_timestamp(ms: u64) -> UnixTimestamp {
    (ms / 1000) as u32
}
