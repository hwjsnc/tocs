pkgtool
=======

`pkgtool` is a CLI tool for reading and writing .pkg and .pka files from the Trails of Cold Steel series.

So far, it supports:
- reading metadata for analysis
- checking pkg/pka integrity (or parser integrity I guess)
- extracting assets from .pkg files
- extracting assets from .pka files
- converting extracted assets back into .pkg files

Note:
- `pkgtool` has several subcommands. They are not fully documented here to avoid duplication and inconsistencies, but if you run the program with `--help` flag, you can get a list of available commands/subcommands/flags and what they do.
- Extracting all assets from a .pka will take a lot of space (e.g. ~50 GiB for Cold Steel 4).
  For an estimate, use the `pkg info` command.
- Extracting all assets from a .pka will also take some time (up to a few minutes, depends mostly on your disk speed).
  If you only want to extract a handful of assets, that is possible and will be much faster.
  However, if you also want to change assets and update the .pka, you have to extract all assets.
- .pka integrity checks are very slow.
  They might take many minutes or even hours depending on your hardware (mostly CPU).

To get more detail about what's going on (e.g. to see if progress is being made), you can set the [`RUST_LOG` environment variable](https://docs.rs/simple_logger/4.0.0/simple_logger/struct.SimpleLogger.html#method.env) to `debug` or `trace`.
This will likely produce a lot of output and may or may not slow down execution.
