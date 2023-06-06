tbled
=====

`tbled` is an editor for `.tbl` files for the Trails of Cold Steel games.

`tbled` is a complex program.
To ensure you don't lose any data due to possible bugs, I recommend to BACK UP ALL DATA FREQUENTLY.
If you notice any issues with data integrity, please do report them.

`tbled` currently supports the following versions of the games:

- Trails of Cold Steel (XSeed PC version, English language only)
- Trails of Cold Steel II (XSeed PC version, English language only)
- Trails of Cold Steel III (NISA PC version)
- Trails of Cold Steel IV (NISA Switch version)

The following versions are untested but likely work:

- Trails of Cold Steel III (NISA Switch version)
- Trails of Cold Steel IV (NISA PC version)

The following versions are explicitely not supported at this time:
- Trails of Cold Steel (XSeed PC version, Japanese language)
- Trails of Cold Steel II (XSeed PC version, Japanese language)

Other versions you'll have to test yourself.

Usage
-----

First off, load a set of tbl schemas from the "Schemas" menu.
You can select one of the builtin schemas (one for each supported game), or import your own (see more below).
Make sure to select the right version or loading the `.tbl` files will likely fail.

Then, load a tbl file under "Load" and select a `.tbl` file.
These files are located in the `data\text` folders relative to the installation path.
If loading was successful, choose an entry type on the right to see all entries of that type in the loaded file.

Editing works via the edit field:

- Clicking on a cell in the table selects that cell and deselects all other cells.
- Double clicking a cell copies the content of that cell into the edit field.
- To edit the text in the edit field, make sure that it is active (e.g. by clicking on it).
- Once you're done editing, press Enter to copy the content of the edit field into the currently selected cell.

If you want to save your changes, select "Save" to overwrite the current file or "Save as" to write to a different file.

![A screenshot of `tbled` in action. Some UI elements are labeled](screenshot.png)

You can also use the CSV import/export function for editing:

- Load a .tbl, export a table (all entries with a given header) to CSV
- Edit the CSV in your favourite spreadsheet editor
- Import the updated CSV
- Save the updated .tbl file

When importing a CSV, under certain circumstances the order of .tbl entries may change relative to the entries that have a different header.
It's unlikely that this would cause any problems, but I cannot fully exclude the possibility so just be aware of that.

Search / row filters
--------------------

`tbled` supports some basic search/filter capabilities via the search bar (right of the edit bar).

- If the content of the search bar contains a colon (`:`), the part before that is an attribute name and the part after is a search string.
  Only rows for which the attribute value (column) contains the search string will be shown.
- If the content of the search bar contains an equal sign (`=`), the part before that is an attribute name and the part after is a search string.
  Only rows for which the attribute value exactly matches the search string will be shown.
- mutatis mutandis for `>` (only show values bigger than the search value) and `<` (only show values smaller than the search value).
- In all other cases, only rows for which any attribute contains the search string will be shown.

For example:

- If you search "`rean`", rows which contain "`rean`" anywhere will be shown. Search is case insensitive.
- If you search "`description:unite`" (without space!), rows for which the `description` contains the string `unite` will be shown. Search is case insensitive.
- If you search "`element=5`" (without space!), rows for which `element` equals `5` will be shown. Text comparison is case sensitive.
- If you search "`effect[0] id>100`" (with exactly one space!), rows for which the id of the first effect is bigger than `100` will be shown.

Combining searches is not possible and no further extensions are planned.
If you need more complex searches, you can convert the data to a sqlite database using the `sqlite_conv` utility and query that database using SQL.

Custom schemas
--------------

The format for custom schemas is a subset of JSON.
The recommended way to make custom schemas is to export one of the builtins via the "Schemas" menu, edit the file, and then load the edited Schemas from the "Schemas" menu.
The exact format is not documented, but it should be easy enough to figure out.
If you find out something about fields which are unknown/opaque in the builtin schemas, do let me know so I can amend the builtins.


Warnings
--------

The tool does certain checks to detect discrepancies in the data it loads.
In some cases, these checks can be too aggressive and reject data which would work perfectly well when loaded by the game.
Files with known schemas can be read under certain circumstances even when these checks fail.
For those cases, a list of warnings will be shown when loading the file.
If there are warnings, it is your responsibility to check every warning and decide whether or not to fix the detected problem, as well as how to fix it.

If there are warnings when loading files that have previously been written with this tool, that's a bug.

Releases (downloads & changelogs)
---------------------------------

### [v1.0-rc1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v1.0-rc1/tbled-v1.0-rc1.exe)

- Warning: This release has not been tested as thoroughly as the others.
- Show warning dialog automatically (if there are warnings) when a file is loaded.
- Support for user-provided schemas
- Menu and workflow reorganization
- Warnings are no longer issued when opening a file with a known fix.
- Improved search:
  - allow restricting substring search to a single attribute
  - add comparison search (specific attribute value has to equal/exceed/subceed search value)

### [v0.5](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.5/tbled-v0.5.exe)

- Reduced size of the release binary
- Added tooltips to UI elements whose function isn't immediately obvious
- Changed undo/redo shortcuts to Ctrl+u and Ctrl+r, respectively (due to technical limitations)
- Added option to adjust cell size
- Modified tbl state tracking with a new wrapper widget
  - improved display performance
  - added row filters (search)
  - allows hiding and reordering columns
- Better looking table, monospaced font for numeric and hex data

### [v0.4](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.4/tbled-v0.4.exe)

- warning popup will not be shown anymore in cases where the warnings are very minor
- fixed a bug where "Save As" would overwrite the wrong file
- undo/redo changes
- preserve row order (except when importing from CSV)
- cleanup of code used to read/write and related code
- CSV import

### [v0.3.1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.3.1/tbled-v0.3.1.exe)

- Fixed a bug that prevented saving files

### [v0.3](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.3/tbled-v0.3.exe):

- schemas for CS4
- improved handling of slightly-inconsistent files
- improved warning reporting

### [v0.2](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.2/tbled-v0.2.exe):

- improved UI
- improved performance
- support for saving to the current file (overwriting)
- added schemas for CS1 & CS2
- added a few schemas for CS3
- CSV export
- allow duplicating & removing entries

### [v0.1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.1/tbled.exe):

- initial public version

Planned features
----------------

- schemas for Trails into Reverie (NISA PC)
  - planned for the 7th or 8th of July
- make a solid manual test plan for the gui before 1.0
