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

pub struct Defer2<FN, CAPTURE>
    where FN: FnOnce(CAPTURE) + Send,
          CAPTURE: Send
{
    f: UncheckedOption<FN>,
    capt: UncheckedOption<CAPTURE>
}

impl<FN, CAPTURE> Defer2<FN, CAPTURE>
    where FN: FnOnce(CAPTURE) + Send,
          CAPTURE: Send
{
    pub fn new(f: FN, capt: CAPTURE) -> Self {
        Self {
            f: UncheckedOption::new(f),
            capt: UncheckedOption::new(capt)
        }
    }

    pub fn captured(&mut self) -> &mut CAPTURE {
        unsafe { self.capt.get_mut() }
    }
}

impl<FN, CAPTURE> Drop for Defer2<FN, CAPTURE>
    where FN: FnOnce(CAPTURE) + Send,
          CAPTURE: Send
{
    fn drop(&mut self) {
        let f: FN = unsafe { self.f.take() };
        let cap: CAPTURE = unsafe { self.capt.take() };
        (f)(cap);
    }
}
