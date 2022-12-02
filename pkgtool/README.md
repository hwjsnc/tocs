pkgtool
=======

`pkgtool` is a CLI tool for reading and writing .pkg and .pka files from the Trails of Cold Steel series.

It supports:
- reading metadata for analysis
- checking pkg/pka integrity (or parser integrity depending on your view)
- extracting assets from .pkg files
- extracting assets from .pka files
- converting extracted assets back into .pkg files
- converting extracted assets back into .pka files

Note:
- `pkgtool` has several subcommands. They are not fully documented here to avoid duplication and inconsistencies, but if you run the program with the `--help` flag, you can get a list of available commands/subcommands/flags and what they do.
- Extracting all assets from a .pka will take a lot of space (e.g. ~70 GiB for Cold Steel 3).
  For an estimate, use the `pkg info` command.
- Extracting all assets from a .pka will also take some time (up to a few minutes, depends mostly on your disk speed).
  If you only want to extract a handful of assets, that is possible and will be much faster.
  However, if you also want to change assets and update the .pka used by the game, you have to extract all assets.
- .pka integrity checks are very slow.
  They might take many minutes or even hours depending on your hardware (mostly CPU).
- Writing a .pka is a complex task (especially when you want to do it fast and not take up 10 of GB of main memory).
  As a result, bugs are possible.
  I recommend checking .pka integrity after packing.

To get more detail about what's going on (e.g. to see if progress is being made), you can set the [`RUST_LOG` environment variable](https://docs.rs/simple_logger/4.0.0/simple_logger/struct.SimpleLogger.html#method.env) to `debug` or `trace` before running the program.
This will likely produce a lot of output and may or may not slow down execution.

Related projects / information
------------------------------

- [Sen-no-Kiseki-Sharp by Sewer56](https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp), a CLI utility for .pkg files
- [Explanation of pkg file structure by Sewer56](https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/PKG-File-Structure.md)
- [Explanation of LZSS compression by Sewer56](https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/blob/master/docs/Compression-Explained.md)
- [Description of pka file structure by CeruleanSky](https://github.com/Sewer56/Sen-no-Kiseki-PKG-Sharp/pull/4/files)
- [Discussion of asset compression by Durante](https://steamcommunity.com/games/991270/announcements/detail/1806446683762590792)

Releases (downloads & changelogs)
---------------------------------

### [v1.0.0](https://git.sr.ht/~quf/tocs/refs/download/pkgtool-v1.0.0/pkgtool-v1.0.0.exe)

- initial public version
