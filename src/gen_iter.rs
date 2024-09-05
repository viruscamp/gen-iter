use core::iter::Iterator;
use core::marker::Unpin;
use core::ops::{Coroutine, CoroutineState};
use core::pin::Pin;

/// an iterator that holds an internal coroutine representing
/// the iteration state
/// 
/// # Example
/// pin a self-referenced coroutine in heap, then use it as `Iterator`
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::GenIter;
/// use std::boxed::Box;
///
/// let arr = [1, 2];
/// let c = Box::pin(#[coroutine] static move || {
///     let arr = &arr;
///     for i in 0..arr.len() {
///        yield arr[i];
///     }
/// });
/// let mut g = GenIter(c);
///
/// assert_eq!(g.collect::<Vec<i32>>(), [1, 2]);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct GenIter<T>(pub T)
where
    T: Coroutine<Return = ()> + Unpin;

impl<T> Iterator for GenIter<T>
where
    T: Coroutine<Return = ()> + Unpin,
{
    type Item = T::Yield;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            CoroutineState::Yielded(n) => Some(n),
            CoroutineState::Complete(()) => None,
        }
    }
}

impl<G> From<G> for GenIter<G>
where
    G: Coroutine<Return = ()> + Unpin,
{
    #[inline]
    fn from(g: G) -> Self {
        GenIter(g)
    }
}


/// macro to simplify iterator - via - coroutine construction
///
/// - create a movable coroutine as `Iterator`
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter;
///
/// let mut g = gen_iter!({
///     yield 1;
///     yield 2;
/// });
///
/// assert_eq!(g.collect::<Vec<i32>>(), [1, 2]);
/// ```
/// 
/// - create an immovable coroutine (self-referenced) pinned in stack as `Iterator`
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter;
///
/// let arr = [1, 2];
/// let mut g = gen_iter!(static move {
///     let arr = &arr;
///     for i in 0..arr.len() {
///        yield arr[i];
///     }
/// });
///
/// assert_eq!(g.collect::<Vec<i32>>(), [1, 2]);
/// ```
#[macro_export]
macro_rules! gen_iter {
    ($block: block) => {
        $crate::GenIter(#[coroutine] || $block)
    };
    (move $block: block) => {
        $crate::GenIter(#[coroutine] move || $block)
    };

    (static $block: block) => {
        $crate::GenIter { 0: ::core::pin::pin!(#[coroutine] static || $block) }
    };
    (static move $block: block) => {
        $crate::GenIter { 0: ::core::pin::pin!(#[coroutine] static move || $block) }
    };
}


#[cfg(test)]
mod tests {
    use super::GenIter;

    #[test]
    fn it_works() {
        let mut g = gen_iter!({
            yield 1;
            yield 2;
        });

        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn into_gen_iter() {
        let mut g: GenIter<_> = (
            #[coroutine]
            || {
                yield 1;
                yield 2;
            }
        ).into();

        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn gen_iter_macro() {
        let mut g = gen_iter!(move {
            yield 1;
            yield 2;
        });

        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn self_ref_coroutine_in_stack() {
        let c = ::core::pin::pin!(#[coroutine] static || {
            let v1 = [1, 2];
            let v = &v1;
            for i in 0..v.len() {
                yield v[i];
            }
        });
        let mut g = GenIter(c);
        
        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn gen_iter_macro_static() {
        let mut g = gen_iter!(static {
            let v1 = [1, 2];
            let v = &v1;
            for i in 0..v.len() {
                yield v[i];
            }
        });
        
        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn gen_iter_macro_static_move() {
        let v1 = [1, 2];
        let mut g = gen_iter!(static move {
            let v = &v1;
            for i in 0..v.len() {
                yield v[i];
            }
        });
        
        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }
}
