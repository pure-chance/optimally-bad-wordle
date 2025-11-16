use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::filterer::Packing;
use crate::letterset::LetterSet;

/// Realizes letterset solutions back into maximally bad Wordle solutions.
pub struct Realizer {
    /// A map from answer lettersets to their realizations.
    answer_realizations: HashMap<LetterSet, Vec<String>>,
    /// A map from guess lettersets to their realizations.
    guess_realizations: HashMap<LetterSet, Vec<String>>,
}

impl Realizer {
    /// Construct a new `Realizer` from a list of answers and guesses.
    #[must_use]
    pub fn new(answers: &[&str], guesses: &[&str]) -> Self {
        let answer_realizations: HashMap<LetterSet, Vec<String>> = answers
            .iter()
            .map(|&answer| (LetterSet::new(answer), answer.to_string()))
            .into_group_map();

        let guess_realizations: HashMap<LetterSet, Vec<String>> = guesses
            .iter()
            .map(|&guess| (LetterSet::new(guess), guess.to_string()))
            .into_group_map();

        Self {
            answer_realizations,
            guess_realizations,
        }
    }
    /// Realizes the set of packings into optimally bad Wordle solutions.
    #[must_use]
    pub fn realize(&self, solutions: &HashSet<Packing>) -> HashSet<BadWordleSolution> {
        solutions
            .par_iter()
            .flat_map(|solution| self.realize_solution(solution))
            .collect()
    }
    /// Realizes a packing into an optimally bad Wordle solution.
    ///
    /// # Panics
    ///
    /// Panics if any of the lettersets in the packing are not found in either
    /// the `answer_realizations` or `guess_realizations`. This will never
    /// happen if the lettersets are from the answers and guesses word list.
    #[must_use]
    pub fn realize_solution(&self, solution: &Packing) -> HashSet<BadWordleSolution> {
        let a = &solution.answer();
        let [g1, g2, g3, g4, g5, g6] = solution.guesses();

        let combinations = [
            self.answer_realizations[a].clone(),
            self.guess_realizations[g1].clone(),
            self.guess_realizations[g2].clone(),
            self.guess_realizations[g3].clone(),
            self.guess_realizations[g4].clone(),
            self.guess_realizations[g5].clone(),
            self.guess_realizations[g6].clone(),
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

/// A maximally bad Wordle solution.
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
