//! # gen_iter - create coroutines to use as iterators
//! 
//! **Important: [rename Generator to Coroutine](https://github.com/rust-lang/rust/pull/116958)**
//!
//! ## Prerequirements
//! Nightly rust toolchain of edition 2021 after 2023-10-21.
//! 
//! ## [`GenIter`] and [`gen_iter!`]
//! [`GenIter`] converts a [`Coroutine<(), Return=()>`](core::ops::Coroutine) into an iterator over the
//! yielded type of the coroutine. The return type of the coroutine needs to be `()`.
//!
//! [`gen_iter!`] helps to create a [`GenIter`]
//!
//! ```
//! #![feature(coroutines)]
//!
//! use gen_iter::gen_iter;
//!
//! fn fibonacci() -> impl Iterator<Item = u64> {
//!     gen_iter!({
//!         let mut a = 0;
//!         let mut b = 1;
//!
//!         loop {
//!             let c = a + b;
//!             a = b;
//!             b = c;
//!
//!             yield a;
//!         }
//!     })
//! }
//!
//! for elem in fibonacci().map(|x| 2 * x).take(10) {
//!     println!("{}", elem);
//! }
//! ```
//! 
//! ## [`GenIterReturn`] and [`gen_iter_return!`]
//! [`GenIterReturn`] can be converted from a [`Coroutine<()>`](core::ops::Coroutine),
//! `&mut GenIterReturn<G>` can be used as iterator.
//! The return value of the coroutine can be got after the iterator is exhausted.
//! 
//! [`gen_iter_return!`] helps to create a [`GenIterReturn`].
//! 
//! ```
//! #![feature(coroutines)]
//!
//! use gen_iter::gen_iter_return;
//!
//! let mut g = gen_iter_return!({
//!     yield 1;
//!     yield 2;
//!     return "done";
//! });
//! 
//! for y in &mut g {
//!     println!("yield {}", y);
//! }
//! println!("coroutine is_done={}", g.is_done()); // true
//! println!("coroutine returns {}", g.return_or_self().ok().unwrap()); // "done"
//! ```

#![no_std]
#![feature(coroutines, coroutine_trait)]
#![feature(stmt_expr_attributes)]

mod gen_iter;
pub use gen_iter::*;

mod gen_iter_return;
pub use gen_iter_return::*;
