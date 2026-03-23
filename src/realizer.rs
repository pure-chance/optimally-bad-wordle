//! Realize packings into Wordle solutions (that are optimally bad).

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::{Deserialize, Serialize};

use crate::packer::Packing;
use crate::signature::Signature;

/// Realizes packings into Wordle solutions (that are optimally bad).
///
/// Realization takes disjoint signature packings and generates all possible
/// word combinations by looking up each signature's corresponding words.
///
/// # Algorithm
///
/// For each packing (a, g₁, g₂, g₃, g₄, g₅, g₆):
/// 1. Look up all words corresponding to each signature
/// 2. Generate the Cartesian product of all word combinations
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
#[must_use]
pub fn realize(
    answers: &[&str],
    guesses: &[&str],
    packings: &HashSet<Packing>,
) -> HashSet<BadWordleSolution> {
    let answer_realizations = compile_realizations(answers);
    let guess_realizations = compile_realizations(guesses);

    let pb = ProgressBar::new(packings.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{msg:.cyan} [{bar:25}] {pos}/{len} packings")
            .expect("Progress bar template is invalid")
            .progress_chars("=> "),
    );
    pb.set_message("Realizing");

    let solutions = packings
        .par_iter()
        .flat_map(|packing| {
            pb.inc(1);
            realize_packing(&answer_realizations, &guess_realizations, packing)
        })
        .collect();

    pb.finish_and_clear();
    solutions
}

/// Build signature-to-words lookup tables.
#[must_use]
pub fn compile_realizations(words: &[&str]) -> HashMap<Signature, Vec<String>> {
    let mut map = HashMap::default();
    for &word in words {
        map.entry(Signature::from(word))
            .or_insert_with(Vec::new)
            .push(word.to_string());
    }
    map
}

/// Convert a single packing into an (optimally bad) Wordle solutions.
///
/// # Panics
///
/// Panics if any signature in the packing is not found in the lookup
/// tables.
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
        .map(|v| {
            let [a, g1, g2, g3, g4, g5, g6] = v.try_into().unwrap();
            BadWordleSolution::new(a, [g1, g2, g3, g4, g5, g6])
        })
        .collect()
}

/// An optimally bad Wordle solution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BadWordleSolution {
    answer: String,
    guesses: [String; 6],
}

impl BadWordleSolution {
    /// Construct a new `BadWordleSolution`.
    #[must_use]
    pub fn new(answer: String, mut guesses: [String; 6]) -> Self {
        guesses.sort_unstable();
        Self { answer, guesses }
    }

    /// Returns the answer word.
    #[must_use]
    pub const fn answer(&self) -> &String {
        &self.answer
    }

    /// Returns the guess words.
    #[must_use]
    pub const fn guesses(&self) -> &[String; 6] {
        &self.guesses
    }
}
