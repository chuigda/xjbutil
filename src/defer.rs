//! RAII structure used for deferring execution
//!
//! It's strongly suggested that you should use macros `xjbutil::defer` and `xjbutil::defer2`.
//! These raw structures are considered hard to use.
//!
//! ```
//! use xjbutil::defer;
//!
//! fn main() {
//!     defer!(|| println!("defer1"));
//!
//!     let mut s1 = "Far from the galaxy".to_string();
//!     let mut s2 = "Is where your home lies".to_string();
//!
//!     defer!(|(s1, s2)| {
//!         assert_eq!(s1, "Save us from destruction the evil monsters");
//!         assert_eq!(s2, "With ultra-beam, spike!");
//!         println!("{} {}", s1, s2);
//!     }, s1, s2);
//!
//!     defer!(|mut s1| {
//!         assert_eq!(s1, "March to the end of the big milky way");
//!         *s1 = "Save us from destruction the evil monsters".to_string();
//!     }, s1);
//!
//!     assert_eq!(*s1, "Far from the galaxy");
//!     assert_eq!(s2, "Is where your home lies");
//!
//!     **s1 = "March to the end of the big milky way".to_string();
//!     *s2 = "With ultra-beam, spike!".to_string();
//! }
//! ```

use crate::unchecked_intern::UncheckedOption;

pub struct Defer<F>
    where F: FnOnce()
{
    f: UncheckedOption<F>
}

impl<F: FnOnce()> Defer<F> {
    pub fn new(f: F) -> Self {
        Self { f: UncheckedOption::new(f) }
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        let f: F = unsafe { self.f.take() };
        (f)()
    }
}

unsafe impl<F> Send for Defer<F> where F: FnOnce() + Send {}

pub struct Defer2<FN, CAP>
    where FN: FnOnce(CAP)
{
    f: UncheckedOption<FN>,
    cap: UncheckedOption<CAP>
}

impl<FN, CAP> Defer2<FN, CAP>
    where FN: FnOnce(CAP)
{
    pub fn new(f: FN, capt: CAP) -> Self {
        Self {
            f: UncheckedOption::new(f),
            cap: UncheckedOption::new(capt)
        }
    }

    pub fn captured(&mut self) -> &mut CAP {
        unsafe { self.cap.get_mut() }
    }
}

impl<FN, CAP> Drop for Defer2<FN, CAP>
    where FN: FnOnce(CAP)
{
    fn drop(&mut self) {
        let f: FN = unsafe { self.f.take() };
        let cap: CAP = unsafe { self.cap.take() };
        (f)(cap);
    }
}

unsafe impl<F, CAP> Send for Defer2<F, CAP> where F: FnOnce(CAP) + Send, CAP: Send {}
