#[cfg(target_arch = "wasm32")]
compile_error!(r#"You fucking WebAssembly kiddies go fuck yourself

Just in order to be compatible with your fucking browser platform, the crate winit had to
change their very conventional poll-based API to a fucking state machine which is completely
anti-Rust pattern.

And when I was developing a gltf conversion program, a fucking wasm-bindgen crate
unexpectedfully occurred in my dependency list and fuckingly requires fucking nightly Rust.
And when I see cargo tree that's in my depedency's dependency's dependency's dependency.

So fuck you webassembly kiddes. Curse you for your fucking platform. Go to the hell."#);

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

#[cfg(all(feature = "provenance", not(feature = "makro")))]
compile_error!(
    "enabling `provenance` feature without `makro` feature is meaningless"
);

#[cfg(all(feature = "strict-sound", feature = "wide_ptr"))]
compile_error!(
    "`wide_ptr` feature is actually UB-rich, it cannot be used soundly"
);

mod mem_intern;
mod rand_intern;
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
#[cfg(feature = "typed-arena")]    pub mod typed_arena;
#[cfg(feature = "slice-arena")]    pub mod slice_arena;
#[cfg(feature = "std-ext")]        pub mod std_ext;
#[cfg(feature = "unchecked")]      pub mod unchecked;
#[cfg(feature = "value")]          pub mod value;
#[cfg(feature = "void")]           pub mod void;
#[cfg(feature = "wide_ptr")]       pub mod wide_ptr;
#[cfg(feature = "zvec")]           pub mod zvec;

#[cfg(feature = "rand")]           pub mod rand { pub use crate::rand_intern::*; }
#[cfg(feature = "mem")]            pub mod mem { pub use crate::mem_intern::*; }
