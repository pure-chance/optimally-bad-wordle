use serde::{Deserialize, Serialize};
use wrong_wordle::{ANSWERS, Filterer, GUESSES, Realizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filterer = Filterer::new(ANSWERS, GUESSES);
    let packings = filterer.find_packings();

    let realizer = Realizer::new(ANSWERS, GUESSES);
    let results = realizer.realize(&packings);

    println!(
        "There are {} (optimally bad) wordle solutions.",
        results.len()
    );
    write(&results, "results.json")?;

    Ok(())
}

fn write<T: Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, data)?;
    Ok(())
}

fn read<T: for<'de> Deserialize<'de>>(filename: &str) -> Result<T, Box<dyn std::error::Error>> {
    let serialized = std::fs::read_to_string(filename)?;
    let data = serde_json::from_str(&serialized)?;
    Ok(data)
}

fn letter_frequencies(words: &[&str]) {
    use std::collections::HashMap;

    let mut letter_counts = HashMap::new();

    for word in words {
        for ch in word
            .chars()
            .filter(|c| c.is_ascii_lowercase())
            .collect::<std::collections::HashSet<_>>()
        {
            *letter_counts.entry(ch).or_insert(0) += 1;
        }
    }

    let mut letter_freq: Vec<_> = letter_counts.into_iter().collect();
    letter_freq.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    println!("Letter frequencies:");
    for (letter, count) in &letter_freq {
        println!(
            "{}: {} ({:.1}%)",
            letter,
            count,
            *count as f64 / words.len() as f64 * 100.0
        );
    }
}
