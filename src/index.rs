use thiserror::Error;

/// The set of possible values that an [`Index`] may take, ranging
/// from 0 (inclusive) to 64 (exclusive).
///
/// Effectively, this a subset of `u8` defined such that the compiler
/// can apply the niche value optimisation, and which has no practical
/// runtime cost (compared to something like the `nonmax` crate, for
/// example).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
enum AllowedIndexValue {
    _00,
    _01,
    _02,
    _03,
    _04,
    _05,
    _06,
    _07,
    _08,
    _09,
    _0A,
    _0B,
    _0C,
    _0D,
    _0E,
    _0F,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18,
    _19,
    _1A,
    _1B,
    _1C,
    _1D,
    _1E,
    _1F,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
    _26,
    _27,
    _28,
    _29,
    _2A,
    _2B,
    _2C,
    _2D,
    _2E,
    _2F,
    _30,
    _31,
    _32,
    _33,
    _34,
    _35,
    _36,
    _37,
    _38,
    _39,
    _3A,
    _3B,
    _3C,
    _3D,
    _3E,
    _3F,
}

/// A valid index into a [crate::Quadboard].
///
/// This type guarantees the niche value optimisation, so
/// in particular the following holds.
///
/// ```
/// use quadboard::Index;
///
/// assert_eq!(
///     std::mem::size_of::<Option<Index>>(),
///     std::mem::size_of::<Index>()
/// )
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct Index(AllowedIndexValue);

/// The unit error produced when a numeric conversion into an [`Index`] fails.
#[derive(Debug, Error)]
#[error("Attempted to construct an Index with a value greater than 63.")]
pub struct InvalidIndexError;

impl TryFrom<u8> for Index {
    type Error = InvalidIndexError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match Self::is_valid_index(value) {
            true => Ok(unsafe { Self::new_unchecked(value) }),
            false => Err(InvalidIndexError),
        }
    }
}

impl From<Index> for u8 {
    fn from(value: Index) -> Self {
        value.get()
    }
}

impl Index {
    /// The minimum value representable by an [`Index`], i.e. 0.
    pub const MIN: Self = Index(AllowedIndexValue::_00);

    /// The maximum value representable by an [`Index`], i.e. 63.
    pub const MAX: Self = Index(AllowedIndexValue::_3F);

    /// Returns the value of `self` as a `u8`.
    pub const fn get(&self) -> u8 {
        self.0 as u8
    }

    /// Constructs an [`Index`] without checking the
    /// validity of `value`.
    ///
    /// # Safety
    /// `value` must be less than 64.
    pub const unsafe fn new_unchecked(value: u8) -> Self {
        debug_assert!(value < 64);
        unsafe { std::mem::transmute(value) }
    }

    /// Checks whether `value` can be safely converted into
    /// an [`Index`] using `Index::new_unchecked`. In general,
    /// for some `x: u8`, the operation `Index::new_unchecked(x)`
    /// is safe if and only if `Index::is_valid_index(x)` returns 
    /// `true`.
    ///
    /// You should generally prefer using this function over ad hoc
    /// comparisons on an arbitrary `u8` value, unless you have additional
    /// knowledge about it such that you could write a cheaper check.
    #[inline(always)]
    pub const fn is_valid_index(value: u8) -> bool {
        // we exploit the fact that a `u8` less than 64
        // must necessarily have 0 as its two upper bits.
        (value & 0b11000000) == 0u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niche_value_optimisation_applies_to_index() {
        assert_eq!(
            std::mem::size_of::<Option<Index>>(),
            std::mem::size_of::<Index>()
        )
    }
}
