set -ex

find . -name 'Cargo.toml' | while read path; do
  dir="$(dirname ${path})"
  printf "%s\n" "$dir"
  cd "$dir"
  cargo fmt --check
  cargo test --release --locked
  cargo test --offline
  cargo clippy
  cd -
done
