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
//! fixed-length buffer. This usage is reflected in the [`RawQuadBoard`] struct,
//! which allows the writing of arbitrary nibbles to arbitrary locations with no
//! concern for their interpretation or validity.
//!
//! But in actual usage, a quadboard is meant to represent a single type, and in
//! that context the manual conversion between a [`Nibble`] and some `T` is just
//! distracting boilerplate. Hence, the [`QuadBoard`] struct wraps a [`RawQuadBoard`]
//! and includes a generic type parameter `T`; the possible interactions with this
//! type are then governed by trait bounds on `T`, and in particular the [`From`],
//! [`Into`], [`TryFrom`], and [`TryInto`] impls whose type parameter is [`Nibble`].
//!
//! # SIMD
//! `TODO`

#![warn(missing_docs)]
#![feature(portable_simd)]
// the two following features are enabled to allow some
// const SIMD stuff, but they seem very far from being
// stable at the moment
#![feature(const_trait_impl)]
#![feature(effects)]

pub use halfling::Nibble;
use std::{marker::PhantomData, mem::MaybeUninit, simd::u64x4};

/// A type whose encoding defines an explicit `EMPTY` value,
/// representing something like an empty space.
pub trait EmptyNibble: Into<Nibble> {
    /// The designated empty nibble for this type.
    const EMPTY_NIBBLE: halfling::Nibble;
}

/// An unopinionated [quadboard](https://www.chessprogramming.org/Quad-Bitboards)
/// implementation, using Rust's [std::simd] API for accelerated per-nibble operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuadBoard<T> {
    inner: RawQuadBoard,
    _data: PhantomData<T>,
}

impl<T, E> QuadBoard<T>
where
    T: TryFrom<Nibble, Error = E>,
    E: std::error::Error,
{
    /// Consumes `self` and maps `T::try_from` over
    /// the [`QuadBoard`], returning the result in a fixed
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
    /// into a [`QuadBoard`].
    pub const fn read(&self, index: u8) -> Result<T, E> {
        assert!(index < 64);
        unsafe { self.get_unchecked(index) }
    }

    /// Reads the [`Nibble`] at the given index without bounds checking
    /// and attempts a [`TryFrom`] conversion before returning.
    ///
    /// # Safety
    /// `index` must be less than 64.
    pub const unsafe fn get_unchecked(&self, index: u8) -> Result<T, E> {
        let nibble = unsafe { self.inner.get_unchecked(index) };
        T::try_from(nibble)
    }
}

impl<T> QuadBoard<T> {
    /// Returns an empty [`QuadBoard`], where the associated `EMPTY` value
    /// on the [`EmptyNibble`] implementation for `T` has been written to
    /// every index.
    pub const fn empty() -> Self
    where
        T: EmptyNibble,
    {
        Self {
            inner: RawQuadBoard::splat(T::EMPTY_NIBBLE),
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

/// An untyped buffer of 64 [`Nibble`] values, stored
/// densely in 4 `u64` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawQuadBoard {
    channels: u64x4,
}

impl RawQuadBoard {
    /// Converts the quadboard into an array of its underlying channels.
    pub const fn into_channels(self) -> [u64; 4] {
        *self.channels.as_array()
    }

    /// Creates a new [`RawQuadBoard`] with each element set to `value`.
    pub const fn splat(value: Nibble) -> Self {
        let value: u8 = value.get();

        // extract the individual bits from the given value
        let (bit1, bit2, bit3, bit4) = unsafe { lower_nibble_bits(value) };

        // construct channels
        let bit_channels = u64x4::from_array([bit1, bit2, bit3, bit4]);

        // choose either u64::MAX or 0u64 based on the bit in each channel
        let channels = bit_channels * u64x4::from_array([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

        Self { channels }
    }

    /// Returns the [`Nibble`] at `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
    pub const unsafe fn get_unchecked(&self, index: u8) -> Nibble {
        // mask off all other values and shift the remaining bits right
        let mask = u64x4::from_array([1 << index, 1 << index, 1 << index, 1 << index]);
        let masked_board = self.channels & mask;
        let bits = masked_board >> (index as u64);

        // shift values according to channel index
        let values = bits << u64x4::from_array([0, 1, 2, 3]);
        unsafe { Nibble::new_unchecked(u64x4_channel_sum(values) as u8) }
    }

    /// Writes `value` to `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
    pub unsafe fn set_unchecked(&mut self, value: Nibble, index: u8) {
        let value: u8 = value.get();
        let (bit1, bit2, bit3, bit4) = unsafe { lower_nibble_bits(value) };

        // shift the bits to the indexed location
        let channel1 = bit1 << index;
        let channel2 = bit2 << index;
        let channel3 = bit3 << index;
        let channel4 = bit4 << index;

        // create mask vector with all bits set and clear the bits at the indexed location
        let mut mask = u64x4::splat(u64::MAX);
        let clear_mask = !(1 << index);
        mask &= u64x4::splat(clear_mask);

        // mask off existing value and write new value
        self.channels &= mask;
        self.channels |= u64x4::from_array([channel1, channel2, channel3, channel4]);
    }
}

/// A `const` equivalent to `value.to_array().iter().sum()`.
const fn u64x4_channel_sum(value: u64x4) -> u64 {
    let arr = value.to_array();
    arr[0] + arr[1] + arr[2] + arr[3]
}

/// Extracts the lower 4 bits from the given value
/// and returns them in increasing order from left
/// to right.
///
/// At some point, this function should be replaced in
/// favour of dedicated const methods on [`Nibble`] itself.
///
/// # Safety
/// This function assumes the upper four bits of `value` are all 0.
const unsafe fn lower_nibble_bits(value: u8) -> (u64, u64, u64, u64) {
    let bit1 = (value & 0b0001) as u64;
    let bit2 = ((value & 0b0010) >> 1) as u64;
    let bit3 = ((value & 0b0100) >> 2) as u64;
    let bit4 = (value >> 3) as u64;

    (bit1, bit2, bit3, bit4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_quadboard_new_is_all_zero() {
        // the channels will all be zero iff their product is zero
        let qb = RawQuadBoard::default();
        let prod: u64 = qb.into_channels().into_iter().product();

        assert!(prod == 0);
    }

    #[test]
    fn raw_quadboard_set_unchecked_is_correct() {
        let mut rqb = RawQuadBoard::default();

        unsafe {
            rqb.set_unchecked(Nibble::try_from(0b1111).unwrap(), 0);
            rqb.set_unchecked(Nibble::try_from(0b1101).unwrap(), 5);
            rqb.set_unchecked(Nibble::try_from(0b1111).unwrap(), 32);
            rqb.set_unchecked(Nibble::try_from(0b0111).unwrap(), 63);
        }

        let lanes = rqb.into_channels().map(|board| u64::from(board));
        for (i, lane) in lanes.iter().enumerate() {
            eprintln!("channel {}: 0x{:016x}", i, lane);
        }

        // these values were chosen to match with the particular
        // values set above; changes to either will break the test
        assert_eq!(lanes[0], 0x8000000100000021);
        assert_eq!(lanes[1], 0x8000000100000001);
        assert_eq!(lanes[2], 0x8000000100000021);
        assert_eq!(lanes[3], 0x0000000100000021);
    }

    #[test]
    fn raw_quadboard_get_unchecked_is_correct() {
        let mut rqb = RawQuadBoard::default();

        unsafe {
            rqb.set_unchecked(Nibble::try_from(0b1111).unwrap(), 17);
            rqb.set_unchecked(Nibble::try_from(0b1001).unwrap(), 3);
            rqb.set_unchecked(Nibble::try_from(0b0100).unwrap(), 38);

            assert_eq!(0b1111, rqb.get_unchecked(17).get());
            assert_eq!(0b1001, rqb.get_unchecked(3).get());
            assert_eq!(0b0100, rqb.get_unchecked(38).get());
        }
    }
}
