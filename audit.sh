set -ex

find . -name 'Cargo.toml' | while read path; do
  dir="$(dirname ${path})"
  printf "%s\n" "$dir"
  cd "$dir"
  cargo audit --deny unmaintained --deny unsound
  cd -
done
