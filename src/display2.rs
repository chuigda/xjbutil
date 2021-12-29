//! Use this when you need a third display mode.
//!
//! //! It's strongly suggested that you should use `xjbutil::display2`
//!
//! ```
//! # use std::fmt::{Display, Formatter};
//! use xjbutil::display2;
//! use xjbutil::display2::Display2;
//!
//! #[derive(Debug)]
//! struct Foo {
//!     xyzzy: i32
//! }
//!
//! impl Display for Foo {
//!      // omitted
//! #    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//! #        write!(f, "Foo: xyzzy = {}", self.xyzzy)
//! #    }
//! }
//!
//! impl Display2 for Foo {
//!     fn fmt2(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
//!         write!(fmt, "I am Display2!")
//!     }
//! }
//!
//! # fn main() {
//! let foo = Foo { xyzzy: 42 };
//! println!("Debug: {:?}\nDisplay: {}\nDisplay2: {}", foo, foo, display2!(&foo));
//! # }
//! ```

use std::cell::{Ref, RefMut};
use std::fmt::{Display, Formatter};

pub trait Display2 {
    fn fmt2(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result;
}

impl<'a, T: Display2> Display2 for Ref<'a, T> {
    fn fmt2(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        Display2::fmt2(&**self, fmt)
    }
}

impl<'a, T: Display2> Display2 for RefMut<'a, T> {
    fn fmt2(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        Display2::fmt2(&**self, fmt)
    }
}

pub struct Display2Wrapper<'a, T: Display2> {
    pub value: &'a T
}

impl<'a, T: Display2> Display2Wrapper<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<'a, T: Display2> Display for Display2Wrapper<'a, T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.fmt2(fmt)
    }
}

pub trait ToString2 {
    fn to_string2(&self) -> String;
}

impl<T: Display2> ToString2 for T {
    fn to_string2(&self) -> String {
        format!("{}", Display2Wrapper::new(self))
    }
}
