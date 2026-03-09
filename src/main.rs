use wrong_wordle::packer;
use wrong_wordle::realizer;
use wrong_wordle::words::{ANSWERS, GUESSES};

/// Find all optimally bad Wordle solutions and save results.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packings = packer::pack(ANSWERS, GUESSES);
    let solutions = realizer::realize(ANSWERS, GUESSES, &packings);

    println!("There are {} packings", packings.len());
    println!(
        "There are {} (optimally bad) wordle solutions.",
        solutions.len()
    );

    std::fs::create_dir_all("results")?;
    write(&packings, "results/packings.json")?;
    write(&solutions, "results/solutions.json")?;

    Ok(())
}

/// Write data to a JSON file with pretty formatting.
fn write<T>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
{
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}
