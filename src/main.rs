use wrong_wordle::filterer::Filterer;
use wrong_wordle::realizer::Realizer;
use wrong_wordle::words::{ANSWERS, GUESSES};

/// Find all optimally bad Wordle solutions.
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

/// Writes the serialized `BadWordleSolution`s to a file.
fn write<T: serde::Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, data)?;
    Ok(())
}
