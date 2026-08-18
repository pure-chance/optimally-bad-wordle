use std::collections::HashSet;

use itertools::Itertools;

use crate::packer::Packing;
use crate::realizer::WrongWordleSolution;
use crate::signature::Signature;

/// Checks whether the solutions contains a unique set of disjoint packings.
///
/// Verification does _not_ check that the set of solutions are complete, but it
/// does check 2 properties:
///
/// - Each solution is a valid disjoint packing.
/// - There are no duplicate solutions (same set of words in different order).
pub fn verify_solutions(solutions: &[WrongWordleSolution]) -> bool {
    // Verify that each solution is a disjoint packing.
    if !solutions.iter().all(verify_solution) {
        return false;
    }

    // Verify that the solutions are unique (no permutations of the same words).
    let mut seen_sets = HashSet::new();
    for solution in solutions {
        let a = solution.answer();
        let [g1, g2, g3, g4, g5, g6] = solution.guesses();
        let mut words = vec![a, g1, g2, g3, g4, g5, g6];
        words.sort_unstable();
        if !seen_sets.insert(words) {
            return false;
        }
    }

    true
}

/// Verifies that a `WrongWordleSolution` is a valid disjoint packing.
fn verify_solution(solution: &WrongWordleSolution) -> bool {
    let packing = convert_solution_to_signatures(solution);
    let signatures = [
        *packing.answer(),
        packing.guesses()[0],
        packing.guesses()[1],
        packing.guesses()[2],
        packing.guesses()[3],
        packing.guesses()[4],
        packing.guesses()[5],
    ];
    signatures
        .iter()
        .array_combinations::<2>()
        .all(|[&a, &b]| a.disjoint(b))
}

/// Converts a `WrongWordleSolution` to a set of `Signature`s.
fn convert_solution_to_signatures(solution: &WrongWordleSolution) -> Packing {
    let answer_signature = Signature::new(solution.answer());
    let [g1, g2, g3, g4, g5, g6] = solution.guesses();
    let mut guess_signatures = [
        Signature::new(g1),
        Signature::new(g2),
        Signature::new(g3),
        Signature::new(g4),
        Signature::new(g5),
        Signature::new(g6),
    ];
    guess_signatures.sort_unstable();
    Packing::from_signatures(answer_signature, guess_signatures)
}
