use wrong_wordle::filterer::Filterer;
use wrong_wordle::realizer::Realizer;
use wrong_wordle::words::{ANSWERS, GUESSES};

/// Find all optimally bad Wordle solutions.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filterer = Filterer::new(ANSWERS, GUESSES);
    let packings = filterer.find_packings();

    let realizer = Realizer::new(ANSWERS, GUESSES);
    let solutions = realizer.realize(&packings);

    println!(
        "There are {} (optimally bad) wordle solutions.",
        solutions.len()
    );

    std::fs::create_dir_all("results")?;
    write(&packings, "results/packings.json")?;
    write(&solutions, "results/solutions.json")?;

    Ok(())
}

/// Writes serializable data to a file.
fn write<T: serde::Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}
