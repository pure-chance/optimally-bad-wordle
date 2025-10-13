use serde::Serialize;
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
