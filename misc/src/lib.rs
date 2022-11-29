use fallible_iterator::FallibleIterator;
use std::path::Path;

pub fn fallible_read_dir<P: AsRef<Path>>(path: P) -> Result<impl FallibleIterator<Item = std::fs::DirEntry, Error = std::io::Error>, std::io::Error> {
    std::fs::read_dir(path).map(fallible_iterator::convert)
}

pub fn fallible_recursive_read_dir<P: AsRef<Path>>(path: P) -> impl FallibleIterator<Item = walkdir::DirEntry, Error = walkdir::Error> {
    fallible_iterator::convert(walkdir::WalkDir::new(path).into_iter())
}
