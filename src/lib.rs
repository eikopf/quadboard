//! A [quadboard](https://www.chessprogramming.org/Quad-Bitboards) is simply
//! a collection of four bitboards (i.e. `u64`s, here called *channels*) used to
//! store information, most commonly about piece positions.
//!
//! Generally, we treat quadboards as though they are given by four horizontal binary
//! channels, each of length 64, and so each vertical "slice" is itself a [`Nibble`].
//!
//!
//! # Typed and Untyped Quadboards
//! Since a quadboard is really just four bitboards, it is effectively an untyped
//! fixed-length buffer. This usage is reflected in the [`RawQuadboard`] struct,
//! which allows the writing of arbitrary nibbles to arbitrary locations with no
//! concern for their interpretation or validity.
//!
//! But in actual usage, a quadboard is meant to represent a single type, and in
//! that context the manual conversion between a [`Nibble`] and some `T` is just
//! distracting boilerplate. Hence, the [`Quadboard`] struct wraps a [`RawQuadboard`]
//! and includes a generic type parameter `T`; the possible interactions with this
//! type are then governed by trait bounds on `T`, and in particular the [`From`],
//! [`Into`], [`TryFrom`], and [`TryInto`] impls whose type parameter is [`Nibble`].
//!
//! # SIMD
//! `TODO`


#![warn(missing_docs)]
#![feature(portable_simd)]

mod index;
mod raw_quadboard;

use std::{marker::PhantomData, mem::MaybeUninit};

pub use halfling::Nibble;
pub use raw_quadboard::RawQuadboard;
pub use index::Index;
pub use index::InvalidIndexError;

/// A type whose encoding defines an explicit `EMPTY` value,
/// representing something like an empty space.
pub trait EmptyNibble: Into<Nibble> {
    /// The designated empty nibble for this type.
    const EMPTY_NIBBLE: halfling::Nibble;
}

/// An unopinionated [quadboard](https://www.chessprogramming.org/Quad-Bitboards)
/// implementation, using Rust's [std::simd] API for accelerated per-nibble operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quadboard<T> {
    inner: RawQuadboard,
    _data: PhantomData<T>,
}

impl<T, E> Quadboard<T>
where
    T: TryFrom<Nibble, Error = E>,
    E: std::error::Error,
{
    /// Consumes `self` and maps `T::try_from` over
    /// it, returning the result in a fixed
    /// length array.
    pub fn into_array(self) -> [Result<T, E>; 64]
    where
        T: Copy,
        E: Copy,
    {
        let mut arr: [MaybeUninit<Result<T, E>>; 64] = [MaybeUninit::uninit(); 64];

        let mut index = 0;
        for elem in &mut arr {
            let nibble = unsafe { self.get_unchecked(index) };
            elem.write(nibble);
            index += 1;
        }

        // this is safe because MaybeInit guarantees size, layout,
        // and ABI compatibility. it would be better if this was
        // a call to std::mem::transmute, but currently the compiler
        // is extremely conservative with generic functions and memory
        // shenanigans.
        unsafe { std::mem::transmute_copy(&arr) }
    }

    /// Reads the [`Nibble`] at the given index and
    /// attempts a [`TryFrom`] conversion before returning.
    ///
    /// # Panics
    /// Panics if `index >= 64`, i.e. if it is an invalid index
    /// into a [`Quadboard`].
    pub fn read(&self, index: u8) -> Result<T, E> {
        assert!(index < 64);
        unsafe { self.get_unchecked(index) }
    }

    /// Reads the [`Nibble`] at the given index without bounds checking
    /// and attempts a [`TryFrom`] conversion before returning.
    ///
    /// # Safety
    /// `index` must be less than 64.
    pub unsafe fn get_unchecked(&self, index: u8) -> Result<T, E> {
        let nibble = unsafe { self.inner.get_unchecked(index) };
        T::try_from(nibble)
    }
}

impl<T> Quadboard<T> {
    /// Returns an empty [`Quadboard`], where the associated `EMPTY` value
    /// on the [`EmptyNibble`] implementation for `T` has been written to
    /// every index.
    #[inline(always)]
    pub fn empty() -> Self
    where
        T: EmptyNibble,
    {
        Self {
            inner: RawQuadboard::splat(T::EMPTY_NIBBLE),
            _data: PhantomData,
        }
    }

    /// Converts `value` into a [`Nibble`] and writes the
    /// resulting `T` value to `index`.
    ///
    /// # Panics
    /// Panics if `index >= 64`, i.e. if the given index is out of
    /// bounds.
    pub fn write(&mut self, value: T, index: u8)
    where
        T: Into<Nibble>,
    {
        assert!(index < 64);
        unsafe { self.set_unchecked(value, index) };
    }

    /// Converts `value` into a [`Nibble`] and writes the
    /// resulting `T` value to `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
    pub unsafe fn set_unchecked(&mut self, value: T, index: u8)
    where
        T: Into<Nibble>,
    {
        let value: Nibble = value.into();
        unsafe { self.inner.set_unchecked(value, index) };
    }
}
