use anyhow::Context;
use wrong_wordle::packer;
use wrong_wordle::realizer;
use wrong_wordle::verifier;
use wrong_wordle::words::{ANSWERS, GUESSES};

use anyhow::Result;

/// Find all optimally wrong Wordle solutions, verify them, and save them to
/// disk.
fn main() -> Result<()> {
    let packings = packer::pack(ANSWERS, GUESSES);
    let solutions = realizer::realize(ANSWERS, GUESSES, &packings);

    verifier::verify_solutions(&solutions).context("Verification failed")?;

    println!("There are {} packings.", packings.len());
    println!(
        "There are {} (optimally wrong) Wordle solutions.",
        solutions.len()
    );

    std::fs::create_dir_all("results").context("Failed to create results directory")?;
    save_as_json(&packings, "results/packings.json")
        .context("Failed to write packings to packings.json")?;
    save_as_json(&solutions, "results/solutions.json")
        .context("Failed to write solutions to solutions.json")?;

    Ok(())
}

/// Write data to a JSON file with pretty formatting.
fn save_as_json<T: serde::Serialize>(data: &T, filename: &str) -> Result<()> {
    let file = std::fs::File::create(filename)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}
