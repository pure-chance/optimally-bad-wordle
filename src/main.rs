use anyhow::Context;
use wrong_wordle::packer;
use wrong_wordle::realizer;
use wrong_wordle::words::{ANSWERS, GUESSES};

use anyhow::Result;

/// Find all optimally wrong Wordle solutions and save results.
fn main() -> Result<()> {
    let packings = packer::pack(ANSWERS, GUESSES);
    let solutions = realizer::realize(ANSWERS, GUESSES, &packings);

    println!("There are {} packings", packings.len());
    println!(
        "There are {} (optimally wrong) wordle solutions.",
        solutions.len()
    );

    std::fs::create_dir_all("results").context("Failed to create results directory")?;
    write(&packings, "results/packings.json")
        .context("Failed to write packings to packings.json")?;
    write(&solutions, "results/solutions.json")
        .context("Failed to write solutions to solutions.json")?;

    Ok(())
}

/// Write data to a JSON file with pretty formatting.
fn write<T>(data: &T, filename: &str) -> Result<()>
where
    T: serde::Serialize,
{
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}
