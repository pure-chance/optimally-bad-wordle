use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use wrong_wordle::packer;
use wrong_wordle::signature::Signature;
use wrong_wordle::words::GUESSES;

fn packer(c: &mut Criterion) {
    let mut group = c.benchmark_group("packer");

    // "civil" = many packings, "pinch" = some packings, "zesty" = few packings
    const BENCHMARK_ANSWERS: [&str; 3] = ["civil", "pinch", "zesty"];

    let guess_signatures = packer::signify_words(GUESSES);
    let partition_key = packer::create_partition_key(&guess_signatures);

    for answer in BENCHMARK_ANSWERS {
        group.bench_with_input(BenchmarkId::new("pack", answer), answer, |b, answer| {
            b.iter(|| {
                let guess_signatures = packer::signify_words(GUESSES);
                let _packings = packer::pack_for_answer(
                    Signature::new(answer),
                    &guess_signatures,
                    partition_key,
                );
            });
        });
    }

    group.finish();
}

criterion_group!(benches, packer);
criterion_main!(benches);
