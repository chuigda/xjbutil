#[cfg(all(
    any(
        feature = "async-astd",
        feature = "async-monoio",
        feature = "async-pollster",
        feature = "async-tokio"
    ),
    not(feature = "async")
))]
compile_error!(
    "appointing concrete async implementation without appointing `async` feature is meaningless"
);

mod rand_intern;
mod mem_intern;
mod unchecked_intern;

#[cfg(feature = "minhttpd")] mod http_commons;

#[cfg(all(
    feature = "async",
    any(
        feature = "async-astd",
        feature = "async-monoio",
        feature = "async-pollster",
        feature = "async-tokio",
    )
))]
pub mod async_utils;
#[cfg(feature = "defer")]          pub mod defer;
#[cfg(feature = "display2")]       pub mod display2;
#[cfg(feature = "either")]         pub mod either;
#[cfg(feature = "flexible-array")] pub mod flex;
#[cfg(feature = "korobka")]        pub mod korobka;
#[cfg(feature = "liberty")]        pub mod liberty;
#[cfg(feature = "makro")]          pub mod makro;
#[cfg(feature = "minhttpd")]       pub mod minhttpd;
#[cfg(feature = "slice-arena")]    pub mod slice_arena;
#[cfg(feature = "std-ext")]        pub mod std_ext;
#[cfg(feature = "unchecked")]      pub mod unchecked;
#[cfg(feature = "value")]          pub mod value;
#[cfg(feature = "void")]           pub mod void;
#[cfg(feature = "wide_ptr")]       pub mod wide_ptr;
#[cfg(feature = "zvec")]           pub mod zvec;

#[cfg(feature = "rand")]           pub mod rand { pub use crate::rand_intern::*; }
#[cfg(feature = "mem")]            pub mod mem { pub use crate::mem_intern::*; }
