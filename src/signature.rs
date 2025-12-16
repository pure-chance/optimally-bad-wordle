//! Compact bitmask representation of (5-letter) words.

use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use serde::{Deserialize, Serialize};

/// Compact bitmask representation of a word's unique letters.
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
    /// Construct a new `Signature` from a word.
    ///
    /// **Correctness**: The word must be exactly 5 lowercase ASCII letters (a valid Wordle
    /// answer / guess). This is checked by assertions in debug builds.
    #[inline]
    #[must_use]
    pub fn new(word: &str) -> Self {
        debug_assert!(word.len() == 5 && word.chars().all(|c| c.is_ascii_lowercase()));
        let mut mask = 0u32;
        for byte in word.as_bytes() {
            mask |= 1u32 << (byte - b'a');
        }
        Self(mask)
    }

    /// Construct a `Signature` from a raw bitmask.
    ///
    /// This allows creating signatures from arbitrary letter combinations.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let hello = Signature::new("hello");
        let world = Signature::new("world");

        assert_eq!(hello.count_letters(), 4); // h, e, l, o
        assert_eq!(world.count_letters(), 5); // w, o, r, l, d
        assert!(!hello.disjoint(world)); // share 'l' and 'o'

        let union = hello.union(world);
        assert_eq!(union.count_letters(), 7); // h, e, l, o, w, r, d
    }

    #[test]
    fn test_disjoint_signatures() {
        let brick = Signature::new("brick");
        let jumpy = Signature::new("jumpy");
        assert!(brick.disjoint(jumpy));
    }

    #[test]
    fn test_anagrams_are_equal() {
        let slate = Signature::new("slate");
        let steal = Signature::new("steal");
        let tales = Signature::new("tales");

        assert_eq!(slate, steal);
        assert_eq!(steal, tales);
    }

    #[test]
    fn test_intersection() {
        let hello = Signature::new("hello");
        let world = Signature::new("world");
        let intersection = hello.intersection(world);

        // Should contain 'l' and 'o'
        assert_eq!(intersection.count_letters(), 2);
    }

    #[test]
    fn test_from_mask() {
        let mask = 0b111; // bits for 'a', 'b', 'c'
        let sig = Signature::from_mask(mask);
        assert_eq!(sig.mask(), mask);
        assert_eq!(sig.count_letters(), 3);
    }
}
