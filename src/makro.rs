//! Some potentially useful macros

#[cfg(feature = "std-ext")]
#[macro_export] macro_rules! boxed_slice {
    () => {
        vec![].into_boxed_slice()
    };
    ($($x:expr),+ $(,)?) => {
        vec![$($x),+].into_boxed_slice()
    };
}

#[cfg(feature = "defer")]
#[macro_export] macro_rules! defer {
    ($func:expr) => {
        #[allow(unused_variables)]
        let deferred: $crate::defer::Defer<_> =
            $crate::defer::Defer::new($func);
    };
    ($func:expr, $capt:ident) => {
        #[allow(unused_variables)]
        let mut deferred: $crate::defer::Defer2<_, _> =
            $crate::defer::Defer2::new($func, $capt);
        #[allow(unused_variables)]
        let $capt = deferred.captured();
    };
    ($func:expr, $($capt:ident),*) => {
        #[allow(unused_variables)]
        let mut deferred: $crate::defer::Defer2<_, _> =
            $crate::defer::Defer2::new($func, ($($capt),*));
        #[allow(unused_variables)]
        let ($($capt),*) = deferred.captured();
    };
}

#[cfg(all(feature = "defer", test))]
mod test {
    #[test]
    fn test_defer() {
        let x = "114".to_string();
        let y = "514".to_string();
        let z = "1919810".to_string();

        defer!(
            |(x, y)| eprintln!("first defer: {}{}", x, y),
            x, y
        );
        defer!(
            |(x, z)| eprintln!("second defer: {}{}", x, z),
            x, z
        );
        defer!(|| eprintln!("1919810"));

        x.push_str(", ");
    }
}
