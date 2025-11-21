//! Filters lettersets to identify all valid disjoint packings.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::letterset::LetterSet;

/// Filters lettersets to identify all valid disjoint packings.
///
/// The `Filterer` solves the core combinatorial problem: finding all
/// combinations of one answer letterset and six guess lettersets where all
/// seven are pairwise disjoint (share no letters in common). This ensures the
/// guesses provide zero information about the answer.
///
/// # Algorithm
///
/// The filterer employs a three-stage process, executed in parallel for each
/// answer:
///
/// **1. Enumerate Triples**
///
/// For each answer a, the algorithm first eliminates all guesses that share any
/// letters with a, producing Compatible[a]. This reduces the number of
/// comparisons by orders of magnitude. It then enumerates all valid disjoint
/// triples (g₁, g₂, g₃) from this reduced set.
///
/// **2. Partition Triples**
///
/// Each triple is partitioned based on its intersection with the top 10 most
/// common letters in the guesses vocabulary. For the standard Wordle word list,
/// this mask is "seaoriltnu" (note: s > e > a > ... > u by frequency). Triples
/// with identical partition signatures are grouped together, creating O(2¹⁰) =
/// 1,024 possible bins.
///
/// **3. Compare compatible triple pairs**
///
/// The algorithm compares pairs of partition bins rather than individual
/// triples. For each pair of bins, if their partition signatures are disjoint
/// (bitwise AND equals zero), all cross-bin triple pairs are candidates for
/// full verification. If the signatures overlap, the entire bin pair is
/// skipped. This pruning reduces the comparison space from O(T²) to between
/// 100,000 and 700,000 comparisons for answers with large compatible sets,
/// where T is the total number of triples.
///
/// # Runtime
///
/// In practice, the algorithm runs in ~20 seconds.
pub struct Filterer {
    /// All unique lettersets from answers.
    answer_sets: Box<[LetterSet]>,
    /// All unique lettersets from guesses.
    guess_sets: Box<[LetterSet]>,
}

impl Filterer {
    /// Construct a new `Filterer` with the given answers and guesses.
    #[must_use]
    pub fn new(answers: &[&str], guesses: &[&str]) -> Self {
        let answer_sets: Box<[LetterSet]> = answers
            .iter()
            .map(|&w| w.into())
            .unique()
            .sorted()
            .collect();
        let guess_sets: Box<[LetterSet]> = guesses
            .iter()
            .map(|&w| w.into())
            .unique()
            .sorted()
            .collect();
        Self {
            answer_sets,
            guess_sets,
        }
    }

    /// Find all possible packings of answer + 6 guesses.
    pub fn find_packings(&self) -> HashSet<Packing> {
        let counter = AtomicUsize::new(0);
        let total = self.answer_sets.len();

        self.answer_sets
            .par_iter()
            .map(|answer| {
                let progress = counter.fetch_add(1, Ordering::Relaxed) + 1;
                println!("Processing answer: {answer:?} ({progress}/{total})");
                self.find_packings_for_answer(*answer)
            })
            .reduce(HashSet::new, |mut acc, packings_for_answer| {
                acc.extend(packings_for_answer);
                acc
            })
    }

    /// Find all possible packings of this particular answer + 6 guesses.
    #[must_use]
    pub fn find_packings_for_answer(&self, answer: LetterSet) -> HashSet<Packing> {
        let triples = Self::find_triples_for_answer(answer, &self.guess_sets);
        let partition = LetterSet::new("seaoriltnu");
        let partitions = Self::partition_triples_by_letterset(&triples, partition);
        let packings = Self::scan_and_merge_partitions(&partitions, answer);
        packings.into_iter().collect()
    }

    /// Find all triples for this particular answer.
    ///
    /// **Correctness**: All triples are unique and sorted by construction.
    fn find_triples_for_answer(answer: LetterSet, guess_sets: &[LetterSet]) -> Vec<Triple> {
        let guess_sets: Vec<LetterSet> = guess_sets
            .iter()
            .copied()
            .filter(|&ls| ls.disjoint(answer))
            .collect();

        let mut triples = Vec::new();
        for (i, &ls1) in guess_sets.iter().enumerate() {
            for (j, &ls2) in guess_sets.iter().enumerate().skip(i + 1) {
                if !ls1.disjoint(ls2) {
                    continue;
                }
                for (_, &ls3) in guess_sets.iter().enumerate().skip(j + 1) {
                    if ls1.union(ls2).disjoint(ls3) {
                        let triple = Triple::new(ls1, ls2, ls3);
                        triples.push(triple);
                    }
                }
            }
        }

        triples
    }

    /// Partition triples by some partition letterset.
    ///
    /// Returns a `HashMap` where the keys are the intersection of each triple's
    /// mask with the partition, and the values are vectors of triples that
    /// share the same intersection.
    fn partition_triples_by_letterset(
        triples: &[Triple],
        partition: LetterSet,
    ) -> HashMap<LetterSet, Vec<Triple>> {
        let mut partitions = HashMap::new();
        for &triple in triples {
            let key = partition.intersection(triple.mask);
            partitions.entry(key).or_insert_with(Vec::new).push(triple);
        }
        partitions
    }

    /// Finds all packings for a given answer by merging disjoint triples using
    /// a scan-and-merge strategy.
    ///
    /// # Algorithm
    ///
    /// 1. Iterates over pairs of partitions, skipping those whose keys are not
    ///    disjoint.
    /// 2. For compatible partitions, check if each pair of triples, one from
    ///    each partition, is disjoint.
    /// 3. If any pair of triples is disjoint, merge them and store the result
    ///    as a valid packing.
    fn scan_and_merge_partitions(
        partition: &HashMap<LetterSet, Vec<Triple>>,
        answer: LetterSet,
    ) -> Vec<Packing> {
        let mut packings = Vec::new();
        for keys in partition.iter().combinations_with_replacement(2) {
            let (&key1, triples1) = keys[0];
            let (&key2, triples2) = keys[1];
            if !key1.disjoint(key2) {
                continue;
            }
            for (&triple1, &triple2) in triples1.iter().cartesian_product(triples2.iter()) {
                if !triple1.disjoint(triple2) {
                    continue;
                }
                let guesses = Packing::sort(triple1.lettersets, triple2.lettersets);
                packings.push(Packing::new(answer, guesses));
            }
        }
        packings
    }
}

/// A solution—a 7-packing of disjoint lettersets (1 answer + 6 guesses).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packing {
    answer: LetterSet,
    guesses: [LetterSet; 6],
}

impl Packing {
    /// Construct a new `Packing` with the given answer and guesses.
    ///
    /// **Correctness**: A packing's guesses must be sorted to ensure that
    /// comparisons depend exclusively on membership, and not order.
    #[must_use]
    pub const fn new(answer: LetterSet, guesses: [LetterSet; 6]) -> Self {
        Self { answer, guesses }
    }

    /// Return the answer of the clique.
    #[must_use]
    pub const fn answer(&self) -> &LetterSet {
        &self.answer
    }

    /// Return the guesses of the clique.
    #[must_use]
    pub const fn guesses(&self) -> &[LetterSet; 6] {
        &self.guesses
    }

    /// Sort a 6-set of guesses (assuming that the triples are sorted).
    pub fn sort(t1: [LetterSet; 3], t2: [LetterSet; 3]) -> [LetterSet; 6] {
        let compare_and_swap = |lettersets: &mut [LetterSet; 6], i: usize, j: usize| {
            if lettersets[i] > lettersets[j] {
                lettersets.swap(i, j);
            }
        };

        let mut lettersets = [t1[0], t1[1], t1[2], t2[0], t2[1], t2[2]];

        compare_and_swap(&mut lettersets, 0, 3);
        compare_and_swap(&mut lettersets, 1, 4);
        compare_and_swap(&mut lettersets, 2, 5);
        compare_and_swap(&mut lettersets, 1, 3);
        compare_and_swap(&mut lettersets, 2, 4);
        compare_and_swap(&mut lettersets, 2, 3);
        compare_and_swap(&mut lettersets, 1, 2);
        compare_and_swap(&mut lettersets, 4, 5);
        compare_and_swap(&mut lettersets, 3, 4);

        lettersets
    }
}

/// A triple of disjoint lettersets.
///
/// The `Triple` has a mask that represents the union of its lettersets. This
/// allows for fast disjointness checks.
#[derive(Debug, Clone, Copy)]
struct Triple {
    lettersets: [LetterSet; 3],
    mask: LetterSet,
}

impl Triple {
    /// Construct a new `Triple` with the given lettersets.
    const fn new(ls1: LetterSet, ls2: LetterSet, ls3: LetterSet) -> Self {
        let lettersets = [ls1, ls2, ls3];
        let mask = ls1.union(ls2).union(ls3);
        Self { lettersets, mask }
    }

    /// Check if two triples are disjoint.
    const fn disjoint(&self, other: Self) -> bool {
        self.mask.disjoint(other.mask)
    }
}
