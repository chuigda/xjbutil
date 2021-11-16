#[cfg(all(
any(feature = "async-astd", feature = "async-pollster", feature = "async-tokio"),
not(feature = "async")
))]
compile_error!(
    "appointing concrete async implementation without appointing `async` feature is meaningless"
);

mod mem_intern;
mod unchecked_intern;

#[cfg(all(
    feature = "async",
    any(feature = "async-astd", feature = "async-pollster", feature = "async-tokio")))
] pub mod async_utils;
#[cfg(feature = "defer")]     pub mod defer;
#[cfg(feature = "display2")]  pub mod display2;
#[cfg(feature = "either")]    pub mod either;
#[cfg(feature = "korobka")]   pub mod korobka;
#[cfg(feature = "makro")]     pub mod makro;
#[cfg(feature = "std-ext")]   pub mod std_ext;
#[cfg(feature = "unchecked")] pub mod unchecked;
#[cfg(feature = "void")]      pub mod void;
#[cfg(feature = "wide_ptr")]  pub mod wide_ptr;
#[cfg(feature = "zvec")]      pub mod zvec;

#[cfg(feature = "mem")]
/// Some memory or pointer related operations
pub mod mem { pub use crate::mem_intern::*; }
