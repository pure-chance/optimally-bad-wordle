use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use wrong_wordle::letterset::LetterSet;
use wrong_wordle::packer::Packer;
use wrong_wordle::words::{ANSWERS, GUESSES};

fn packer(c: &mut Criterion) {
    let mut group = c.benchmark_group("packer");

    // "civic" = many packings, "pinch" = some packings, "zesty" = few packings
    const BENCHMARK_ANSWERS: [&str; 3] = ["civic", "pinch", "zesty"];

    for answer in BENCHMARK_ANSWERS {
        group.bench_with_input(BenchmarkId::new("pack", answer), answer, |b, answer| {
            b.iter(|| {
                let _ =
                    Packer::new(ANSWERS, GUESSES).find_packings_for_answer(LetterSet::new(answer));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, packer);
criterion_main!(benches);
