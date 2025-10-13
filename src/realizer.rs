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
        let realizations: HashSet<_> = solutions
            .par_iter()
            .flat_map(|solution| self.realize_solution(solution))
            .collect();
        realizations
    }
    /// Realizes a packing into an optimally bad Wordle solution.
    ///
    /// # Panics
    ///
    /// Panics if any of the lettersets in the packing are not found either the
    /// `answer_realizations` or `guess_realizations`. This will never happen if
    /// the packing is valid.
    #[must_use]
    pub fn realize_solution(&self, solution: &Packing) -> HashSet<BadWordleSolution> {
        let a = &solution.answer();
        let [g1, g2, g3, g4, g5, g6] = solution.guesses();
        let combinations: [Vec<String>; 7] = [
            self.answer_realizations.get(a).unwrap().clone(),
            self.guess_realizations.get(g1).unwrap().clone(),
            self.guess_realizations.get(g2).unwrap().clone(),
            self.guess_realizations.get(g3).unwrap().clone(),
            self.guess_realizations.get(g4).unwrap().clone(),
            self.guess_realizations.get(g5).unwrap().clone(),
            self.guess_realizations.get(g6).unwrap().clone(),
        ];

        combinations
            .into_iter()
            .multi_cartesian_product()
            .par_bridge()
            .map(|v| {
                let [a, g1, g2, g3, g4, g5, g6]: [String; 7] = v.try_into().unwrap();
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
    pub fn new(answer: String, guesses: [String; 6]) -> Self {
        let mut guesses = guesses;
        guesses.sort();
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
