extern crate clap;
extern crate tocs;
extern crate walkdir;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    /// Either a directory or file.
    #[clap(value_hint = clap::ValueHint::AnyPath)]
    paths: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), String> {
    let Args { paths } = Args::parse();
    for path in paths.iter() {
        if path.is_file() {
            try_fix(path)?;
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry.map_err(|e| format!("Couldn't get dir entry: {}", e))?;
                let path = entry.path();
                if entry.file_type().is_file() {
                    try_fix(path)?;
                }
            }
        } else {
            return Err(format!("{} - not a file or directory", path.display()));
        }
    }
    Ok(())
}

fn try_fix(path: &std::path::Path) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("Couldn't read file '{}': {}", path.display(), e))?;
    let fixed = tocs::cs1::tbl::fix(&data);
    if fixed != data {
        std::fs::write(path, &fixed).map_err(|e| format!("{} - couldn't write fixed data: {}", path.display(), e))?;
        println!("{} - fixed!", path.display());
        return Ok(());
    }
    let fixed = tocs::cs3::tbl::fix(&data);
    if fixed != data {
        std::fs::write(path, &fixed).map_err(|e| format!("{} - couldn't write fixed data: {}", path.display(), e))?;
        println!("{} - fixed!", path.display());
        return Ok(());
    }
    println!("{} - no fix applied", path.display());
    Ok(())
}
