use core::iter::{FusedIterator, Iterator};
use core::marker::Unpin;
use core::ops::{Coroutine, CoroutineState};
use core::pin::Pin;

/// `GenIterReturn<G>` holds a coroutine `G` or the return value of `G`,
/// `&mut GenIterReturn<G>` acts as an iterator.
///
/// Differences with `GenIter<G>`:
/// 1. able to get return value of a coroutine
/// 2. safe to call `next()` after coroutine is done without panic
/// 3. maybe less efficient than `GenIter<G>`
#[derive(Copy, Clone, Debug)]
pub struct GenIterReturn<G: Coroutine + Unpin>(
    #[doc(hidden)]
    pub Result<G::Return, G>
);

impl<G: Coroutine + Unpin> GenIterReturn<G> {
    #[inline]
    pub fn new(g: G) -> Self {
        GenIterReturn(Err(g))
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.0.is_ok()
    }

    #[inline]
    pub fn return_or_self(self) -> Result<G::Return, Self> {
        match self.0 {
            Ok(r) => Ok(r),
            Err(_) => Err(self),
        }
    }
}

/// Force use `&mut g` as iterator to prevent the code below,
/// in which return value cannot be got.
/// ```compile_fail
/// // !!INVALID CODE!!
/// # #![feature(coroutines)]
/// # use gen_iter::gen_iter_return;
/// let mut g = gen_iter_return!({ yield 1; return "done"; });
/// for v in g {} // invalid, because `GenIterReturn<G>` is not `Iterator`
/// let ret = g.return_or_self(); // g is dropped after for loop
/// ```
impl<G: Coroutine + Unpin> Iterator for &mut GenIterReturn<G> {
    type Item = G::Yield;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Ok(_) => None,
            Err(ref mut g) => match Pin::new(g).resume(()) {
                CoroutineState::Yielded(y) => Some(y),
                CoroutineState::Complete(r) => {
                    self.0 = Ok(r);
                    None
                },
            },
        }
    }
}

/// `GenIterReturn<G>` satisfies the trait `FusedIterator`
impl<G: Coroutine + Unpin> FusedIterator for &mut GenIterReturn<G> {}

impl<G: Coroutine + Unpin> From<G> for GenIterReturn<G> {
    #[inline]
    fn from(g: G) -> Self {
        GenIterReturn::new(g)
    }
}

/// macro to simplify iterator - via - coroutine with return value construction
/// - create a movable coroutine as `Iterator`
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter_return;
///
/// let mut g = gen_iter_return!({
///     yield 1;
///     yield 2;
///     return "done";
/// });
///
/// assert_eq!((&mut g).collect::<Vec<_>>(), [1, 2]); // use `&mut g` as an iterator
/// assert_eq!(g.is_done(), true); // check whether the coroutine is done
/// assert_eq!((&mut g).next(), None); // safe to call `next()` after done
/// assert_eq!(g.return_or_self().ok(), Some("done")); // get return value of the coroutine
/// ```
/// 
/// - create an immovable coroutine (self-referenced) pinned in stack as `Iterator`
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter_return;
///
/// let arr = [1, 2];
/// let mut g = gen_iter_return!(static move {
///     let v = &arr;
///     for i in 0..v.len() {
///        yield v[i];
///     }
///     return v.len();
/// });
///
/// assert_eq!((&mut g).collect::<Vec<i32>>(), [1, 2]);
/// assert_eq!(g.is_done(), true);
/// assert_eq!(g.return_or_self().ok(), Some(2));
/// ```
#[macro_export]
macro_rules! gen_iter_return {
    ($block: block) => {
        $crate::GenIterReturn::new(#[coroutine] || $block)
    };
    (move $block: block) => {
        $crate::GenIterReturn::new(#[coroutine] move || $block)
    };

    (static $block: block) => {
        $crate::GenIterReturn {
            0: ::core::result::Result::Err {
                0: ::core::pin::pin!(#[coroutine] static || $block)
            }
        }
    };
    (static move $block: block) => {
        $crate::GenIterReturn {
            0: ::core::result::Result::Err {
                0: ::core::pin::pin!(#[coroutine] static move || $block)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::GenIterReturn;

    /// test `new` and all instance method,
    /// and show that it won't panic when call `next()` even exhausted.
    #[test]
    fn it_works() {
        let mut g = GenIterReturn::new(
            #[coroutine]
            || {
                yield 1;
                return "done";
            },
        );

        assert_eq!((&mut g).next(), Some(1));
        assert_eq!(g.is_done(), false);

        g = match g.return_or_self() {
            Ok(_) => panic!("coroutine is done but should not"),
            Err(g) => g,
        };

        assert_eq!((&mut g).next(), None);
        assert_eq!(g.is_done(), true);

        assert_eq!((&mut g).next(), None); // it won't panic when call `next()` even exhausted.

        assert_eq!(g.return_or_self().ok(), Some("done"));
    }

    #[test]
    fn from_coroutine() {
        let mut g = GenIterReturn::from(
            #[coroutine]
            || {
                yield 1;
                return "done";
            },
        );

        assert_eq!((&mut g).next(), Some(1));
        assert_eq!((&mut g).next(), None);

        assert_eq!(g.is_done(), true);
        assert_eq!(g.return_or_self().ok(), Some("done"));
    }

    /// normal usage using macro `gen_iter_return`
    #[test]
    fn macro_gen_iter_return() {
        let mut g = gen_iter_return!(move {
            yield 1;
            yield 2;
            return "done";
        });

        let (mut sum, mut count) = (0, 0);
        for y in &mut g {
            sum += y;
            count += 1;
        }
        assert_eq!((sum, count), (3, 2));

        assert_eq!(g.is_done(), true);
        assert_eq!(g.return_or_self().ok(), Some("done"));
    }

    /// use macro `gen_iter_return` to make a immovable coroutine
    #[test]
    fn macro_gen_iter_return_static_move() {
        let arr = [1, 2];
        let mut g = gen_iter_return!(static move {
            let v = &arr;
            for i in 0..v.len() {
                yield v[i];
            }
            return v.len();
        });

        let (mut sum, mut count) = (0, 0);
        for y in &mut g {
            sum += y;
            count += 1;
        }
        assert_eq!((sum, count), (3, 2));

        assert_eq!(g.is_done(), true);
        assert_eq!(g.return_or_self().ok(), Some(2));
    }
}
