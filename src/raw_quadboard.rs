//! Untyped quadboards storing [`Nibble`] values.

use crate::index::Index;
use halfling::Nibble;
use std::simd::u64x4;

/// An untyped quadboard, effectively storing 64
/// [`Nibble`] values in a [std::simd::u64x4].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawQuadboard {
    channels: u64x4,
}

impl RawQuadboard {
    /// Consumes `self` and returns an array of the underlying `u64` channels.
    #[inline(always)]
    pub const fn into_channels(self) -> [u64; 4] {
        self.channels.to_array()
    }

    /// Returns a reference to the underlying `u64` channels.
    #[inline(always)]
    pub const fn as_channels(&self) -> &[u64; 4] {
        self.channels.as_array()
    }

    /// Creates a new [`RawQuadboard`] with each element set to `value`.
    #[inline(always)]
    pub fn splat(value: Nibble) -> Self {
        let value: u8 = value.get();

        // extract the individual bits from the given value
        let (bit1, bit2, bit3, bit4) = unsafe { lower_nibble_bits(value) };

        // construct channels, so we have something like
        //
        //                 the bits of `value`, from x = bit1 to w = bit4 ┐
        // 000000000000000000000000000000000000000000000000000000000000000x
        // 000000000000000000000000000000000000000000000000000000000000000y
        // 000000000000000000000000000000000000000000000000000000000000000z
        // 000000000000000000000000000000000000000000000000000000000000000w
        // ^              ^               ^               ^               ^
        // └ bit 64       └ bit 48        └ bit 32        └ bit 16        └ bit 1
        let bit_channels = u64x4::from_array([bit1, bit2, bit3, bit4]);

        // choose either u64::MAX or 0u64 based on the bit in each channel,
        // copying the lowest bit across the entire SIMD lane
        //
        // xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
        // yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy
        // zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz
        // wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww
        // ^              ^               ^               ^               ^
        // └ bit 64       └ bit 48        └ bit 32        └ bit 16        └ bit 1
        let channels = bit_channels * u64x4::splat(u64::MAX);

        Self { channels }
    }

    /// Returns the [`Nibble`] at `index`.
    #[inline(always)]
    pub fn get(&self, index: Index) -> Nibble {
        unsafe { self.get_unchecked(index.get()) }
    }

    /// Sets the value of `self` at `index` to `value`.
    #[inline(always)]
    pub fn set(&mut self, index: Index, value: Nibble) {
        unsafe { self.set_unchecked(index.get(), value) }
    }

    /// Returns the [`Nibble`] at `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: u8) -> Nibble {
        // mask off all other values and shift the remaining bits right
        let mask = u64x4::splat(1 << index);
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
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, index: u8, value: Nibble) {
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
#[inline(always)]
const fn u64x4_channel_sum(value: u64x4) -> u64 {
    let arr = value.to_array();
    arr[0] + arr[1] + arr[2] + arr[3]
}

/// Extracts the lower 4 bits from the given value
/// and returns them in increasing order from left
/// to right.
///
/// # Safety
/// This function assumes the upper four bits of `value` are all 0.
#[inline(always)]
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
        let qb = RawQuadboard::default();
        let prod: u64 = qb.into_channels().into_iter().product();

        assert!(prod == 0);
    }

    #[test]
    fn raw_quadboard_set_unchecked_is_correct() {
        let mut rqb = RawQuadboard::default();

        unsafe {
            rqb.set_unchecked(0, Nibble::try_from(0b1111).unwrap());
            rqb.set_unchecked(5, Nibble::try_from(0b1101).unwrap());
            rqb.set_unchecked(32, Nibble::try_from(0b1111).unwrap());
            rqb.set_unchecked(63, Nibble::try_from(0b0111).unwrap());
        }

        let channels = rqb.into_channels();
        for (i, lane) in channels.iter().enumerate() {
            eprintln!("channel {}: 0x{:016x}", i, lane);
        }

        // these values were chosen to match with the particular
        // values set above; changes to either will break the test
        assert_eq!(channels[0], 0x8000000100000021);
        assert_eq!(channels[1], 0x8000000100000001);
        assert_eq!(channels[2], 0x8000000100000021);
        assert_eq!(channels[3], 0x0000000100000021);
    }

    #[test]
    fn raw_quadboard_get_unchecked_is_correct() {
        let mut rqb = RawQuadboard::default();

        unsafe {
            rqb.set_unchecked(17, Nibble::try_from(0b1111).unwrap());
            rqb.set_unchecked(3, Nibble::try_from(0b1001).unwrap());
            rqb.set_unchecked(38, Nibble::try_from(0b0100).unwrap());

            assert_eq!(0b1111, rqb.get_unchecked(17).get());
            assert_eq!(0b1001, rqb.get_unchecked(3).get());
            assert_eq!(0b0100, rqb.get_unchecked(38).get());
        }
    }
}
