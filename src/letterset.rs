use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use serde::{Deserialize, Serialize};

/// Represents a set of letters.
///
/// A `LetterSet` is a mask, where setting a bit at position `i` indicates that
/// the letter `i` is present in the word.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LetterSet(u32);

impl LetterSet {
    /// Construct a new `LetterSet` with no letters set.
    #[inline]
    #[must_use]
    pub fn new(word: &str) -> Self {
        let mut mask = 0u32;
        for byte in word.as_bytes() {
            mask |= 1u32 << (byte - b'a');
        }
        Self(mask)
    }
    /// Construct a `LetterSet` from the underlying letter mask.
    #[inline]
    #[must_use]
    pub const fn from_mask(mask: u32) -> Self {
        Self(mask)
    }
    /// Checks if two lettersets have no letters in common.
    #[inline]
    #[must_use]
    pub const fn disjoint(self, other: Self) -> bool {
        self.0 & other.0 == 0
    }
    /// Returns the union of two lettersets.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Returns the intersection of two lettersets.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
    /// Returns the number of unique letters set in the letterset.
    #[inline]
    #[must_use]
    pub const fn count_letters(self) -> u32 {
        self.0.count_ones()
    }
    /// Returns the underlying mask value.
    #[inline]
    #[must_use]
    pub const fn mask(self) -> u32 {
        self.0
    }
}

impl BitAnd for LetterSet {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for LetterSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for LetterSet {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for LetterSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<&str> for LetterSet {
    #[inline]
    fn from(word: &str) -> Self {
        Self::new(word)
    }
}

impl Debug for LetterSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:026b}", self.0)
    }
}
