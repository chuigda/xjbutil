//! RAII structure used for deferring execution

use crate::unchecked_intern::UncheckedOption;

pub struct Defer<F>
    where F: FnOnce() + Send
{
    f: UncheckedOption<F>
}

impl<F: FnOnce() + Send> Defer<F> {
    pub fn new(f: F) -> Self {
        Self { f: UncheckedOption::new(f) }
    }
}

impl<F: FnOnce() + Send> Drop for Defer<F> {
    fn drop(&mut self) {
        let f: F = unsafe { self.f.take() };
        (f)()
    }
}

pub struct Defer2<FN, CAP>
    where FN: FnOnce(CAP) + Send,
          CAP: Send
{
    f: UncheckedOption<FN>,
    cap: UncheckedOption<CAP>
}

impl<FN, CAP> Defer2<FN, CAP>
    where FN: FnOnce(CAP) + Send,
          CAP: Send
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
    where FN: FnOnce(CAP) + Send,
          CAP: Send
{
    fn drop(&mut self) {
        let f: FN = unsafe { self.f.take() };
        let cap: CAP = unsafe { self.cap.take() };
        (f)(cap);
    }
}
