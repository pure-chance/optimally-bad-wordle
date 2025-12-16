//! Compact bitmask representation of (5-letter) words.

use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use serde::{Deserialize, Serialize};

/// Represents a set of letters as a compact bitmask.
///
/// A `Signature` uses a 26-bit integer where bit `i` indicates whether the
/// letter at position `i` in the alphabet is present. For example, the word
/// "slate" maps to the signature {s,l,a,t,e}, represented as a bitmask with
/// bits set at positions 0 (a), 4 (e), 11 (l), 18 (s), and 19 (t).
///
/// This representation enables efficient disjointness checking: two signatures
/// are disjoint if and only if their bitwise AND equals zero.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Signature(u32);

impl Signature {
    /// Construct a new `Signature` with no letters set.
    #[inline]
    #[must_use]
    pub fn new(word: &str) -> Self {
        let mut mask = 0u32;
        for byte in word.as_bytes() {
            mask |= 1u32 << (byte - b'a');
        }
        Self(mask)
    }

    /// Construct a `Signature` from the underlying letter mask.
    #[inline]
    #[must_use]
    pub const fn from_mask(mask: u32) -> Self {
        Self(mask)
    }

    /// Checks if two signatures have no letters in common.
    #[inline]
    #[must_use]
    pub const fn disjoint(self, other: Self) -> bool {
        self.0 & other.0 == 0
    }

    /// Returns the union of two signatures.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the intersection of two signatures.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns the number of unique letters set in the signature.
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

impl BitAnd for Signature {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Signature {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Signature {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Signature {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<&str> for Signature {
    #[inline]
    fn from(word: &str) -> Self {
        Self::new(word)
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:026b}", self.0)
    }
}
