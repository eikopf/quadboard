//! A [quadboard](https://www.chessprogramming.org/Quad-Bitboards) is simply
//! a collection of four bitboards (i.e. `u64`s, here called *channels*) used to
//! store information, most commonly about piece positions.
//!
//! Generally, we treat quadboards as though they are given by four horizontal binary
//! channels, each of length 64, and so each 4-bit vertical section is a [`Nibble`].
//!
//! # Typed and Untyped Quadboards
//! Since a quadboard is really just four bitboards, it is effectively an untyped
//! fixed-length buffer. This usage is reflected in the [`RawQuadboard`] struct,
//! which allows the writing of arbitrary nibbles to arbitrary locations with no
//! concern for their interpretation or validity.
//!
//! But in actual usage, a quadboard is meant to represent a `[T; 64]` for some `T`;
//! in this context the manual conversion between [`Nibble`] and `T` values is just
//! distracting boilerplate. Hence, the [`Quadboard`] struct wraps a [`RawQuadboard`]
//! and includes a generic type parameter `T`; the possible interactions with this
//! type are then governed by trait bounds on `T`, and in particular the [`From`],
//! [`Into`], [`TryFrom`], and [`TryInto`] impls whose type parameter is [`Nibble`].
//!
//! # SIMD
//! As the `portable_simd` feature is currently nightly-only, this crate is also
//! considered to be unstable. It's possible to write non-SIMD equivalents to the
//! algorithms used here and provide the SIMD versions via cargo feature flags,
//! but my hope is that [std::simd] will be stabilised relatively soon.

#![warn(missing_docs)]
#![feature(portable_simd)]

pub mod index;
pub mod raw_quadboard;

use std::marker::PhantomData;

use crate::index::Index;
use crate::raw_quadboard::RawQuadboard;
pub use halfling::Nibble;

/// A fixed-length 32-byte buffer of 64 `T` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quadboard<T> {
    inner: RawQuadboard,
    _data: PhantomData<T>,
}

impl<T> Default for Quadboard<T>
where
    T: Default + Into<Nibble>,
{
    fn default() -> Self {
        Self {
            inner: RawQuadboard::splat(T::default().into()),
            _data: Default::default(),
        }
    }
}

impl<T> Quadboard<T> {
    /// Returns the value at the given [`Index`].
    #[inline(always)]
    pub fn get(&self, index: Index) -> T
    where
        Nibble: Into<T>,
    {
        unsafe { self.get_unchecked(index.get()) }
    }

    /// Reads the [`Nibble`] at the given index and
    /// passes it to `T::from`.
    ///
    /// # Safety
    /// `index` must be less than 64.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: u8) -> T
    where
        Nibble: Into<T>,
    {
        let nibble = unsafe { self.inner.get_unchecked(index) };
        nibble.into()
    }

    /// Converts `value` into a [`Nibble`] and writes the
    /// resulting `T` value to the element at `index`.
    #[inline(always)]
    pub fn set(&mut self, index: Index, value: T)
    where
        T: Into<Nibble>,
    {
        unsafe { self.set_unchecked(index.get(), value) };
    }

    /// Converts `value` into a [`Nibble`] and writes the
    /// resulting `T` value to `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, index: u8, value: T)
    where
        T: Into<Nibble>,
    {
        let value: Nibble = value.into();
        unsafe { self.inner.set_unchecked(index, value) };
    }

    /// Returns a reference to the underlying [`RawQuadboard`].
    #[inline(always)]
    pub const fn as_raw_quadboard(&self) -> &RawQuadboard {
        &self.inner
    }
}
