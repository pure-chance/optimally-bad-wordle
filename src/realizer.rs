//! Realize packings into Wordle solutions (that are optimally bad).

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::packer::Packing;
use crate::signature::Signature;

/// Realizes packings into Wordle solutions (that are optimally bad).
///
/// Specifically, the realizer finds all Wordle solutions that map to the
/// disjoint packings found by the packer.
///
/// # Algorithm
///
/// Given a packing (a, g₁, g₂, g₃, g₄, g₅, g₆) where each element is a
/// signature, the realizer:
///
/// 1. looks up all words that correspond to each signature using a precomputed
///    dictionary, and
/// 2. enumerates the Cartesian product across all seven positions.
///
/// # Example
///
/// Consider the following (simplified) packing:
///
/// ```text
/// a  = {a,e,l,s,t} → ["least", "slate"]
/// g₁ = {b,i,k,l,n} → ["blink"]
/// g₂ = {c,o,r,u,y} → ["corny", "court", "curvy"]
/// ```
///
/// The realizer generates 2 × 1 × 3 = 6 solutions:
///
/// ```text
/// ("least", "blink", "corny"),
/// ("least", "blink", "court"),
/// ("least", "blink", "curvy"),
/// ("slate", "blink", "corny"),
/// ("slate", "blink", "court"),
/// ("slate", "blink", "curvy"),
/// ```
pub struct Realizer {}

impl Realizer {
    /// Realizes the set of packings into optimally bad Wordle solutions.
    ///
    /// While processing, displays the progress of the realization process.
    #[must_use]
    pub fn realize(
        answers: &[&str],
        guesses: &[&str],
        packings: &HashSet<Packing>,
    ) -> HashSet<BadWordleSolution> {
        let (answer_realizations, guess_realizations) =
            Self::compile_realizations(answers, guesses);

        let pb = ProgressBar::new(packings.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{msg:.cyan} [{bar:25}] {pos}/{len} packings")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message("Realizing");

        let solutions = packings
            .par_iter()
            .flat_map(|packing| {
                pb.inc(1);
                Self::realize_packing(&answer_realizations, &guess_realizations, packing)
            })
            .collect();

        pb.finish_and_clear();
        solutions
    }

    /// Realizes the set of packings into optimally bad Wordle solutions.
    #[must_use]
    pub fn realize_packings(
        answers: &[&str],
        guesses: &[&str],
        packings: &HashSet<Packing>,
    ) -> HashSet<BadWordleSolution> {
        let (answer_realizations, guess_realizations) =
            Self::compile_realizations(answers, guesses);
        packings
            .par_iter()
            .flat_map(|packing| {
                Self::realize_packing(&answer_realizations, &guess_realizations, packing)
            })
            .collect()
    }

    /// Compile all the unique answer and guesses realizations.
    #[must_use]
    pub fn compile_realizations(
        answers: &[&str],
        guesses: &[&str],
    ) -> (
        HashMap<Signature, Vec<String>>,
        HashMap<Signature, Vec<String>>,
    ) {
        let answer_realizations: HashMap<Signature, Vec<String>> = answers
            .iter()
            .map(|&answer| (answer.into(), answer.to_string()))
            .into_group_map();
        let guess_realizations: HashMap<Signature, Vec<String>> = guesses
            .iter()
            .map(|&guess| (guess.into(), guess.to_string()))
            .into_group_map();
        (answer_realizations, guess_realizations)
    }

    /// Realizes a packing into an optimally bad Wordle solution.
    ///
    /// # Panics
    ///
    /// Panics if any of the signatures in the packing are not found in either
    /// the `answer_realizations` or `guess_realizations`. This will never
    /// happen if the signatures are from the answers and guesses word lists.
    #[must_use]
    pub fn realize_packing(
        answer_realizations: &HashMap<Signature, Vec<String>>,
        guess_realizations: &HashMap<Signature, Vec<String>>,
        packing: &Packing,
    ) -> HashSet<BadWordleSolution> {
        let a = &packing.answer();
        let [g1, g2, g3, g4, g5, g6] = packing.guesses();
        let combinations = [
            answer_realizations[a].clone(),
            guess_realizations[g1].clone(),
            guess_realizations[g2].clone(),
            guess_realizations[g3].clone(),
            guess_realizations[g4].clone(),
            guess_realizations[g5].clone(),
            guess_realizations[g6].clone(),
        ];
        combinations
            .into_iter()
            .multi_cartesian_product()
            .par_bridge()
            .map(|v| {
                let [a, g1, g2, g3, g4, g5, g6] = v.try_into().unwrap();
                BadWordleSolution::new(a, [g1, g2, g3, g4, g5, g6])
            })
            .collect()
    }
}

/// An optimally bad Wordle solution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BadWordleSolution {
    answer: String,
    guesses: [String; 6],
}

impl BadWordleSolution {
    /// Construct a new `BadWordleSolution` from an answer and a list of
    /// guesses.
    #[must_use]
    pub fn new(answer: String, mut guesses: [String; 6]) -> Self {
        guesses.sort_unstable();
        Self { answer, guesses }
    }

    /// Returns the answer of the `BadWordleSolution`.
    #[must_use]
    pub const fn answer(&self) -> &String {
        &self.answer
    }

    /// Returns the guesses of the `BadWordleSolution`.
    #[must_use]
    pub const fn guesses(&self) -> &[String; 6] {
        &self.guesses
    }
}
