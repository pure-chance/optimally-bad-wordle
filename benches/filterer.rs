use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use wrong_wordle::filterer::Filterer;
use wrong_wordle::letterset::LetterSet;
use wrong_wordle::words::{ANSWERS, GUESSES};

fn filterer(c: &mut Criterion) {
    let mut group = c.benchmark_group("filterer");

    const BENCHMARKS_ANSWERS: [&str; 3] = ["civic", "pinch", "zesty"];

    for answer in BENCHMARKS_ANSWERS {
        group.bench_with_input(BenchmarkId::new("filterer", answer), answer, |b, answer| {
            b.iter(|| {
                let _ = Filterer::new(ANSWERS, GUESSES)
                    .find_packings_for_answer(LetterSet::new(answer));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, filterer);
criterion_main!(benches);
