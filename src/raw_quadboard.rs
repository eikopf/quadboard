use halfling::Nibble;
use std::simd::u64x4;

/// An untyped quadboard representing 64 [`Nibble`] values, 
/// implemented with the [std::simd] API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawQuadboard {
    channels: u64x4,
}

impl RawQuadboard {
    /// Converts the quadboard into an array of its underlying channels.
    pub const fn into_channels(self) -> [u64; 4] {
        *self.channels.as_array()
    }

    /// Creates a new [`RawQuadboard`] with each element set to `value`.
    pub fn splat(value: Nibble) -> Self {
        let value: u8 = value.get();

        // extract the individual bits from the given value
        let (bit1, bit2, bit3, bit4) = unsafe { lower_nibble_bits(value) };

        // construct channels
        let bit_channels = u64x4::from_array([bit1, bit2, bit3, bit4]);

        // choose either u64::MAX or 0u64 based on the bit in each channel
        let channels = bit_channels * u64x4::splat(u64::MAX);

        Self { channels }
    }

    /// Returns the [`Nibble`] at `index` without bounds checking.
    ///
    /// # Safety
    /// `index` must be strictly less than 64.
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
        let qb = RawQuadboard::default();
        let prod: u64 = qb.into_channels().into_iter().product();

        assert!(prod == 0);
    }

    #[test]
    fn raw_quadboard_set_unchecked_is_correct() {
        let mut rqb = RawQuadboard::default();

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
        let mut rqb = RawQuadboard::default();

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
