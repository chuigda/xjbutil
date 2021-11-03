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
    ($func:expr, $($capt:ident),*) => {
        #[allow(unused_variables)]
        let mut deferred: $crate::defer::Defer2<_, _> =
            $crate::defer::Defer2::new($func, ($($capt),*));
        let ($($capt),*) = deferred.captured();
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_defer2() {
        let x = "114".to_string();
        let y = "514".to_string();

        defer!(
            |(x, y): (String, String)| eprintln!("{}{}", x, y),
            x, y
        );
        defer!(|| eprintln!("1919810"));

        x.push_str(", ");
        eprintln!("{}{}", x, y);
    }
}
