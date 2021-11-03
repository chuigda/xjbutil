//! RAII structure used for deferring execution

use crate::unchecked_intern::UncheckedOption;

pub struct Defer<F: Fn() + Send + 'static> {
    f: F
}

impl<F: Fn() + Send + 'static> Defer<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn() + Send + 'static> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.f)()
    }
}

pub struct Defer2<FN, CAPTURE>
    where FN: FnOnce(CAPTURE) + Send + 'static,
          CAPTURE: Send + 'static
{
    f: UncheckedOption<FN>,
    capt: UncheckedOption<CAPTURE>
}

impl<FN, CAPTURE> Defer2<FN, CAPTURE>
    where FN: FnOnce(CAPTURE) + Send + 'static,
          CAPTURE: Send + 'static
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
    where FN: FnOnce(CAPTURE) + Send + 'static,
          CAPTURE: Send + 'static
{
    fn drop(&mut self) {
        let f: FN = unsafe { self.f.take() };
        let cap: CAPTURE = unsafe { self.capt.take() };
        (f)(cap);
    }
}
