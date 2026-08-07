//! Realize packings into Wordle solutions (that are optimally wrong).

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Serialize;

use crate::packer::Packing;
use crate::signature::Signature;

/// Realizes packings into Wordle solutions (that are optimally wrong).
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
    answers: &[&'static str],
    guesses: &[&'static str],
    packings: &[Packing],
) -> HashSet<WrongWordleSolution> {
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
pub fn compile_realizations(words: &[&'static str]) -> HashMap<Signature, Vec<&'static str>> {
    let mut map = HashMap::default();
    for &word in words {
        map.entry(Signature::new(word))
            .or_insert_with(Vec::new)
            .push(word);
    }
    map
}

/// Convert a single packing into an (optimally wrong) Wordle solutions.
///
/// # Panics
///
/// Panics if any signature in the packing is not found in the lookup
/// tables.
#[must_use]
pub fn realize_packing(
    answer_realizations: &HashMap<Signature, Vec<&'static str>>,
    guess_realizations: &HashMap<Signature, Vec<&'static str>>,
    packing: &Packing,
) -> HashSet<WrongWordleSolution> {
    let a = &packing.answer();
    let [g1, g2, g3, g4, g5, g6] = packing.guesses();
    let combinations = [
        answer_realizations[a].as_slice(),
        guess_realizations[g1].as_slice(),
        guess_realizations[g2].as_slice(),
        guess_realizations[g3].as_slice(),
        guess_realizations[g4].as_slice(),
        guess_realizations[g5].as_slice(),
        guess_realizations[g6].as_slice(),
    ];
    combinations
        .into_iter()
        .multi_cartesian_product()
        .map(|v| {
            let [a, g1, g2, g3, g4, g5, g6] = v.try_into().unwrap();
            WrongWordleSolution::new(a, [g1, g2, g3, g4, g5, g6])
        })
        .collect()
}

/// An optimally wrong Wordle solution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct WrongWordleSolution {
    answer: &'static str,
    guesses: [&'static str; 6],
}

impl WrongWordleSolution {
    /// Construct a new `WrongWordleSolution`.
    #[must_use]
    pub fn new(answer: &'static str, mut guesses: [&'static str; 6]) -> Self {
        guesses.sort_unstable();
        Self { answer, guesses }
    }

    /// Returns the answer word.
    #[must_use]
    pub const fn answer(&self) -> &'static str {
        self.answer
    }

    /// Returns the guess words.
    #[must_use]
    pub const fn guesses(&self) -> &[&'static str; 6] {
        &self.guesses
    }
}
