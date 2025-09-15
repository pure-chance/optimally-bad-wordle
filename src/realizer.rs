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
    pub fn new(answers: &[&str], guesses: &[&str]) -> Self {
        let mut answer_lettersets = HashMap::new();
        for &answer in answers {
            let letterset = LetterSet::new(answer);
            answer_lettersets
                .entry(letterset)
                .or_insert_with(Vec::new)
                .push(answer.to_string());
        }

        let mut guess_lettersets = HashMap::new();
        for &guess in guesses {
            let letterset = LetterSet::new(guess);
            guess_lettersets
                .entry(letterset)
                .or_insert_with(Vec::new)
                .push(guess.to_string());
        }

        Self {
            answer_realizations: answer_lettersets,
            guess_realizations: guess_lettersets,
        }
    }
    /// Realizes the set of packings into optimally bad Wordle solutions.
    pub fn realize(&self, solutions: &HashSet<Packing>) -> Vec<BadWordleSolution> {
        solutions
            .par_iter()
            .flat_map(|solution| self.realize_solution(solution))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
    /// Realizes a packing into an optimally bad Wordle solution.
    pub fn realize_solution(&self, solution: &Packing) -> Vec<BadWordleSolution> {
        let a = solution.answer();
        let [g1, g2, g3, g4, g5, g6] = solution.guesses();
        let combinations: [Vec<String>; 7] = [
            self.answer_realizations.get(&a).unwrap().clone(),
            self.guess_realizations.get(&g1).unwrap().clone(),
            self.guess_realizations.get(&g2).unwrap().clone(),
            self.guess_realizations.get(&g3).unwrap().clone(),
            self.guess_realizations.get(&g4).unwrap().clone(),
            self.guess_realizations.get(&g5).unwrap().clone(),
            self.guess_realizations.get(&g6).unwrap().clone(),
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
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct BadWordleSolution {
    answer: String,
    guesses: [String; 6],
}

impl BadWordleSolution {
    /// Construct a new `BadWordleSolution` from an answer and a list of guesses.
    pub fn new(answer: String, guesses: [String; 6]) -> Self {
        let mut guesses = guesses;
        guesses.sort();
        Self { answer, guesses }
    }
    /// Returns the answer of the `BadWordleSolution`.
    pub fn answer(&self) -> &str {
        &self.answer
    }
    /// Returns the guesses of the `BadWordleSolution`.
    pub fn guesses(&self) -> &[String; 6] {
        &self.guesses
    }
}

impl PartialEq for BadWordleSolution {
    fn eq(&self, other: &Self) -> bool {
        if self.answer == other.answer {
            return true;
        }
        let solution_set: HashSet<BadWordleSolution> = HashSet::from([self.clone(), other.clone()]);
        solution_set.len() == 1
    }
}

impl Eq for BadWordleSolution {}
