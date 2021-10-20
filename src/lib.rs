#[cfg(all(
    any(feature = "async-astd", feature = "async-pollster", feature = "async-tokio"),
    not(feature = "async")
))]
compile_error!(
    "appointing concrete async implementation without appointing `async` feature is meaningless"
);

#[cfg(all(
    feature = "async",
    not(any(feature = "async-astd", feature = "async-pollster", feature = "async-tokio"))
))]
compile_error!(
    "appointing `async` feature without choosing concrete async implementation is meaningless"
);

mod mem_intern;

#[cfg(feature = "async")]     pub mod async_utils;
#[cfg(feature = "defer")]     pub mod defer;
#[cfg(feature = "either")]    pub mod either;
#[cfg(feature = "fat_ptr")]   pub mod fat_ptr;
#[cfg(feature = "korobka")]   pub mod korobka;
#[cfg(feature = "makro")]     pub mod makro;
#[cfg(feature = "std-ext")]   pub mod std_ext;
#[cfg(feature = "unchecked")] pub mod unchecked;
#[cfg(feature = "void")]      pub mod void;
#[cfg(feature = "zvec")]      pub mod zvec;

#[cfg(feature = "mem")]
pub mod mem { pub use crate::mem_intern::*; }
