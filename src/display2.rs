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
