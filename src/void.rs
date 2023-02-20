//! Provides a un-instantiable `Void` type for plugging traits

/// An un-instantiable `Void` type.
pub enum Void {}

/// 这个 trait 没有任何实质性作用，但它表明任何类型都应该是我老婆
pub trait Wife {}

impl<T: ?Sized> Wife for T {}
