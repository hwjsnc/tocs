#![forbid(unsafe_code)]

use clap::Parser;

use anyhow::Context as _;

#[derive(Debug, Parser)]
struct Args {
    /// Either a directory or file.
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    paths: Vec<std::path::PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let Args { paths } = Args::parse();
    for path in paths.iter() {
        if path.is_file() {
            try_fix(path)?;
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry.context("couldn't read directory entry")?;
                let path = entry.path();
                if entry.file_type().is_file() {
                    try_fix(path)?;
                }
            }
        } else {
            return Err(anyhow::Error::msg(format!("{} - not a file or directory", path.display())));
        }
    }
    Ok(())
}

fn try_fix(path: &std::path::Path) -> anyhow::Result<()> {
    use tocs::tbl::Version;

    let mut data = std::fs::read(path).with_context(|| format!("couldn't read file '{}'", path.display()))?;
    // TODO: parse with fixes, then reserialize (if parse was successful)
    // this is also unnecessarily inefficient since the file is getting hashed over and over for every game version
    for version in [Version::CS1En, Version::CS2En, Version::CS3, Version::CS4] {
        if tocs::tbl::fixes::fix_in_place(&version, &mut data).was_fixed() {
            std::fs::write(path, &data).with_context(|| format!("couldn't write fixed data: {}", path.display()))?;
            println!("{} - fixed!", path.display());
            return Ok(());
        }
    }
    println!("{} - no fix applied", path.display());
    Ok(())
}
