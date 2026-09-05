use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let grammar = PathBuf::from(arguments.next().ok_or("missing grammar path")?);
    let output = PathBuf::from(arguments.next().ok_or("missing output directory")?);
    if arguments.next().is_some() {
        return Err("expected exactly two arguments: GRAMMAR OUTPUT_DIRECTORY".into());
    }

    let generation = antlr_rust_codegen::Builder::new()
        .grammar(&grammar)
        .library_directory(grammar.parent().ok_or("grammar has no parent directory")?)
        .out_dir(output)
        .generate()?;

    for warning in generation.warnings() {
        eprintln!("{warning}");
    }
    Ok(())
}
