use fallible_iterator::FallibleIterator;

use anyhow::Context as _;
use clap::Parser as _;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;
use std::borrow::Borrow as _;
use std::io::BufRead as _;
use std::io::Seek as _;
use std::io::Write as _;
use std::ops::DerefMut as _;

#[derive(clap::Parser)]
#[command(version, about)]
enum CliArgs {
    /// Commands related to .pkg files
    #[command(subcommand)]
    Pkg(PkgArgs),
    /// Commands related to .pka files
    #[command(subcommand)]
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
    /// Check .pkg consistency (for testing)
    Check {
        /// Path to .pkg
        path: std::path::PathBuf,
    },
    /// Unpack contents of a .pkg
    Unpack {
        /// Overwrite existing data
        #[arg(long, short)]
        force: bool,
        /// Do not use this option! It is intended for testing and development. Unpacks data without attempting decompression.
        #[arg(long)]
        raw_i_know_what_i_am_doing: bool,
        /// Path to .pkg file to unpack
        #[arg(long)]
        pkg: std::path::PathBuf,
        /// Output directory (where the file contents will be written)
        #[arg(long, short)]
        out: std::path::PathBuf,
        /// Unpack these specific files (if none are given, unpack all)
        files: Vec<String>,
    },
    /// Create .pkg from the contents of a given folder
    Pack {
        /// Path to the folder to pack
        dir: std::path::PathBuf,
        /// Compression type
        #[arg(long, short)]
        compression: CompressionArg,
        /// Output .pkg file (if it is not given, append .pkg to the folder name)
        #[arg(long, short)]
        out: Option<std::path::PathBuf>,
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
        /// Overwrite existing stat files
        #[arg(long, short)]
        force: bool,
        /// Path to .pka
        path: std::path::PathBuf,
    },
    /// Check .pka consistency (for testing)
    Check {
        /// Path to .pka
        path: std::path::PathBuf,
    },
    /// Unpack the .pka file and save its contents as .pkg files without decompressing them.
    /// This also creates a file index (blob_order.txt) for recompression
    UnpackPkgs {
        /// Overwrite existing data
        #[arg(long, short)]
        force: bool,
        /// Path to .pka file
        #[arg(long)]
        pka: std::path::PathBuf,
        /// Output directory (where the .pkg files will be written)
        #[arg(long, short)]
        out: std::path::PathBuf,
        /// Unpack these specific .pkgs (if none are given, unpack all)
        files: Vec<String>,
    },
    /// Unpack the .pka file and decompress all its contents.
    /// Also creates a file index for recompression.
    /// This requires a lot of free space! For an estimate, use the pka info command.
    Unpack {
        /// Overwrite existing data
        #[arg(long, short)]
        force: bool,
        /// Do not use this option! It is intended for testing and development. Unpacks data without attempting decompression.
        #[arg(long)]
        raw_i_know_what_i_am_doing: bool,
        /// Path to .pka file
        #[arg(long)]
        pka: std::path::PathBuf,
        /// Output directory (where the files will be written)
        #[arg(long, short)]
        out: std::path::PathBuf,
        /// Unpack these specific files (if none are given, unpack all)
        files: Vec<String>,
    },
    /// Packs a folder of .pkg files to a .pka, without recompressing.
    PackFromPkgs {
        /// Overwrite existing data
        #[arg(long, short)]
        force: bool,
        /// Path of the folder containing .pkgs.
        /// Needs to contain a blob_order.txt file.
        dir: std::path::PathBuf,
        /// Output .pka file
        #[arg(long, short)]
        out: std::path::PathBuf,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompressionArg {
    Uncompressed,
    Lzss,
    LZ4,
    Zstd,
}

impl From<CompressionArg> for tocs::compression::Compression {
    fn from(x: CompressionArg) -> Self {
        match x {
            CompressionArg::Uncompressed => tocs::compression::Compression::None,
            CompressionArg::Lzss => tocs::compression::Compression::LZSS,
            CompressionArg::LZ4 => tocs::compression::Compression::LZ4,
            CompressionArg::Zstd => tocs::compression::Compression::Zstd,
        }
    }
}

impl clap::ValueEnum for CompressionArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[CompressionArg::Uncompressed, CompressionArg::Lzss, CompressionArg::LZ4, CompressionArg::Zstd]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            CompressionArg::Uncompressed => clap::builder::PossibleValue::new("uncompressed").help("no compression"),
            CompressionArg::Lzss => clap::builder::PossibleValue::new("lzss").help("custom LZSS compression (used by CS1/2)"),
            CompressionArg::LZ4 => clap::builder::PossibleValue::new("lz4").help("LZ4 compression (used by CS3)"),
            CompressionArg::Zstd => clap::builder::PossibleValue::new("zstd").help("Zstandard compression (used by CS4)"),
        })
    }
}

impl std::str::FromStr for CompressionArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uncompressed" => Ok(CompressionArg::Uncompressed),
            "lzss" => Ok(CompressionArg::Lzss),
            "lz4" => Ok(CompressionArg::LZ4),
            "zstd" => Ok(CompressionArg::Zstd),
            _ => Err(format!("Invalid compression: {}", s)),
        }
    }
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    match CliArgs::parse() {
        CliArgs::Pkg(PkgArgs::Info { path }) => pkg_info(path),
        CliArgs::Pkg(PkgArgs::Ls { path }) => pkg_ls(path),
        CliArgs::Pkg(PkgArgs::Stats { dir }) => pkg_stats(dir),
        CliArgs::Pkg(PkgArgs::Check { path }) => pkg_check(path),
        CliArgs::Pkg(PkgArgs::Unpack {
            force,
            raw_i_know_what_i_am_doing,
            pkg,
            out,
            files,
        }) => pkg_unpack(force, raw_i_know_what_i_am_doing, pkg, out, &files),
        CliArgs::Pkg(PkgArgs::Pack { dir, out, compression }) => pkg_pack(&dir, &out.unwrap_or_else(|| dir.with_extension(".pkg")), compression.into()),
        CliArgs::Pka(PkaArgs::Info { path }) => pka_info(path),
        CliArgs::Pka(PkaArgs::Ls { path }) => pka_ls(path),
        CliArgs::Pka(PkaArgs::Stats { path, force }) => pka_stats(path, force),
        CliArgs::Pka(PkaArgs::Check { path }) => pka_check(path),
        CliArgs::Pka(PkaArgs::UnpackPkgs { force, pka, out, files }) => pka_unpack_pkgs(force, pka, out, &files),
        CliArgs::Pka(PkaArgs::Unpack {
            force,
            raw_i_know_what_i_am_doing,
            pka,
            out,
            files,
        }) => pka_unpack(force, raw_i_know_what_i_am_doing, pka, out, &files),
        CliArgs::Pka(PkaArgs::PackFromPkgs { force, dir, out }) => pka_pack_pkgs(force, dir, out),
    }
}

fn name_to_path(name: &str) -> anyhow::Result<&std::path::Path> {
    if !name.is_ascii() || name.chars().any(|c| c.is_ascii_control() || (c.is_ascii_whitespace() && c != ' ')) || name.contains('/') || name.contains('\\') {
        Err(anyhow::Error::msg(format!("invalid file name '{}'", name)))
    } else {
        Ok(std::path::Path::new(name))
    }
}

fn name_is_ok(name: &str) -> bool {
    name_to_path(name).is_ok()
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

fn pkg_check<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    if path.as_ref().is_dir() {
        let files: std::vec::Vec<_> = walkdir::WalkDir::new(path.as_ref())
            .into_iter()
            .map(|entry| entry.unwrap().path().to_owned())
            .filter(|path| path.is_file())
            .collect();
        files.par_iter().map(|path| pkg_check_single(path.as_ref())).find_any(|result| result.is_err()).unwrap_or(Ok(()))?;
    } else {
        pkg_check_single(path.as_ref())?;
    }
    log::info!("ok");
    Ok(())
}

fn pkg_check_single(pkg: &std::path::Path) -> anyhow::Result<()> {
    log::debug!("Checking file {}", pkg.display());

    let data = std::fs::read(&pkg).with_context(|| format!("couldn't read pkg file '{}'", pkg.display()))?;
    let mut cursor = std::io::Cursor::new(&data);
    let filelist = tocs::pkg::load_filelist(&mut cursor).with_context(|| format!("couldn't parse pkg file '{}'", pkg.display()))?;

    let offset_after_metadata = cursor.stream_position().expect("cursor stream_position should not fail");

    /* check that data does not overlap and leaves no space empty */
    let mut filelist = filelist;
    filelist.sort_by_key(|e| e.data_offset);
    let filelist = filelist;
    let mut expected_start: u32 = offset_after_metadata.try_into().expect("data should be < 4GiB");
    for e in filelist.iter() {
        log::debug!("checking offset of entry {}", e.filename);
        if e.data_offset != expected_start {
            return Err(anyhow::Error::msg(format!("{}: unexpected start ({} rather than {})", e.filename, e.data_offset, expected_start)).context(format!("{}: consistency error", pkg.display())));
        }
        expected_start += e.compressed_size;
    }
    if <u32 as TryInto<usize>>::try_into(expected_start).expect("32 bits please") != data.len() {
        return Err(anyhow::Error::msg(format!("trailing data starting at offset {} (0x{:x})", expected_start, expected_start)).context(format!("{}: consistency error", pkg.display())));
    }

    /* check read->decompress->recompress->decompress loop */
    for e in filelist.iter() {
        log::debug!("Testing {} / {}", pkg.display(), e.filename);
        if !name_is_ok(&e.filename) {
            return Err(anyhow::Error::msg(format!("{}: bad file name '{}'", pkg.display(), e.filename)));
        }
        let compressed_data = tocs::pkg::get_data_raw_from_buffer(&data, e)?;
        let decompressed_data = tocs::pkg::get_data_from_buffer(&data, e)?;
        let recompressed_data = e.compression.compress(&decompressed_data).map_err(anyhow::Error::msg)?;
        let redecompressed_data = e
            .compression
            .decompress_with_exact_output_size(&recompressed_data, e.decompressed_size.try_into().expect("32 bits please"))
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("{}: couldn't decompress and recompress data", pkg.display()))?;
        assert_eq!(decompressed_data, redecompressed_data);
        if recompressed_data.len() != compressed_data.len() {
            // exact equality is unrealistic
            log::warn!(
                "{}: recompressing data changes size ({} instead of {} bytes)",
                e.filename,
                recompressed_data.len(),
                compressed_data.len()
            );
        }
    }

    log::debug!("{}: ok.", pkg.display());

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

fn pkg_unpack<P: AsRef<std::path::Path>, S: AsRef<str>>(force: bool, raw: bool, pkg: P, out: P, files: &[S]) -> anyhow::Result<()> {
    let data = std::fs::read(&pkg).with_context(|| format!("couldn't read pkg file '{}'", pkg.as_ref().display()))?;
    let mut cursor = std::io::Cursor::new(&data);
    let filelist = tocs::pkg::load_filelist(&mut cursor).with_context(|| format!("couldn't parse pkg file '{}'", pkg.as_ref().display()))?;
    std::fs::create_dir_all(&out).with_context(|| format!("creating output directory {}", out.as_ref().display()))?;

    let file_by_name: std::collections::HashMap<_, _> = filelist.iter().map(|entry| (entry.filename.clone(), entry)).collect();

    let extract = |entry: &tocs::pkg::FileListEntry| {
        if !name_is_ok(&entry.filename) {
            return Err(anyhow::Error::msg(format!("invalid file name: {}", entry.filename)));
        }
        log::info!("extracting file '{}'", entry.filename);
        let out_file = std::path::Path::join(out.as_ref(), name_to_path(&entry.filename)?);
        let data = if raw {
            tocs::pkg::get_data_raw_from_buffer(&data, entry).map(|slice| slice.to_vec())
        } else {
            tocs::pkg::get_data_from_buffer(&data, entry)
        }
        .with_context(|| format!("couldn't read data for pkg '{}', file '{}'", pkg.as_ref().display(), entry.filename))?;
        let mut file = if force {
            std::fs::OpenOptions::new().write(true).create(true).open(&out_file)
        } else {
            std::fs::OpenOptions::new().write(true).create_new(true).open(&out_file)
        }
        .with_context(|| format!("couldn't open output file '{}'", out_file.display()))?;
        file.write_all(&data).with_context(|| format!("couldn't write data to file '{}'", out_file.display()))?;
        Ok(())
    };

    if files.is_empty() {
        for entry in filelist.iter() {
            extract(entry)?;
        }
    } else {
        for file in files.iter() {
            match file_by_name.get(file.as_ref()) {
                Some(entry) => extract(entry)?,
                None => {
                    return Err(anyhow::Error::msg(format!("pkg '{}' does not contain file '{}'", pkg.as_ref().display(), file.as_ref())))?;
                }
            }
        }
    }

    Ok(())
}

fn pkg_pack<P: AsRef<std::path::Path>>(dir: P, pkg: P, compression: tocs::compression::Compression) -> anyhow::Result<()> {
    let mut files = vec![];
    for entry in walkdir::WalkDir::new(dir.as_ref()) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path == dir.as_ref() {
            continue;
        } else if path.is_file() {
            let data = std::fs::read(&path).with_context(|| format!("couldn't read file '{}'", path.display()))?;
            let name = path.file_name().expect("folder child should have a final component");
            files.push((
                name.to_os_string()
                    .into_string()
                    .map_err(|_| anyhow::Error::msg(format!("can't put filename '{}' into a .pkg", path.display())))?,
                data,
            ));
        } else {
            return Err(anyhow::Error::msg(format!("'{}' is not a file", path.display())));
        }
    }
    let pkg_data = tocs::pkg::generate_pkg(files, compression).context("couldn't create .pkg")?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(pkg.as_ref())
        .with_context(|| format!("couldn't open output file '{}'", pkg.as_ref().display()))?;
    file.write(&pkg_data).with_context(|| format!("couldn't write to output file '{}'", pkg.as_ref().display()))?;
    file.sync_data().with_context(|| format!("couldn't write to output file '{}'", pkg.as_ref().display()))?;
    Ok(())
}

fn pka_info<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    let mut f = std::io::BufReader::new(std::fs::File::open(&path).with_context(|| format!("couldn't open pka file '{}' for reading", path.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    let number_of_files: usize = pkgs.iter().map(|pkg| pkg.filenames.len()).sum();
    println!("number of files: {}", number_of_files);

    println!("number of blobs: {}", blobs.len());
    let mut counts = std::collections::HashMap::new();
    for e in blobs.iter() {
        counts.entry(e.compression).and_modify(|c| *c += 1).or_insert(1);
    }
    let mut counts: std::vec::Vec<_> = counts.into_iter().collect();
    counts.sort_by_key(|(_, c)| -c);
    println!("of these:");
    for (compression, count) in counts.iter() {
        println!(" - {} blobs are compressed with {}", count, compression);
    }

    let blob_hashes: std::collections::HashSet<[u8; 32]> = blobs.iter().map(|blob| blob.hash).collect();
    let file_hashes: std::collections::HashSet<[u8; 32]> = pkgs.iter().flat_map(|pkg| pkg.hashes.iter().copied()).collect();
    println!("number of blobs with duplicate hash: {} (should be 0)", blobs.len() - blob_hashes.len());
    println!("number of files with duplicate hash: {} (should be ≥ 0)", number_of_files - file_hashes.len());
    println!("number of files with no corresponding blob: {} (should be 0)", file_hashes.difference(&blob_hashes).count());
    println!("number of blobs with no corresponding file: {} (should be 0)", blob_hashes.difference(&file_hashes).count());

    let compressed_size: u64 = blobs.iter().map(|blob| blob.compressed_size as u64).sum();
    let blobs: std::collections::HashMap<_, _> = blobs.into_iter().map(|blob| (blob.hash, blob)).collect();
    let decompressed_size: u64 = pkgs
        .iter()
        .flat_map(|pkg| pkg.hashes.iter().filter_map(|hash| blobs.get(hash)).map(|blob| blob.decompressed_size as u64))
        .sum();

    println!("total size of compressed data: {} bytes ({:.3} GB)", compressed_size, compressed_size as f64 / 1_000_000_000.0);
    println!("estimated size after decompression: {} bytes ({:.3} GB)", decompressed_size, decompressed_size as f64 / 1_000_000_000.0);
    println!("compression factor: {:.3}", compressed_size as f64 / std::cmp::max(1, decompressed_size) as f64);

    Ok(())
}

fn pka_ls<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    let mut f = std::io::BufReader::new(std::fs::File::open(&path).with_context(|| format!("couldn't open pka file '{}' for reading", path.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    let blobs: std::collections::HashMap<_, _> = blobs.into_iter().map(|blob| (blob.hash, blob)).collect();

    for pkg in pkgs {
        println!("{}:", pkg.name);
        for (file, hash) in pkg.filenames.iter().zip(pkg.hashes.iter()) {
            if let Some(blob) = blobs.get(hash) {
                println!(
                    "{:64} (compressed size {}, decompressed size {}, (compression factor {:.3}), compression = {})",
                    file,
                    blob.compressed_size,
                    blob.decompressed_size,
                    blob.compressed_size as f64 / std::cmp::max(1, blob.decompressed_size) as f64,
                    blob.compression
                );
            } else {
                println!("  {}: no corresponding blob (hash = {})!", file, hex::encode(hash));
            }
        }
    }
    Ok(())
}

fn pka_stats<P: AsRef<std::path::Path>>(path: P, force: bool) -> anyhow::Result<()> {
    let mut f = std::io::BufReader::new(std::fs::File::open(&path).with_context(|| format!("couldn't open pka file '{}' for reading", path.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    let mut open_options = std::fs::File::options();
    open_options.write(true);
    if force {
        open_options.create(true);
    } else {
        open_options.create_new(true);
    }

    let files_csv = std::path::Path::new("files.csv");
    let blobs_csv = std::path::Path::new("blobs.csv");

    let f = std::io::BufWriter::new(open_options.open(files_csv).with_context(|| format!("couldn't open output file '{}'", files_csv.display()))?);
    let mut csv = csv::Writer::from_writer(f);
    csv.write_record(&["pkg", "file", "hash"]).with_context(|| String::from("couldn't write csv header to stdout"))?;
    for pkg in pkgs {
        for (file, hash) in pkg.filenames.iter().zip(pkg.hashes.iter()) {
            csv.write_record(&[pkg.name.to_string(), file.to_string(), hex::encode(hash)])
                .with_context(|| String::from("couldn't write record"))?;
        }
    }

    let f = std::io::BufWriter::new(open_options.open(blobs_csv).with_context(|| format!("couldn't open output file '{}'", blobs_csv.display()))?);
    let mut csv = csv::Writer::from_writer(f);
    csv.write_record(&["hash", "offset", "compressed size", "decompressed size", "compression"])
        .with_context(|| String::from("couldn't write csv header to stdout"))?;
    for blob in blobs {
        csv.write_record(&[
            hex::encode(blob.hash),
            blob.offset.to_string(),
            blob.compressed_size.to_string(),
            blob.decompressed_size.to_string(),
            blob.compression.to_string(),
        ])
        .with_context(|| String::from("couldn't write record"))?;
    }

    Ok(())
}

fn pka_check<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<()> {
    let mut f = std::io::BufReader::new(std::fs::File::open(&path).with_context(|| format!("couldn't open pka file '{}' for reading", path.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    // wheck file names
    for pkg in pkgs.iter() {
        log::debug!("checking pkg {}", pkg.name);
        assert!(name_is_ok(&pkg.name));
        for name in pkg.filenames.iter() {
            log::debug!("checking file {}", name);
            assert!(name_is_ok(name));
        }
    }

    let offset_after_metadata = f.stream_position()?;

    let blob_hashes: std::collections::HashSet<_> = blobs.iter().map(|blob| blob.hash).collect();
    let file_hashes: std::collections::HashSet<_> = pkgs.iter().flat_map(|pkg| pkg.hashes.iter().copied()).collect();
    for hash in blob_hashes.difference(&file_hashes) {
        panic!("blob hash with no associated file: {}", hex::encode(hash));
    }
    for hash in file_hashes.difference(&blob_hashes) {
        panic!("file hash with no associated data: {}", hex::encode(hash));
    }

    // check that file names are sorted
    /*
    let mut prev_pkg_name = None;
    for pkg in pkgs.iter() {
        match prev_pkg_name {
            Some(prev) if prev >= pkg.name => {
                panic!("pkg name {} is not greater than previous ({})", pkg.name, prev);
            }
            _ => {}
        }
        prev_pkg_name = Some(pkg.name.clone());

        let mut prev_file_name = None;
        for name in pkg.filenames.iter() {
            match prev_file_name {
                Some(prev) if prev >= pkg.name => {
                    panic!("file name {} / {} is not greater than previous ({} / {})", pkg.name, name, pkg.name, prev);
                }
                _ => {}
            }
            prev_file_name = Some(name.clone());
        }
    }
    */

    // check that blob metadata is sorted by hash
    let mut prev_hash = None;
    for blob in blobs.iter() {
        match prev_hash {
            Some(prev) if prev >= blob.hash => {
                panic!("blob hash {} is not greater than previous ({})", hex::encode(blob.hash), hex::encode(prev));
            }
            _ => {}
        }
        prev_hash = Some(blob.hash);
    }

    // check that data is overlap free and leaves no unused space
    let mut blobs = blobs;
    blobs.sort_by_key(|blob| blob.offset);
    let blobs = blobs;
    let mut expected_start: u64 = offset_after_metadata;
    for blob in blobs.iter() {
        log::debug!("checking offset of blob {}", hex::encode(blob.hash));
        if blob.offset != expected_start {
            panic!("{}: unexpected start ({} rather than {})", hex::encode(blob.hash), blob.offset, expected_start);
        }
        expected_start += <u32 as Into<u64>>::into(blob.compressed_size);
    }
    if expected_start != f.get_ref().metadata()?.len() {
        panic!("trailing data starting at offset {} (0x{:x})", expected_start, expected_start);
    }

    log::info!("metadata checks complete, starting checking contents");

    // check read->decompress->recompress->decompress loop and content hash
    let io_mutex = std::sync::Mutex::new(f);
    blobs.par_iter().map(|blob| pka_test_blob(&io_mutex, blob)).find_any(|result| result.is_err()).unwrap_or(Ok(()))?;

    log::info!("ok");
    Ok(())
}

fn pka_test_blob<F: std::io::Read + std::io::Seek>(io_mutex: &std::sync::Mutex<F>, blob: &tocs::pka::FileDataEntry) -> anyhow::Result<()> {
    // reading and decompressing should be fast, so locking the file should not be too much of a problem
    let mut guard = io_mutex
        .lock()
        .map_err(|e| e.to_string())
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("blob {}: failed to read data", hex::encode(blob.hash)))?;
    log::debug!("Testing {}", hex::encode(blob.hash));
    let compressed_data = tocs::pka::get_blob_data_raw(guard.deref_mut(), blob).with_context(|| format!("blob {}: failed to read data", hex::encode(blob.hash)))?;
    let decompressed_data = tocs::pka::get_blob_data(guard.deref_mut(), blob).with_context(|| format!("blob {}: failed to read data", hex::encode(blob.hash)))?;
    drop(guard);

    // recompressing is the bit that really takes time
    let recompressed_data = blob
        .compression
        .compress(&decompressed_data)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("blob {}: failed to recompress data", hex::encode(blob.hash)))?;
    let redecompressed_data = blob
        .compression
        .decompress_with_exact_output_size(&recompressed_data, blob.decompressed_size.try_into().expect("32 bits please"))
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("blob {}: failed to decompress the recompressed data", hex::encode(blob.hash)))?;
    let hash = tocs::pka::hash_data(&decompressed_data);
    if blob.hash != hash {
        return Err(anyhow::Error::msg(format!(
            "blob {}: hash does not match sha256 of decompressed data {}",
            hex::encode(blob.hash),
            hex::encode(hash)
        )));
    }
    if decompressed_data != redecompressed_data {
        return Err(anyhow::Error::msg(format!("blob {}: recompress->decompress changes data", hex::encode(blob.hash))));
    }

    let delta = (recompressed_data.len() as f64 - compressed_data.len() as f64).abs() / std::cmp::max(recompressed_data.len(), compressed_data.len()) as f64;
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(delta < 0.05) {
        // exact equality is unrealistic
        log::warn!(
            "blob {}: recompressing data changes size significantly ({} instead of {} bytes)",
            hex::encode(blob.hash),
            recompressed_data.len(),
            compressed_data.len()
        );
    }

    Ok(())
}

fn pka_unpack_pkgs<P: AsRef<std::path::Path>, S: AsRef<str>>(force: bool, pka: P, out: P, files: &[S]) -> anyhow::Result<()> {
    let mut f = std::io::BufReader::new(std::fs::File::open(pka.as_ref()).with_context(|| format!("couldn't open pka file '{}' for reading", pka.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pka file '{}'", pka.as_ref().display()))?;

    let mut open_options = std::fs::File::options();
    open_options.write(true);
    if force {
        open_options.create(true);
    } else {
        open_options.create_new(true);
    };

    std::fs::create_dir_all(&out).with_context(|| format!("couldn't create output directory {}", out.as_ref().display()))?;

    let blobs_by_hash: std::collections::HashMap<_, _> = blobs.iter().map(|blob| (blob.hash, blob.clone())).collect();

    if files.is_empty() {
        write_blob_order_file(&blobs, &open_options, out.as_ref())?;
        for pkg in pkgs.iter() {
            pka_unpack_single_pkg(&open_options, &mut f, out.as_ref(), &blobs_by_hash, pkg)?;
        }
    } else {
        // we're (probably) not unpacking everything, so we don't need to be able to repack
        let pkgs_by_name: std::collections::HashMap<_, _> = pkgs.iter().map(|pkg| (pkg.name.clone(), pkg)).collect();
        for name in files {
            let pkg = pkgs_by_name
                .get(name.as_ref())
                .ok_or_else(|| anyhow::Error::msg(format!("pkg '{}' not present in pka", name.as_ref())))?;
            pka_unpack_single_pkg(&open_options, &mut f, out.as_ref(), &blobs_by_hash, pkg)?;
        }
    }

    Ok(())
}

fn pka_unpack_single_pkg<F: std::io::Read + std::io::Seek>(
    open_options: &std::fs::OpenOptions,
    pka: &mut F,
    out_dir: &std::path::Path,
    blobs_by_hash: &std::collections::HashMap<[u8; 32], tocs::pka::FileDataEntry>,
    pkg: &tocs::pka::PkgListEntry,
) -> anyhow::Result<()> {
    log::debug!("unpack single pkg '{}'", &pkg.name);

    let out = std::path::Path::join(out_dir, name_to_path(&pkg.name).with_context(|| format!("Invalid pkg file name '{}'", &pkg.name))?);

    let mut pkg_files = vec![];
    for (name, hash) in pkg.filenames.iter().zip(pkg.hashes.iter()) {
        let blob = blobs_by_hash
            .get(hash)
            .ok_or_else(|| anyhow::Error::msg(format!("no data for file '{}/{}' (hash {})", pkg.name, name, hex::encode(hash))))?;
        let compressed_data = tocs::pka::get_blob_data_raw(pka, blob).with_context(|| format!("couldn't read data for file '{}/{}'", pkg.name, name))?;
        pkg_files.push(tocs::pkg::CompressedFile {
            name: tocs::pkg::PaddedFilename::new(name, 64)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("couldn't write pkg data for file '{}/{}'", pkg.name, name))?,
            data: compressed_data,
            decompressed_size: blob.decompressed_size,
            compression: blob.compression,
        });
    }

    let data = tocs::pkg::generate_pkg_from_compressed_files(&pkg_files).with_context(|| format!("couldn't generate pkg data for pkg '{}'", pkg.name))?;

    let mut out_file = open_options
        .open(&out)
        .with_context(|| anyhow::Error::msg(format!("couldn't open output file '{}' for writing", out.display())))?;
    out_file
        .write(&data)
        .with_context(|| anyhow::Error::msg(format!("couldn't open output file '{}' for writing", out.display())))?;

    Ok(())
}

fn pka_unpack<P: AsRef<std::path::Path>, S: AsRef<str>>(force: bool, raw: bool, pka: P, out: P, files: &[S]) -> anyhow::Result<()> {
    if raw && files.is_empty() {
        return Err(anyhow::Error::msg("refusing to unpack complete pka raw"));
    }

    let mut f = std::io::BufReader::new(std::fs::File::open(pka.as_ref()).with_context(|| format!("couldn't open pka file '{}' for reading", pka.as_ref().display()))?);
    let (pkgs, blobs) = tocs::pka::load_filelist(&mut f).with_context(|| format!("couldn't parse pka file '{}'", pka.as_ref().display()))?;

    let mut open_options = std::fs::File::options();
    open_options.write(true);
    if force {
        open_options.create(true);
    } else {
        open_options.create_new(true);
    }
    let open_options = open_options;

    std::fs::create_dir_all(&out).with_context(|| format!("couldn't create output directory {}", out.as_ref().display()))?;

    let blobs_by_hash: std::collections::HashMap<_, _> = blobs.iter().map(|blob| (blob.hash, blob)).collect();

    if files.is_empty() {
        // extract all

        // write blob order file
        write_blob_order_file(&blobs, &open_options, out.as_ref())?;

        // extract all files in all pkgs
        for pkg in pkgs.iter() {
            log::info!("unpacking contents of pkg {}", pkg.name);
            let pkgname = name_to_path(&pkg.name)?;
            let pkgpath = std::path::Path::join(out.as_ref(), pkgname);
            std::fs::create_dir_all(&pkgpath).with_context(|| format!("couldn't create output directory {} for pkg {}", out.as_ref().display(), pkg.name))?;
            for (name, hash) in pkg.filenames.iter().zip(pkg.hashes.iter()) {
                if name.is_empty() {
                    log::warn!("skipping file with empty name (blob hash {})", hex::encode(hash));
                    continue;
                }
                log::debug!("unpacking {} (blob {})", name, hex::encode(hash));
                let filename = std::path::Path::join(&pkgpath, name_to_path(name)?);
                let mut outfile = std::io::BufWriter::new(
                    open_options
                        .open(&filename)
                        .with_context(|| format!("couldn't open file {} for pkg file {}/{}", filename.display(), pkg.name, name))?,
                );
                let blobinfo = match blobs_by_hash.get(hash) {
                    Some(blobinfo) => blobinfo,
                    None => {
                        return Err(anyhow::Error::msg(format!("couldn't read data for pkg file {}/{}", pkg.name, name)));
                    }
                };
                let data = if raw {
                    unimplemented!("unpacking entire .pka raw data seems unnecessary")
                } else {
                    tocs::pka::get_blob_data(&mut f, blobinfo)
                }
                .with_context(|| format!("couldn't read data for {}/{}", pkg.name, name))?;
                outfile.write_all(&data).with_context(|| format!("couldn't write data for pkg file {}/{}", pkg.name, name))?;
            }
        }
    } else {
        // extract only specific files

        let blobs_by_name: std::collections::HashMap<_, _> = pkgs
            .iter()
            .flat_map(|pkg| {
                pkg.filenames.iter().zip(pkg.hashes.iter()).filter_map(|(filename, hash)| {
                    let name = format!("{}/{}", pkg.name, filename); // note: this is unambiguous since neither name contains "/"
                    let blobinfo = blobs_by_hash.get(hash)?;
                    Some((name, blobinfo))
                })
            })
            .collect();
        for file in files.iter() {
            log::debug!("unpacking {}", file.as_ref());
            let blobinfo = match blobs_by_name.get(file.as_ref()) {
                Some(blobinfo) => blobinfo,
                None => {
                    return Err(anyhow::Error::msg(format!("no data for file {}", file.as_ref())));
                }
            };
            let filename = std::path::Path::join(out.as_ref(), std::path::Path::new(file.as_ref()));
            std::fs::create_dir_all(filename.parent().expect("result of a join should have a parent")).with_context(|| format!("couldn't create output directory {}", out.as_ref().display()))?;
            let mut outfile = open_options.open(&filename).with_context(|| format!("couldn't open output file {}", filename.display()))?;
            let data = if raw {
                tocs::pka::get_blob_data_raw(&mut f, blobinfo)
            } else {
                tocs::pka::get_blob_data(&mut f, blobinfo)
            }
            .with_context(|| format!("couldn't read data for {}", file.as_ref()))?;
            outfile.write_all(&data).with_context(|| format!("couldn't write data to {}", filename.display()))?;
        }
    }
    Ok(())
}

fn write_blob_order_file(blobs: &[tocs::pka::FileDataEntry], open_options: &std::fs::OpenOptions, out_dir: &std::path::Path) -> anyhow::Result<()> {
    let mut blobs = blobs.to_vec();
    blobs.sort_by_key(|blob| blob.offset);
    let mut f = std::io::BufWriter::new(
        open_options
            .open(std::path::Path::join(out_dir, std::path::Path::new("blob_order.txt")))
            .context("couldn't open blob order file for writing")?,
    );
    for blob in blobs.iter() {
        writeln!(f, "{}", hex::encode(blob.hash)).context("couldn't write blob order file")?;
    }
    f.into_inner().context("couldn't close blob order file after writing")?;
    Ok(())
}

fn read_blob_order_file(base_dir: &std::path::Path) -> anyhow::Result<std::vec::Vec<[u8; 32]>> {
    let filename = std::path::Path::join(base_dir, "blob_order.txt");
    log::debug!("reading blob order file '{}'", filename.display());
    let mut hashes = vec![];
    for line in std::io::BufReader::new(std::fs::File::open(&filename)?).lines() {
        let line = line?;
        let hash: [u8; 32] = hex::decode(&line)
            .map_err(anyhow::Error::msg)
            .and_then(|data| (&data[..]).try_into().map_err(anyhow::Error::msg))
            .with_context(|| format!("invalid line '{}', expected a (hex encoded) SHA-256 hash", &line))?;
        hashes.push(hash);
    }
    Ok(hashes)
}

fn pka_pack_pkgs<P: AsRef<std::path::Path>>(force: bool, dir: P, out: P) -> anyhow::Result<()> {
    let blob_order = read_blob_order_file(dir.as_ref()).context("couldn't read blob order file")?;

    log::info!("preparing file and blob index (this may take some time)");

    // this part is IO-bound except on SSD
    // we could probably parallelize it.
    let mut blob_info: std::collections::HashMap<[u8; 32], tocs::pka::CompressedBlobMetadata> = std::collections::HashMap::new();
    let mut pkg_list: std::vec::Vec<tocs::pka::PkgListEntry> = vec![];
    let mut blobs_that_need_to_be_read: std::collections::HashMap<std::path::PathBuf, std::vec::Vec<(usize, [u8; 32])>> = std::collections::HashMap::new();
    misc::fallible_read_dir(dir.as_ref())
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("couldn't open directory '{}' for reading", dir.as_ref().display()))?
        .map(|dirent| Ok(dirent.path()))
        .filter(|path| {
            if path.is_file() {
                if path.extension() == Some(std::ffi::OsStr::new("pkg")) {
                    log::debug!("reading file '{}'", path.display());
                    Ok(true)
                } else {
                    log::debug!("ignoring non-pkg file '{}'", path.display());
                    Ok(false)
                }
            } else {
                log::debug!("ignoring '{}' because it's not a file", path.display());
                Ok(false)
            }
        })
        .map_err(anyhow::Error::msg)
        .for_each(|path| {
            let pkg_data = std::fs::read(&path)
                .map_err(tocs::pkg::PkgError::IO)
                .and_then(tocs::pkg::Pkg::new)
                .with_context(|| format!("couldn't read pkg file '{}'", path.display()))?;
            let pkg_name: &str = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow::Error::msg(format!("invalid pkg file name: {}", path.display())))?;
            let mut pkg_list_entry = tocs::pka::PkgListEntry {
                name: pkg_name.into(),
                filenames: vec![],
                hashes: vec![],
            };
            pkg_data.entries().map_err(anyhow::Error::msg).enumerate().for_each(|(i, entry)| {
                let decompressed_data = entry
                    .decompressed_data()
                    .with_context(|| format!("couldn't read data for file '{}' in pkg '{}'", entry.name().borrow(), path.display()))?;
                let hash = tocs::pka::hash_data(&decompressed_data);
                let blobinfo = tocs::pka::CompressedBlobMetadata {
                    hash,
                    decompressed_size: entry.decompressed_size(),
                    compressed_size: entry.compressed_size(),
                    compression: entry.compression(),
                };
                if let Some(old) = blob_info.insert(hash, blobinfo) {
                    if old != blobinfo {
                        log::warn!("same blob ({}) appears with different sizes/compression", hex::encode(hash));
                    }
                } else {
                    blobs_that_need_to_be_read
                        .entry(path.clone())
                        .and_modify(|entries| {
                            entries.push((i, hash));
                        })
                        .or_insert_with(|| vec![(i, hash)]);
                }
                pkg_list_entry.filenames.push(entry.name().borrow().into());
                pkg_list_entry.hashes.push(hash);
                Ok(())
            })?;
            pkg_list.push(pkg_list_entry);
            Ok(())
        })?;
    let blob_info: std::vec::Vec<tocs::pka::CompressedBlobMetadata> = blob_info.into_values().collect();

    // This part is IO bound because it's mostly reading and writing
    // TODO: it might be possible to do better by memory mmapping both the pkgs and the pka and then copying from source to target (with no intermediate allocation). We would however need to break abstraction and/or unsafely transmute the mmapped memory to a &[u8].
    let blob_iterator = blobs_that_need_to_be_read.into_iter().map(|(pkgpath, entries_to_read)| {
        log::debug!("reading '{}'", pkgpath.display());
        let mut pkgfile = std::io::BufReader::new(std::fs::File::open(&pkgpath).with_context(|| format!("couldn't open pkg '{}'", pkgpath.display()))?);
        let filelist = tocs::pkg::load_filelist(&mut pkgfile).with_context(|| format!("couldn't read pkg '{}'", pkgpath.display()))?;
        let mut entries_to_read = entries_to_read;
        entries_to_read.sort_unstable_by_key(|(i, _)| *i); // read blobs in order
        let blobs: std::vec::Vec<([u8; 32], std::vec::Vec<u8>)> = fallible_iterator::convert(
            entries_to_read
                .into_iter()
                .map(|(i, hash)| tocs::pkg::get_data_raw_from_file(&mut pkgfile, &filelist[i]).map(|data| (hash, data))),
        )
        .collect()
        .with_context(|| format!("couldn't read entries of pkg '{}'", pkgpath.display()))?;
        Ok(fallible_iterator::convert(blobs.into_iter().map(Ok::<_, anyhow::Error>)))
    });

    log::info!("writing '{}' (this will take some time)", out.as_ref().display());
    tocs::pka::write_pka_from_compressed_data(out.as_ref(), force, pkg_list, blob_order, blob_info, fallible_iterator::convert(blob_iterator).flatten()).map_err(anyhow::Error::msg)?;

    log::info!("done");
    Ok(())
}
