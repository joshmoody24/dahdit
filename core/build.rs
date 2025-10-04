use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("trigrams.rs");

    // Read the CSV file
    let csv_content = fs::read_to_string("english_3grams.csv")?;

    // Parse CSV and convert to Rust code
    let mut trigram_data = Vec::new();
    let mut line_count = 0;
    const MAX_TRIGRAMS: usize = 2000; // Only use top 2000 for performance

    for line in csv_content.lines().skip(1) {
        // Skip header
        if line_count >= MAX_TRIGRAMS {
            break;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            let trigram = parts[0];
            let freq: u64 = parts[1].parse().unwrap_or(0);

            // Only include trigrams that are exactly 3 characters
            if trigram.len() == 3 && freq > 0 {
                // Convert frequency to cost (higher frequency = lower cost)
                // Use log scale to compress the range
                let cost = if freq > 0 {
                    // Scale: most common trigram "the" (77M) gets cost ~0.5
                    // Less common trigrams get higher costs up to ~4.0
                    let normalized = (77534223.0 / freq as f64).ln();
                    (normalized * 0.5).min(4.0) as f32
                } else {
                    8.0
                };

                trigram_data.push((trigram.to_uppercase(), cost));
                line_count += 1;
            }
        }
    }

    // Generate Rust code - just the data array for inclusion
    let mut output = String::new();
    output.push_str("// Auto-generated trigram data from english_3grams.csv\n");
    output.push_str("// DO NOT EDIT - regenerated at build time\n");
    output.push_str("&[\n");

    for (trigram, cost) in trigram_data {
        output.push_str(&format!("    (\"{}\", {:.3}),\n", trigram, cost));
    }

    output.push_str("]\n");

    // Write the generated code
    fs::write(&dest_path, output)?;

    println!("cargo:rerun-if-changed=english_3grams.csv");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
