//! Find all disjoint packings of signatures.

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;

use crate::signature::Signature;

/// Packs all disjoint packings of signatures.
///
/// Packing solves the core combinatorial problem: finding all combinations of
/// one answer signature and six guess signatures where all seven are pairwise
/// disjoint (share no letters in common). This ensures guesses provide zero
/// information about the answer.
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
#[must_use]
pub fn pack(answers: &[&str], guesses: &[&str]) -> Vec<Packing> {
    let answer_signatures = signify_words(answers);
    let guess_signatures = signify_words(guesses);

    let partition_key = create_partition_key(&guess_signatures);

    let pb = ProgressBar::new(answer_signatures.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{msg:.cyan} [{bar:25}] {pos}/{len} answers")
            .expect("Progress bar template is invalid")
            .progress_chars("=> "),
    );
    pb.set_message("Packing");

    let packings = answer_signatures
        .par_iter()
        .map(|&answer| {
            pb.inc(1);
            pack_for_answer(answer, &guess_signatures, partition_key)
        })
        .reduce(Vec::default, |mut acc, packings_for_answer| {
            acc.extend(packings_for_answer);
            acc
        });

    pb.finish_and_clear();
    packings
}

/// Convert word lists to unique, sorted signatures.
#[must_use]
pub fn signify_words(words: &[&str]) -> Box<[Signature]> {
    words
        .iter()
        .map(|&w| Signature::new(w))
        .unique()
        .sorted()
        .collect()
}

/// Create a partition key by taking the 10 most common letters across all signatures.
pub fn create_partition_key(signatures: &[Signature]) -> Signature {
    let mut frequencies = [0; 26];
    for signature in signatures {
        for i in 0..26 {
            frequencies[i] += ((signature.mask() >> i) & 1) as u32;
        }
    }

    let mut most_common: [usize; 10] = [0; 10];
    for i in 0..10 {
        let mut max_freq = 0;
        let mut max_index = 0;
        for (i, &freq) in frequencies.iter().enumerate() {
            if freq > max_freq {
                max_freq = freq;
                max_index = i;
            }
        }
        most_common[i] = max_index;
        frequencies[max_index] = 0;
    }

    let mut signature: u32 = 0;
    for &letter in &most_common {
        signature |= 1 << letter;
    }

    Signature::from_mask(signature)
}

/// Find all packings for a specific answer signature.
///
/// This is done by (1) finding all triples for the answer, (2) partitioning
/// them by signature, and (3) scanning and merging the partitions. Look at
/// the documentation of `pack` for more details.
#[must_use]
pub fn pack_for_answer(
    answer: Signature,
    guess_signatures: &[Signature],
    partition_key: Signature,
) -> Vec<Packing> {
    let triples = find_triples_for_answer(&guess_signatures, answer);
    let partitions = partition_triples_by_signature(&triples, partition_key);
    let packings = scan_and_merge_partitions(&partitions, answer);
    packings
}

/// Find all disjoint triples compatible with the given answer.
///
/// **Correctness**: All triples are unique and sorted by construction.
fn find_triples_for_answer(guess_signatures: &[Signature], answer: Signature) -> Vec<Triple> {
    let candidates: Vec<Signature> = guess_signatures
        .iter()
        .copied()
        .filter(|&sig| sig.disjoint(answer))
        .collect();

    // Pre-allocate 1/2 the maximum possible number of triples (which is C(n, 3)).
    let num_candidates = candidates.len();
    let triples_capacity_initial = num_candidates * (num_candidates - 1) * (num_candidates - 2) / 6;
    let mut triples = Vec::with_capacity(triples_capacity_initial);

    for (i, &sig_a) in candidates.iter().enumerate() {
        for (j, &sig_b) in candidates.iter().enumerate().skip(i + 1) {
            // If sig_a and sig_b are not disjoint, sig_c cannot be disjoint.
            if !sig_a.disjoint(sig_b) {
                continue;
            }
            for (_, &sig_c) in candidates.iter().enumerate().skip(j + 1) {
                if !sig_a.union(sig_b).disjoint(sig_c) {
                    continue;
                }
                let triple = Triple::new(sig_a, sig_b, sig_c);
                triples.push(triple);
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
    let mut partitions = HashMap::default();
    for &triple in triples {
        let key = partition_key.intersection(triple.union);
        partitions.entry(key).or_insert_with(Vec::new).push(triple);
    }
    partitions
}

/// Merge disjoint triples into packings using partition-based pruning.
///
/// Partition keys provide the first level of pruning: if two keys overlap,
/// every triple in one partition overlaps every triple in the other on at least
/// one partition-key letter. Only pairs of partitions with disjoint keys can
/// therefore contain compatible triples.
///
/// # Completeness
///
/// Let the six sorted guess signatures of any valid packing be
/// `x0 < x1 < x2 < x3 < x4 < x5`. Triple enumeration produces every sorted
/// three-element combination, so it necessarily contains both `[x0, x1, x2]`
/// and `[x3, x4, x5]`. These triples are disjoint, their partition keys are
/// disjoint, and [`merge_ordered_triples`] accepts them because every signature
/// in the first triple sorts before every signature in the second. Interleaved
/// decompositions of the same packing may be rejected, but this canonical
/// lower-three/upper-three decomposition is always present and accepted.
///
/// The hash map does not provide a meaningful order between two distinct
/// partitions, so each distinct partition pair is merged in both directions.
/// This ensures the partition containing the canonical lower triple is examined
/// as `lower_triples`. A partition paired with itself needs only one direction.
///
/// # Uniqueness
///
/// A pair is emitted only when all three signatures in its lower triple sort
/// before all three signatures in its upper triple. For any set of six distinct
/// signatures, exactly one of its ten unordered splits into two triples has
/// this property: the split between its third- and fourth-smallest signatures.
/// Consequently, every valid packing is emitted exactly once, without a final
/// deduplication pass.
fn scan_and_merge_partitions(
    partitions: &HashMap<Signature, Vec<Triple>>,
    answer: Signature,
) -> Vec<Packing> {
    let mut packings = Vec::new();

    for part in partitions.iter().combinations_with_replacement(2) {
        let (&key_a, triples_a) = part[0];
        let (&key_b, triples_b) = part[1];

        if !key_a.disjoint(key_b) {
            continue;
        }

        // Treat A as the lower triple and B as the upper triple.
        merge_ordered_triples(triples_a, triples_b, answer, &mut packings);

        // Between partitions order is arbitrary, so for distinct partitions
        // the canonical lower triple might be in B instead. For the same partition, the
        // first merge already found all canonical pairs.
        if key_a != key_b {
            merge_ordered_triples(triples_b, triples_a, answer, &mut packings);
        }
    }

    packings
}

/// Merge one directed pair of triple partitions in canonical signature order.
///
/// Triple enumeration is lexicographic, and placing triples into partitions
/// preserves their relative order. In particular, `upper_triples` is sorted by
/// each triple's smallest signature. For a given `lower`, `partition_point`
/// skips directly to triples satisfying
/// `lower.signatures[2] < upper.signatures[0]`. Since each triple is internally
/// sorted, this is equivalent to requiring every lower signature to sort before
/// every upper signature.
///
/// This ordering rule is what removes duplicate decompositions. Two disjoint
/// triples may be interleaved and are deliberately ignored here; if they belong
/// to a valid six-signature packing, that packing's independently enumerated
/// lower-three and upper-three triples will be accepted instead.
///
/// When both slices are the same partition, an accepted pair is still emitted
/// only once. It is found when the lower of the two triples is visited; when
/// the upper triple is later visited as `lower`, the other triple lies before
/// the `partition_point` and is not reconsidered.
fn merge_ordered_triples(
    lower_triples: &[Triple],
    upper_triples: &[Triple],
    answer: Signature,
    packings: &mut Vec<Packing>,
) {
    for &lower in lower_triples {
        // Triples are generated lexicographically, and partitioning preserves
        // that order. Find the first triple whose smallest signature sorts
        // after the largest signature in `lower`.
        let upper_start =
            upper_triples.partition_point(|upper| upper.signatures[0] <= lower.signatures[2]);

        for &upper in &upper_triples[upper_start..] {
            if lower.disjoint(upper) {
                packings.push(Packing::from_triples(answer, lower, upper));
            }
        }
    }
}

/// A disjoint packing of one answer and six guess signatures.
///
/// Guesses are stored in a sorted array. This ensures that comparisons between
/// packings are based on membership, and not order. This is important for
/// deduplication, as two packings with the same answer and guesses are equal,
/// regardless of permutation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Packing {
    answer: Signature,
    guesses: [Signature; 6],
}

impl Packing {
    /// Construct a new `Packing`.
    ///
    /// **Correctness**: Guesses must be sorted, otherwise equality comparisons
    /// will not deduplicate properly.
    pub const fn new(answer: Signature, guesses: [Signature; 6]) -> Self {
        Self { answer, guesses }
    }

    /// Construct a new `Packing` from and answer and two triples.
    fn from_triples(answer: Signature, a: Triple, b: Triple) -> Self {
        let mut guesses = [
            a.signatures[0],
            a.signatures[1],
            a.signatures[2],
            b.signatures[0],
            b.signatures[1],
            b.signatures[2],
        ];
        guesses.sort();
        Self::new(answer, guesses)
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
}

/// A triple of disjoint signatures.
///
/// The `Triple` has a mask that represents the union of its signatures. This
/// allows for fast disjointness checks.
#[derive(Debug, Clone, Copy)]
struct Triple {
    signatures: [Signature; 3],
    union: Signature,
}

impl Triple {
    /// Construct a new `Triple` with the given signatures.
    const fn new(ls1: Signature, ls2: Signature, ls3: Signature) -> Self {
        let signatures = [ls1, ls2, ls3];
        let union = ls1.union(ls2).union(ls3);
        Self { signatures, union }
    }

    /// Check if two triples are disjoint.
    const fn disjoint(self, other: Self) -> bool {
        self.union.disjoint(other.union)
    }
}
