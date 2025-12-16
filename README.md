# Optimally Bad Wordle

## Problem Statement

In standard Wordle, players attempt to identify a hidden five-letter word within six guesses, receiving feedback after each attempt regarding letter correctness and positioning. This project inverts that objective: it seeks to identify all possible combinations of one answer word and six guess words where the guesses provide zero information about the answer.

Formally, the problem can be stated as follows: Find all 6-tuples S = (a, g₁, g₂, g₃, g₄, g₅, g₆), where a ∈ A and gᵢ ∈ G for i ∈ {1, 2, 3, 4, 5, 6}, such that all words w ∈ S are pairwise disjoint with respect to their constituent letters.

## Implementation

This problem represents an instance of the set-packing problem, which is NP-hard. Let 𝐴 denote the set of answers and G denote the set of guesses. With |A| = 2,331 and |G| = 10,657, computational efficiency is essential for obtaining solutions within reasonable time constraints.

### Bitset Representation

The approach exploits a computationally efficient method for testing letter disjointness between word pairs. Each word is represented as a bitset, where each bit position corresponds to one of the 26 letters of the alphabet. Two words are disjoint if and only if the bitwise AND operation on their respective bitsets yields zero.

Bitset representation provides an additional advantage: it reduces the search space by consolidating words that share identical letter sets. Many distinct words contain the same set of unique letters (e.g., "slate" and "least" both map to {s, l, a, t, e}). Operating directly on these letter sets eliminates redundancy, reducing the effective vocabulary to |l(A)| = 2,037 and |l(G)| = 6,655.

### Packing

Despite these optimizations, the problem remains computationally intractable without additional pruning strategies. The algorithm proceeds through several stages of progressive refinement:

1. Answer-based packings: For each answer letterset, the algorithm first eliminates all guess lettersets that share any letters with it, substantially narrowing the candidate pool.

2. Triple enumeration: The algorithm then identifies all triples (g₁, g₂, g₃) of guess lettersets that are mutually disjoint with both the answer and each other. This enumeration employs early termination: if g₁ and g₂ share any letters, no valid triple can be formed, and the search branch is abandoned immediately.

3. Partition-based comparison: Even after triple enumeration, comparing all pairs of triples remains computationally prohibitive. To address this, the algorithm employs a hashing strategy based on a partition letterset containing the 10 most frequently occurring letters in the dataset. Each triple is assigned a hash key indicating which partition letters it contains. Only triple pairs with disjoint keys—that is, pairs whose combined partition letters do not overlap—are compared for full disjointness. This partitioning typically reduces the comparison space to several hundred thousand candidate pairs.

The algorithm parallelizes across answer lettersets, completing execution in approximately 20 seconds on my laptop.

### Realization

The packinging phase produces solutions in terms of lettersets rather than actual words. To realize concrete word combinations, a dictionary maps each letterset to its corresponding words. For each letterset solution tuple, the Cartesian product of the associated word lists generates all valid word-level solutions. This step is highly parallelizable and executes in under a second.

## Results

The complete solution set contains 1,122,348 optimally bad Wordle combinations, computed using Wordle's official answer list and extended guess vocabulary.
