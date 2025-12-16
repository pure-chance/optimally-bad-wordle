//! Identify all valid disjoint packings.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::signature::Signature;

/// Packs signatures to identify all valid disjoint packings.
///
/// The `Packer` solves the core combinatorial problem: finding all
/// combinations of one answer signature and six guess signatures where all
/// seven are pairwise disjoint (share no letters in common). This ensures the
/// guesses provide zero information about the answer.
///
/// # Algorithm
///
/// The packer employs a three-stage process, executed in parallel for each
/// answer:
///
/// **1. Enumerate Triples**
///
/// For each answer a, the algorithm first eliminates all guesses that share any
/// letters with a, producing Compatible(a). This reduces the number of
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
pub struct Packer {
    /// All unique signatures from answers.
    answer_sets: Box<[Signature]>,
    /// All unique signatures from guesses.
    guess_sets: Box<[Signature]>,
}

impl Packer {
    /// Construct a new `Packer` with the given answers and guesses.
    #[must_use]
    pub fn new(answers: &[&str], guesses: &[&str]) -> Self {
        let answer_sets: Box<[Signature]> = answers
            .iter()
            .map(|&w| w.into())
            .unique()
            .sorted()
            .collect();
        let guess_sets: Box<[Signature]> = guesses
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
    ///
    /// Packings are found for each answer in parallel and then merged into a
    /// single set.
    ///
    /// This function shows the progress of the packing process.
    pub fn pack_with_progress(&self) -> HashSet<Packing> {
        let progress = AtomicUsize::new(0);

        let pb = ProgressBar::new(self.answer_sets.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{msg:.cyan} [{bar:25}] {pos}/{len} answers")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message("Packing");

        let result = self
            .answer_sets
            .par_iter()
            .map(|answer| {
                let _ = progress.fetch_add(1, Ordering::Relaxed) + 1;
                pb.inc(1);
                self.pack_for_answer(*answer)
            })
            .reduce(HashSet::new, |mut acc, packings_for_answer| {
                acc.extend(packings_for_answer);
                acc
            });

        pb.finish_and_clear();
        result
    }

    /// Find all possible packings of answer + 6 guesses.
    ///
    /// Packings are found for each answer in parallel and then merged into a
    /// single set.
    pub fn pack(&self) -> HashSet<Packing> {
        self.answer_sets
            .par_iter()
            .map(|answer| self.pack_for_answer(*answer))
            .reduce(HashSet::new, |mut acc, packings_for_answer| {
                acc.extend(packings_for_answer);
                acc
            })
    }

    /// Find all possible packings of this particular answer + 6 guesses.
    ///
    /// This is done by (1) finding all triples for the answer, (2) partitioning
    /// them by signature, and (3) scanning and merging the partitions. Look at
    /// the documentation of `Packer` for more details.
    #[must_use]
    pub fn pack_for_answer(&self, answer: Signature) -> HashSet<Packing> {
        let triples = Self::find_triples_for_answer(answer, &self.guess_sets);
        let partition = Signature::new("seaoriltnu");
        let partitions = Self::partition_triples_by_signature(&triples, partition);
        let packings = Self::scan_and_merge_partitions(&partitions, answer);
        packings.into_iter().collect()
    }

    /// Find all triples for this particular answer.
    ///
    /// **Correctness**: All triples are unique and sorted by construction.
    fn find_triples_for_answer(answer: Signature, guess_sets: &[Signature]) -> Vec<Triple> {
        let guess_sets: Vec<Signature> = guess_sets
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

    /// Partition triples by some partition signature.
    ///
    /// Returns a `HashMap` where the keys are the intersection of each triple's
    /// mask with the partition, and the values are vectors of triples that
    /// share the same intersection.
    fn partition_triples_by_signature(
        triples: &[Triple],
        partition: Signature,
    ) -> HashMap<Signature, Vec<Triple>> {
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
        partition: &HashMap<Signature, Vec<Triple>>,
        answer: Signature,
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
                let guesses = Packing::sort(triple1.signatures, triple2.signatures);
                packings.push(Packing::new(answer, guesses));
            }
        }
        packings
    }
}

/// A solution—a 7-packing of disjoint signatures (1 answer + 6 guesses).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packing {
    answer: Signature,
    guesses: [Signature; 6],
}

impl Packing {
    /// Construct a new `Packing` with the given answer and guesses.
    ///
    /// **Correctness**: A packing's guesses must be sorted to ensure that
    /// comparisons depend exclusively on membership, and not order.
    #[must_use]
    pub const fn new(answer: Signature, guesses: [Signature; 6]) -> Self {
        Self { answer, guesses }
    }

    /// Return the answer of the clique.
    #[must_use]
    pub const fn answer(&self) -> &Signature {
        &self.answer
    }

    /// Return the guesses of the clique.
    #[must_use]
    pub const fn guesses(&self) -> &[Signature; 6] {
        &self.guesses
    }

    /// Sort a 6-set of guesses (assuming that the triples are sorted).
    #[must_use]
    pub fn sort(t1: [Signature; 3], t2: [Signature; 3]) -> [Signature; 6] {
        let compare_and_swap = |signatures: &mut [Signature; 6], i: usize, j: usize| {
            if signatures[i] > signatures[j] {
                signatures.swap(i, j);
            }
        };

        let mut signatures = [t1[0], t1[1], t1[2], t2[0], t2[1], t2[2]];

        compare_and_swap(&mut signatures, 0, 3);
        compare_and_swap(&mut signatures, 1, 4);
        compare_and_swap(&mut signatures, 2, 5);
        compare_and_swap(&mut signatures, 1, 3);
        compare_and_swap(&mut signatures, 2, 4);
        compare_and_swap(&mut signatures, 2, 3);
        compare_and_swap(&mut signatures, 1, 2);
        compare_and_swap(&mut signatures, 4, 5);
        compare_and_swap(&mut signatures, 3, 4);

        signatures
    }
}

/// A triple of disjoint signatures.
///
/// The `Triple` has a mask that represents the union of its signatures. This
/// allows for fast disjointness checks.
#[derive(Debug, Clone, Copy)]
struct Triple {
    signatures: [Signature; 3],
    mask: Signature,
}

impl Triple {
    /// Construct a new `Triple` with the given signatures.
    const fn new(ls1: Signature, ls2: Signature, ls3: Signature) -> Self {
        let signatures = [ls1, ls2, ls3];
        let mask = ls1.union(ls2).union(ls3);
        Self { signatures, mask }
    }

    /// Check if two triples are disjoint.
    const fn disjoint(&self, other: Self) -> bool {
        self.mask.disjoint(other.mask)
    }
}
