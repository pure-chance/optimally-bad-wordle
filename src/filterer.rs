use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::letterset::LetterSet;

pub struct Filterer {
    /// All unique lettersets from answers.
    answer_sets: Vec<LetterSet>,
    /// All unique lettersets from guesses.
    guess_sets: Vec<LetterSet>,
}

impl Filterer {
    /// Construct a new `Filterer` with the given answers and guesses.
    #[must_use]
    pub fn new(answers: &[&str], guesses: &[&str]) -> Self {
        let answer_sets: Vec<LetterSet> = answers.iter().map(|&w| w.into()).unique().collect();
        let guess_sets: Vec<LetterSet> = guesses.iter().map(|&w| w.into()).unique().collect();
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
        Self::compare_compatible_triples(&partitions, answer)
            .into_iter()
            .collect()
    }
    /// Find all triples for this particular answer.
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
                };
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
    /// Compare compatible triples in the partition.
    fn compare_compatible_triples(
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
                let [ls1, ls2, ls3] = triple1.lettersets;
                let [ls4, ls5, ls6] = triple2.lettersets;
                packings.push(Packing::new(answer, [ls1, ls2, ls3, ls4, ls5, ls6]));
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
    /// A clique is sorted to ensure that comparisons depend exclusively on
    /// membership.
    #[must_use]
    pub fn new(answer: LetterSet, mut guesses: [LetterSet; 6]) -> Self {
        guesses.sort();
        Self { answer, guesses }
    }
    /// Return the answer of the clique.
    #[must_use]
    pub const fn answer(&self) -> LetterSet {
        self.answer
    }
    /// Return the guesses of the clique.
    #[must_use]
    pub const fn guesses(&self) -> &[LetterSet; 6] {
        &self.guesses
    }
}

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
