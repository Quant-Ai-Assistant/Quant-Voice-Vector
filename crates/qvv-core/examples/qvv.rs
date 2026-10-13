//! Developer CLI around the core.
//!
//!   cargo run --release -p qvv-core --example qvv -- enroll <models> <name> <out.qvoice.json> <audio>...
//!   cargo run --release -p qvv-core --example qvv -- verify <models> <profile.qvoice.json> <audio>...

use std::path::Path;

use qvv_core::{Engine, ProfileBuilder, Sample, Source, VoiceprintFile, audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage =
        "usage: qvv enroll <models> <name> <out> <audio>... | verify <models> <profile> <audio>...";
    match args.first().map(String::as_str) {
        Some("enroll") if args.len() >= 5 => {
            let engine = Engine::load(Path::new(&args[1]))?;
            let mut builder = ProfileBuilder::default();
            for file in &args[4..] {
                let chunks = engine.chunks(&audio::load_file(Path::new(file))?)?;
                println!("{file}: {} segments", chunks.len());
                for chunk in chunks {
                    builder.add(Sample {
                        vector: engine.embed(&chunk.samples)?,
                        speech_seconds: chunk.seconds(),
                        source: Source::File,
                    });
                }
            }
            println!(
                "speech {:.1} s, rejected segments: {}",
                builder.speech_seconds(),
                builder.rejected()
            );
            let profile = builder.build(&args[2])?;
            std::fs::write(&args[3], profile.to_json())?;
            println!(
                "saved {} with {} vectors",
                args[3],
                profile.embeddings.len()
            );
        }
        Some("verify") if args.len() >= 4 => {
            let engine = Engine::load(Path::new(&args[1]))?;
            let profile = VoiceprintFile::from_json(&std::fs::read_to_string(&args[2])?)?;
            for file in &args[3..] {
                let speech = engine.speech_only(&audio::load_file(Path::new(file))?)?;
                let result = profile.compare(&engine.embed(&speech)?);
                println!(
                    "{file}: centroid {:.3}, best segment {:.3}",
                    result.centroid, result.best_segment
                );
            }
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}
