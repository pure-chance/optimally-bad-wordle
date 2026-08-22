use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashSet as HashSet};

use crate::packer::Packing;
use crate::realizer::WrongWordleSolution;
use crate::signature::Signature;

/// Checks whether the solutions contains a unique set of disjoint packings.
///
/// Verification does not check that the set of solutions are complete, but it
/// does check 2 properties:
///
/// - Each solution is a valid disjoint packing.
/// - There are no duplicate solutions (same set of words in different order).
///
/// # Errors
///
/// If the solution is invalid, `VerificationError::InvalidSolution` is
/// returned. If there are duplicate solutions,
/// `VerificationError::DuplicateSolution` is returned.
pub fn verify_solutions(solutions: &[WrongWordleSolution]) -> Result<(), VerificationError> {
    // Verify that each solution is a disjoint packing.
    for solution in solutions {
        if !verify_solution(solution) {
            return Err(VerificationError::InvalidSolution(solution.clone()));
        }
    }

    // Verify that there are no duplicate solutions.
    //
    // Note that the solutions are normalized when constructed, so we can use
    // the default hash function here.
    let mut seen = HashSet::with_capacity_and_hasher(solutions.len(), FxBuildHasher);
    for solution in solutions {
        if !seen.insert(solution) {
            return Err(VerificationError::DuplicateSolution(solution.clone()));
        }
    }

    Ok(())
}

/// Verifies that a `WrongWordleSolution` is a valid disjoint packing.
fn verify_solution(solution: &WrongWordleSolution) -> bool {
    let packing = convert_solution_to_normalized_signatures(solution);
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

/// Returns the normalized (sorted) packing of the `WrongWordleSolution`.
fn convert_solution_to_normalized_signatures(solution: &WrongWordleSolution) -> Packing {
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

/// The set of possible (checkable) errors that invalidate a set of solutions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationError {
    InvalidSolution(WrongWordleSolution),
    DuplicateSolution(WrongWordleSolution),
}

impl std::error::Error for VerificationError {}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSolution(invalid_solution) => {
                write!(f, "invalid solution: {invalid_solution}")
            }
            Self::DuplicateSolution(duplication_solution) => {
                write!(f, "duplicate solution: {duplication_solution}")
            }
        }
    }
}
