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
    use tocs::tbl::fixes::{fix_in_place, GameVersion};

    let mut data = std::fs::read(path).with_context(|| format!("couldn't read file '{}'", path.display()))?;
    // TODO: parse with fixes
    if fix_in_place(GameVersion::Unknown, &mut data).was_fixed() {
        std::fs::write(path, &data).with_context(|| format!("couldn't write fixed data: {}", path.display()))?;
        println!("{} - fixed!", path.display());
    } else {
        println!("{} - no fix applied", path.display());
    }
    Ok(())
}
