use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::atomic::Ordering::SeqCst;
use std::time::{SystemTime, UNIX_EPOCH};

static SEED: AtomicU64 = AtomicU64::new(0x5bd1e995);
static INIT: AtomicBool = AtomicBool::new(false);

pub fn random() -> u64 {
    let _ = INIT.fetch_update(SeqCst, SeqCst, |init| {
        if !init {
            let _ = SEED.fetch_update(SeqCst, SeqCst, |seed| {
                let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                Some(seed.wrapping_mul(secs))
            });
        }
        Some(true)
    });

    SEED.fetch_update(
        SeqCst,
        SeqCst,
        |seed| Some(seed.wrapping_mul(19260817).wrapping_add(19660813))
    ).unwrap()
}

pub fn random_string(count: usize) -> String {
    let mut ret = Vec::with_capacity(count);
    for _ in 0..count {
        match random() % 3 {
            0 => ret.push('a' as u8 + (random() % 26) as u8),
            1 => ret.push('A' as u8 + (random() % 26) as u8),
            2 => ret.push('0' as u8 + (random() % 10) as u8),
            _ => unreachable!(),
        }
    }
    String::from_utf8(ret).unwrap()
}

pub fn random_string_lossy(count: usize) -> String {
    let mut ret = Vec::with_capacity(count);
    for _ in 0..count {
        ret.push((random() % 95 + 31) as u8);
    }
    String::from_utf8_lossy(&ret).to_string()
}
