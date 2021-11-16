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

#[cfg(feature = "display2")]
#[macro_export] macro_rules! display2 {
    ($input:expr) => {
        $crate::display2::Display2Wrapper::new($input)
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_boxed_slice() {
        let boxed_slice: Box<[i32]> = boxed_slice![1, 2, 3, 4];
        assert_eq!(boxed_slice.len(), 4);
        assert_eq!(boxed_slice[0], 1);
        assert_eq!(boxed_slice[3], 4);

        let another_boxed_slice: Box<[i32]> = boxed_slice![];
        assert_eq!(another_boxed_slice.len(), 0);
    }
}

#[cfg(all(feature = "display2", test))]
mod test_display2 {
    use std::fmt::Formatter;
    use crate::display2::Display2;

    #[test]
    fn test_display2() {
        struct S { a: i32, b: i32 }

        impl Display2 for S {
            fn fmt2(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Display2 S({}, {})", self.a, self.b)
            }
        }

        println!("{}", display2!(&S {a: 1, b: 2}))
    }
}

#[cfg(all(feature = "defer", test))]
mod test_defer {
    #[test]
    fn test_defer() {
        let x = "114".to_string();
        let y = "1919810".to_string();
        let z = "1919810".to_string();

        defer!(|| {
            eprintln!("happy birthday!");
        });

        defer!(|(x, y)| {
            assert_eq!(x, "114514");
            assert_eq!(y, "1919810");
        }, x, y);

        defer!(|z| {
            assert_eq!(z, "893");
        }, z);

        *z = "893".into();
        x.push_str("514");
    }
}
