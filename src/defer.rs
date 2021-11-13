//! RAII structure used for deferring execution

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
