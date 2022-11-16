use anyhow::Context as _;
use clap::Parser as _;

#[derive(clap::Parser)]
enum CliArgs {
    /// Commands related to .pkg files
    #[clap(subcommand)]
    Pkg(PkgArgs),
    /// Commands related to .pka files
    #[clap(subcommand)]
    Pka(PkaArgs),
}

#[derive(clap::Parser)]
enum PkgArgs {
    /// Print general information about the .pkg file and its contents
    Info {
        /// Path to .pkg file
        path: std::path::PathBuf,
    },
    /// List files contained in the .pkg
    Ls {
        /// Path to .pkg file
        path: std::path::PathBuf,
    },
    /// Print .pkg file content statistics in CSV format (You probably don't need this)
    Stats {
        /// Path to directory containing (only) .pkg files
        dir: std::path::PathBuf,
    },
    /// Unpack contents of a .pkg
    Unpack {
        /// Overwrite existing data
        #[clap(long, short)]
        force: bool,
        /// Do not use this option! It is intended for testing and development. Unpacks data without attempting decompression.
        #[clap(long)]
        raw_i_know_what_i_am_doing: bool,
        /// Path to .pkg file to unpack
        pkg: std::path::PathBuf,
        /// Output directory (where the file contents will be written)
        out: std::path::PathBuf,
        /// Unpack these specific files (if none are given, unpack all)
        files: Vec<String>,
    },
}

#[derive(clap::Parser)]
enum PkaArgs {
    /// Print general information about the .pka file and its contents
    Info { path: std::path::PathBuf },
    /// List files contained in the .pka
    Ls { path: std::path::PathBuf },
    /// Print .pka file content statistics in CSV format (You probably don't need this)
    Stats {
        /// Path to .pka
        path: std::path::PathBuf,
    },
    /// Unpack the .pka file and save its contents as .pkg files without decompressing them.
    /// This also creates a file index for recompression
    UnpackPkgs {
        /// Overwrite existing data
        #[clap(long, short)]
        force: bool,
        /// Path to .pka file
        #[clap(long)]
        pka: std::path::PathBuf,
        /// Output directory (where the .pkg files will be written)
        #[clap(long, short)]
        out: std::path::PathBuf,
        /// Unpack these specific .pkgs (if none are given, unpack all)
        files: Vec<String>,
    },
    /// Unpack the .pka file and decompress all its contents.
    /// Also creates a file index for recompression.
    /// This requires a lot of free space! For an estimate, use the pka info command.
    Unpack {
        /// Overwrite existing data
        #[clap(long, short)]
        force: bool,
        /// Do not use this option! It is intended for testing and development. Unpacks data without attempting decompression.
        #[clap(long)]
        raw_i_know_what_i_am_doing: bool,
        /// Path to .pka file
        #[clap(long)]
        pka: std::path::PathBuf,
        /// Output directory (where the files will be written)
        #[clap(long, short)]
        out: std::path::PathBuf,
        /// Unpack these specific files (if none are given, unpack all)
        files: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    match CliArgs::parse() {
        CliArgs::Pkg(PkgArgs::Info { path }) => pkg_info(path),
        CliArgs::Pkg(PkgArgs::Ls { path }) => pkg_ls(path),
        CliArgs::Pkg(PkgArgs::Stats { dir }) => pkg_stats(dir),
        CliArgs::Pkg(PkgArgs::Unpack {
            force,
            raw_i_know_what_i_am_doing,
            pkg,
            out,
            files,
        }) => {
            todo!(
                "pkg unpack force={} raw={} pkg={} out={}, files={:?}",
                force,
                raw_i_know_what_i_am_doing,
                pkg.display(),
                out.display(),
                files
            );
        }
        CliArgs::Pka(PkaArgs::Info { path }) => pka_info(path),
        CliArgs::Pka(PkaArgs::Ls { path }) => pka_ls(path),
        CliArgs::Pka(PkaArgs::Stats { path }) => pka_stats(path),
        CliArgs::Pka(PkaArgs::UnpackPkgs { force, pka, out, files }) => {
            todo!("pka unpackpkgs force={} pka={} out={} files={:?}", force, pka.display(), out.display(), files)
        }
        CliArgs::Pka(PkaArgs::Unpack {
            force,
            raw_i_know_what_i_am_doing,
            pka,
            out,
            files,
        }) => {
            todo!(
                "pka unpack force={} raw={} pka={} out={} files={:?}",
                force,
                raw_i_know_what_i_am_doing,
                pka.display(),
                out.display(),
                files
            )
        }
    }
}

fn pkg_info<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    let data = std::fs::read(&path).with_context(|| format!("couldn't read pkg file '{}'", path.as_ref().display()))?;
    let filelist = tocs::pkg::load_filelist(&mut std::io::Cursor::new(data)).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    println!("number of files: {}", filelist.len());

    let mut counts = std::collections::HashMap::new();
    for e in filelist.iter() {
        counts.entry(e.compression).and_modify(|c| *c += 1).or_insert(1);
    }
    let mut counts: std::vec::Vec<_> = counts.into_iter().collect();
    counts.sort_by_key(|(_, c)| -c);
    println!("of these:");
    for (compression, count) in counts.iter() {
        println!(" - {} files are compressed with {}", count, compression);
    }

    let compressed_size: u64 = filelist.iter().map(|e| e.compressed_size as u64).sum();
    println!("total size of compressed data: {} bytes ({:.3} MB)", compressed_size, compressed_size as f64 / 1_000_000.0);
    let decompressed_size: u64 = filelist.iter().map(|e| e.decompressed_size as u64).sum();
    println!("estimated size after decompression: {} bytes ({:.3} MB)", decompressed_size, decompressed_size as f64 / 1_000_000.0);
    println!("compression factor: {:.3}", compressed_size as f64 / std::cmp::max(1, decompressed_size) as f64);

    Ok(())
}

fn pkg_ls<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    let data = std::fs::read(&path).with_context(|| format!("couldn't read pkg file '{}'", path.as_ref().display()))?;
    let filelist = tocs::pkg::load_filelist(&mut std::io::Cursor::new(data)).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    for e in filelist.iter() {
        println!(
            "{:64} (compressed size {} bytes, decompressed size {} bytes (compression factor {:.3}), compression={}",
            e.filename,
            e.compressed_size,
            e.decompressed_size,
            e.compressed_size as f64 / std::cmp::max(1, e.decompressed_size) as f64,
            e.compression
        );
    }
    Ok(())
}

fn pkg_stats<P: AsRef<std::path::Path>>(dir: P) -> anyhow::Result<()> {
    let mut csv = csv::Writer::from_writer(std::io::stdout());
    csv.write_record(&["pkg", "file", "decompressed size", "compressed size", "data offset", "compression"])
        .with_context(|| String::from("couldn't write csv header to stdout"))?;

    for direntry in walkdir::WalkDir::new(dir) {
        let direntry = direntry.unwrap();
        let path = direntry.path();
        if direntry.file_type().is_file() {
            let data = std::fs::read(&path).with_context(|| format!("couldn't read pkg file '{}'", path.display()))?;
            let filelist = tocs::pkg::load_filelist(&mut std::io::Cursor::new(data)).with_context(|| format!("couldn't parse pkg file '{}'", path.display()))?;
            for e in filelist.iter() {
                csv.write_record(&[
                    path.display().to_string(),
                    e.filename.to_string(),
                    e.decompressed_size.to_string(),
                    e.compressed_size.to_string(),
                    e.data_offset.to_string(),
                    e.compression.to_string(),
                ])
                .with_context(|| String::from("couldn't write record"))?;
            }
        }
    }
    Ok(())
}

fn pka_info<P: AsRef<std::path::Path>>(_path: P) -> anyhow::Result<()> {
    todo!("");
}

fn pka_ls<P: AsRef<std::path::Path>>(_path: P) -> anyhow::Result<()> {
    todo!("");
}

fn pka_stats<P: AsRef<std::path::Path>>(_path: P) -> anyhow::Result<()> {
    todo!("");
}
