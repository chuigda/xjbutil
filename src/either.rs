//! A simple `Either` structure
//!
//! This structure is in effect isomorphic with `Result`, but does not have certain traits
//! implemented. Personally I think this is the better choice for expressing something which is not
//! really a `Result`.

use std::fmt::{Debug, Formatter};

pub enum Either<T1, T2> {
    Left(T1),
    Right(T2)
}

impl<E1, E2> Debug for Either<E1, E2>
    where E1: Debug,
          E2: Debug
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Either::Left(left) => f.debug_tuple("Either::Left").field(left).finish(),
            Either::Right(right) => f.debug_tuple("Either::Right").field(right).finish()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::either::Either;

    #[test]
    fn test_either() {
        let e1: Either<i32, String> = Either::Left(114);
        let e2: Either<i32, String> = Either::Right("514".to_string());

        eprintln!("e1 = {:?}, e2 = {:?}", e1, e2);

        if let Either::Left(num) = e1 {
            assert_eq!(num, 114);
        } else {
            unreachable!();
        }

        if let Either::Right(s) = e2 {
            assert_eq!(s, "514");
        } else {
            unreachable!();
        }
    }
}
