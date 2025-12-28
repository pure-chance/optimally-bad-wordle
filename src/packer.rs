//! Find all disjoint packings of signatures.

use std::collections::{HashMap, HashSet};

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::signature::Signature;

/// Packs all disjoint packings of signatures.
///
/// The `Packer` solves the core combinatorial problem: finding all
/// combinations of one answer signature and six guess signatures where all
/// seven are pairwise disjoint (share no letters in common). This ensures
/// guesses provide zero information about the answer.
///
/// # Algorithm
///
/// The algorithm employs a three-stage process, executed in parallel for each
/// answer:
///
/// **1. Enumerate Triples**
///
/// For each answer, the algorithm eliminates all guesses that share letters
/// with it, then enumerates all valid disjoint triples (g₁, g₂, g₃) from
/// the remaining candidates.
///
/// **2. Partition Triples**
///
/// Each triple is partitioned based on its intersection with the 10 most
/// common letters. Triples with identical partition signatures are grouped
/// together, creating up to 1,024 possible bins.
///
/// **3. Compare Triple Pairs**
///
/// The algorithm compares pairs of partition bins rather than individual
/// triples. If bin signatures are disjoint, all cross-bin triple pairs
/// are verified for full disjointness. This reduces the comparison space
/// from O(T²) to hundreds of thousands of operations.
///
/// # Runtime
///
/// In practice, the algorithm runs in ~20 seconds.
pub struct Packer {}

impl Packer {
    /// Find all possible packings (with progress display).
    #[must_use]
    pub fn pack(answers: &[&str], guesses: &[&str]) -> HashSet<Packing> {
        let (answer_signatures, guess_signatures) = Self::compile_signatures(answers, guesses);

        let pb = ProgressBar::new(answer_signatures.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{msg:.cyan} [{bar:25}] {pos}/{len} answers")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message("Packing");

        let packings = answer_signatures
            .par_iter()
            .map(|&answer| {
                pb.inc(1);
                Self::pack_for_answer(&guess_signatures, answer)
            })
            .reduce(HashSet::new, |mut acc, packings_for_answer| {
                acc.extend(packings_for_answer);
                acc
            });

        pb.finish_and_clear();
        packings
    }

    /// Find all possible packings (without progress display).
    #[must_use]
    pub fn pack_signatures(answers: &[&str], guesses: &[&str]) -> HashSet<Packing> {
        let (answer_signatures, guess_signatures) = Self::compile_signatures(answers, guesses);
        answer_signatures
            .par_iter()
            .map(|answer| Self::pack_for_answer(&guess_signatures, *answer))
            .reduce(HashSet::new, |mut acc, packings_for_answer| {
                acc.extend(packings_for_answer);
                acc
            })
    }

    /// Convert word lists to unique, sorted signatures.
    #[must_use]
    pub fn compile_signatures(
        answers: &[&str],
        guesses: &[&str],
    ) -> (Box<[Signature]>, Box<[Signature]>) {
        let answer_signatures: Box<[Signature]> = answers
            .iter()
            .map(|&w| w.into())
            .unique()
            .sorted()
            .collect();
        let guess_signatures: Box<[Signature]> = guesses
            .iter()
            .map(|&w| w.into())
            .unique()
            .sorted()
            .collect();
        (answer_signatures, guess_signatures)
    }

    /// Find all packings for a specific answer signature.
    ///
    /// This is done by (1) finding all triples for the answer, (2) partitioning
    /// them by signature, and (3) scanning and merging the partitions. Look at
    /// the documentation of `Packer` for more details.
    #[must_use]
    pub fn pack_for_answer(guess_signatures: &[Signature], answer: Signature) -> HashSet<Packing> {
        let triples = Self::find_triples_for_answer(answer, guess_signatures);
        let partition_key = Signature::from_mask(0b00000111100110100100010001); // "seaoriltnu"
        let partitions = Self::partition_triples_by_signature(&triples, partition_key);
        let packings = Self::scan_and_merge_partitions(&partitions, answer);
        packings.into_iter().collect()
    }

    /// Find all disjoint triples compatible with the given answer.
    ///
    /// **Correctness**: All triples are unique and sorted by construction.
    fn find_triples_for_answer(answer: Signature, guess_signatures: &[Signature]) -> Vec<Triple> {
        let candidates: Vec<Signature> = guess_signatures
            .iter()
            .copied()
            .filter(|&ls| ls.disjoint(answer))
            .collect();
        let mut triples = Vec::new();
        for (i, &ls1) in candidates.iter().enumerate() {
            for (j, &ls2) in candidates.iter().enumerate().skip(i + 1) {
                if !ls1.disjoint(ls2) {
                    continue;
                }
                for (_, &ls3) in candidates.iter().enumerate().skip(j + 1) {
                    if ls1.union(ls2).disjoint(ls3) {
                        let triple = Triple::new(ls1, ls2, ls3);
                        triples.push(triple);
                    }
                }
            }
        }
        triples
    }

    /// Partition triples using a partition key.
    ///
    /// Groups triples by their intersection with the partition key, enabling
    /// efficient pruning during the merge phase.
    fn partition_triples_by_signature(
        triples: &[Triple],
        partition_key: Signature,
    ) -> HashMap<Signature, Vec<Triple>> {
        let mut partitions = HashMap::new();
        for &triple in triples {
            let key = partition_key.intersection(triple.mask);
            partitions.entry(key).or_insert_with(Vec::new).push(triple);
        }
        partitions
    }

    /// Merge disjoint triples into packings using partition-based pruning.
    ///
    /// Only compares triples from partitions with disjoint keys, then
    /// verifies full disjointness before creating packings.
    fn scan_and_merge_partitions(
        partitions: &HashMap<Signature, Vec<Triple>>,
        answer: Signature,
    ) -> Vec<Packing> {
        let mut packings = Vec::new();
        for keys in partitions.iter().combinations_with_replacement(2) {
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

/// A disjoint packing of one answer and six guess signatures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packing {
    answer: Signature,
    guesses: [Signature; 6],
}

impl Packing {
    /// Construct a new `Packing`.
    ///
    /// **Correctness**: A packing's guesses must be sorted to ensure
    /// membership-based comparisons.
    #[must_use]
    pub const fn new(answer: Signature, guesses: [Signature; 6]) -> Self {
        Self { answer, guesses }
    }

    /// Return the answer signature.
    #[must_use]
    pub const fn answer(&self) -> &Signature {
        &self.answer
    }

    /// Return the guess signatures.
    #[must_use]
    pub const fn guesses(&self) -> &[Signature; 6] {
        &self.guesses
    }

    /// Sort six signatures from two triples into canonical order.
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
    const fn disjoint(self, other: Self) -> bool {
        self.mask.disjoint(other.mask)
    }
}
