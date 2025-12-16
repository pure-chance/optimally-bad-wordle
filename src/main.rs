use wrong_wordle::packer::Packer;
use wrong_wordle::realizer::Realizer;
use wrong_wordle::words::{ANSWERS, GUESSES};

/// Find all optimally bad Wordle solutions.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packings = Packer::pack(ANSWERS, GUESSES);
    let solutions = Realizer::realize(ANSWERS, GUESSES, &packings);

    println!(
        "There are {} (optimally bad) wordle solutions.",
        solutions.len()
    );

    std::fs::create_dir_all("results")?;
    write(&packings, "results/packings.json")?;
    write(&solutions, "results/solutions.json")?;

    Ok(())
}

/// Write serializable data to a JSON file with pretty formatting.
fn write<T>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}
