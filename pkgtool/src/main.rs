use anyhow::Context as _;
use clap::Parser as _;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;
use sha2::Digest as _;
use std::io::Seek as _;
use std::ops::DerefMut as _;

#[derive(clap::Parser)]
#[clap(version, about)]
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
    /// Check .pkg consistency (for testing)
    Check {
        /// Path to .pkg
        path: std::path::PathBuf,
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
        /// Overwrite existing stat files
        #[clap(long, short)]
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
        CliArgs::Pka(PkaArgs::Stats { path, force }) => pka_stats(path, force),
        CliArgs::Pka(PkaArgs::Check { path }) => pka_check(path),
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

fn name_is_ok(name: &str) -> bool {
    name.is_ascii() && !name.chars().any(|c| c.is_ascii_control() || (c.is_ascii_whitespace() && c != ' ')) && !name.contains('/') && !name.contains('\\')
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
        for direntry in walkdir::WalkDir::new(path.as_ref()) {
            let direntry = direntry.unwrap();
            let path = direntry.path();
            if direntry.file_type().is_file() {
                pkg_check(&path)?;
            }
        }
        return Ok(());
    }

    log::info!("Checking file {}", path.as_ref().display());

    let data = std::fs::read(&path).with_context(|| format!("couldn't read pkg file '{}'", path.as_ref().display()))?;
    let mut cursor = std::io::Cursor::new(&data);
    let filelist = tocs::pkg::load_filelist(&mut cursor).with_context(|| format!("couldn't parse pkg file '{}'", path.as_ref().display()))?;

    let offset_after_metadata = cursor.stream_position().expect("cursor stream_position should not fail");

    /* check that data does not overlap and leaves no space empty */
    let mut filelist = filelist;
    filelist.sort_by_key(|e| e.data_offset);
    let filelist = filelist;
    let mut expected_start: u32 = offset_after_metadata.try_into().expect("data should be < 4GiB");
    for e in filelist.iter() {
        log::debug!("checking offset of entry {}", e.filename);
        if e.data_offset != expected_start {
            panic!("{}: unexpected start ({} rather than {})", e.filename, e.data_offset, expected_start);
        }
        expected_start += e.compressed_size;
    }
    if <u32 as TryInto<usize>>::try_into(expected_start).expect("32 bits please") != data.len() {
        panic!("trailing data starting at offset {} (0x{:x})", expected_start, expected_start);
    }

    /* check read->decompress->recompress->decompress loop */
    for e in filelist.iter() {
        log::debug!("Testing {}", e.filename);
        assert!(name_is_ok(&e.filename));
        let compressed_data = tocs::pkg::get_data_raw(&mut cursor, &e)?;
        let decompressed_data = tocs::pkg::get_data(&mut cursor, &e)?;
        let recompressed_data = e.compression.compress(&decompressed_data).map_err(anyhow::Error::msg)?;
        let redecompressed_data = e
            .compression
            .decompress_with_exact_output_size(&recompressed_data, e.decompressed_size.try_into().expect("32 bits please"))
            .map_err(anyhow::Error::msg)?;
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

    println!("ok.");

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
    let blobs: std::collections::HashMap<_, _> = blobs.into_iter().map(|blob| (blob.hash, blob.clone())).collect();
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

    let blobs: std::collections::HashMap<_, _> = blobs.into_iter().map(|blob| (blob.hash, blob.clone())).collect();

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
        log::debug!("checking pkg {} name", pkg.name);
        assert!(name_is_ok(&pkg.name));
        for (name, hash) in pkg.filenames.iter().zip(pkg.hashes.iter()) {
            log::debug!("checking file {} name", name);
            assert!(name_is_ok(&name));
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
    let sha256 = sha2::Sha256::digest(&decompressed_data);
    let sha256: [u8; 32] = sha256.as_slice().try_into().expect("SHA256 result should be 32 byte long");
    if blob.hash != sha256 {
        return Err(anyhow::Error::msg(format!(
            "blob {}: hash does not match sha256 of decompressed data {}",
            hex::encode(blob.hash),
            hex::encode(sha256)
        )));
    }
    if decompressed_data != redecompressed_data {
        return Err(anyhow::Error::msg(format!("blob {}: recompress->decompress changes data", hex::encode(blob.hash))));
    }

    let delta = (recompressed_data.len() as f64 - compressed_data.len() as f64).abs() / std::cmp::max(recompressed_data.len(), compressed_data.len()) as f64;
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
