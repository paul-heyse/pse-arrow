# Search, Discovery, and Content-Inspection Tooling for Coding Agents

Scope: Linux/Unix command-line tools, modern search utilities, Git tools, structured-data processors, and code-intelligence interfaces.\
Research date: October 31, 2026

Coding agents use a much broader toolkit than `ripgrep` (`rg`) to discover, inspect, and understand files. The most common pattern is to locate relevant content, narrow the results, read selected portions, and escalate to structural or semantic analysis when text alone is insufficient.

`sed`, which you mentioned, is particularly common because it lets an agent read an exact range of lines without placing an entire file into its context.

There are two important layers:

- Agent-native tools: `Read`, `Grep`, `Glob`, `list_directory`, and similar operations exposed directly by an agent's runtime.
- System tools: `rg`, `sed`, `awk`, `find`, `git`, `jq`, and executables invoked through a shell.

These frequently overlap. For example, Claude's documented agent toolset includes dedicated `Read`, `Glob`, and `Grep` tools alongside Bash; OpenAI's Codex guidance explicitly discusses `rg`, `sed`, `cat`, `git show`, `nl`, and `wc` for exploration. The exact implementation and availability depend on the agent environment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://platform.claude.com\&sz=32)

Claude Platform Docs+1



## 1. Complete capability map

The table below distinguishes common baseline utilities from more specialized tools. It is a capability inventory, not a measured frequency ranking of agent commands.

| Operation                        | Principal tools                                                              | What they contribute                                              |
| -------------------------------- | ---------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| 1. Discover files                | `ls`, `find`, `fd`, `tree`, `rg --files`                                     | Locate files, enumerate directories, filter by paths and metadata |
| 2. Search contents               | `rg`, `grep`, `git grep`, `ag`, `ack`, `ugrep`                               | Find literals, regex matches, and matching files                  |
| 3. Read exact content            | `cat`, `sed`, `head`, `tail`, `nl`, `bat`, `less`                            | Read files, targeted line ranges, or surrounding context          |
| 4. Filter and transform text     | `awk`, `sed`, `cut`, `tr`, `sort`, `uniq`, `comm`, `paste`                   | Select fields, transform streams, aggregate or compare results    |
| 5. Inspect file properties       | `file`, `stat`, `wc`, `du`, `readlink`, `realpath`                           | Identify type, size, encoding hints, filesystem properties        |
| 6. Inspect Git state and history | `git status`, `git diff`, `git show`, `git log`, `git blame`, `git ls-files` | See changes, historical versions, ownership, tracked files        |
| 7. Query structured data         | `jq`, `yq`, `xq`, `mlr`, `csvkit`, `duckdb`, `sqlite3`                       | Query JSON, YAML, XML, CSV, and tabular data                      |
| 8. Inspect logs and streams      | `tail -f`, `less`, `journalctl`, `zgrep`, `rg`, `lnav`                       | Examine large logs, live output, and compressed records           |
| 9. Inspect non-text formats      | `pdftotext`, `unzip`, `tar`, `strings`, `xxd`, `readelf`, `objdump`          | Extract document, archive, and binary information                 |
| 10. Search source structure      | `ast-grep`, `Semgrep`, `tree-sitter`, `comby`                                | Find syntax-aware patterns and code constructs                    |
| 11. Navigate code semantics      | Language servers, `ctags`, `cscope`, `gtags`, SCIP                           | Identify declarations, references, definitions, and relationships |
| 12. Search indexed repositories  | GitHub Code Search, Sourcegraph, Zoekt                                       | Query indexed code across repositories and revisions              |
| 13. Retrieve external content    | `curl`, `wget`, `gh api`, web-fetch tools                                    | Retrieve documentation, remote files, and API responses           |
| 14. Compose operations           | Shell pipelines, `xargs`, `tee`, `parallel`, `fzf`                           | Combine discovery, filtering, selection, and inspection           |

The most important distinction is the type of question being answered:

| Question                                  | Natural tool                      |
| ----------------------------------------- | --------------------------------- |
| Where is a file?                          | `fd`, `find`, `rg --files`        |
| Which files mention a string?             | `rg`                              |
| Show lines 120–180 of a known file        | `sed`, `bat`                      |
| What does this field-separated text mean? | `awk`                             |
| What does this JSON object contain?       | `jq`                              |
| How did this code change?                 | `git log`, `git show`, `git diff` |
| Which code has this syntactic form?       | `ast-grep`                        |
| Where is this symbol actually defined?    | Language server / semantic index  |

That progression is important: each step requires a different information model, and a higher-level search tool is not necessarily better for a lower-level task.

## 2. Filesystem discovery: `ls`, `find`, `fd`, `tree`, `rg --files`

These answer which files exist and which ones are relevant? They generally operate on filenames, directory structure, or filesystem metadata rather than file contents.

| Tool           | Distinctive capabilities                                                        | Main limitation                                             |
| -------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `ls`           | Directory listings, file types, permissions, timestamps, sizes                  | Lists rather than recursively searches                      |
| `tree`         | Hierarchical directory visualization, depth limits, path exclusions             | Large trees create excessive output                         |
| `find`         | Complex metadata predicates, Boolean expressions, filesystem traversal, actions | More verbose syntax; no default Git ignore awareness        |
| `fd`           | Fast, parallel, ignore-aware filename matching, extension/type filters          | Does not expose all of `find`'s predicates                  |
| `rg --files`   | Fast enumeration using ripgrep's filtering rules                                | Primarily path filtering, not arbitrary metadata predicates |
| `git ls-files` | Tracked, staged, modified, deleted, and untracked path enumeration              | Git-centric; not a general filesystem walker                |
| `eza`          | Enhanced directory listings and tree display with Git metadata                  | Primarily interactive presentation                          |

### Useful patterns

```
# Examine directory structure
ls -lah
tree -L 3

# Find source files by filename
fd 'parser' -e rs

# Discover files through ripgrep's ignore-aware walker
rg --files -g '*.py'

# Discover files modified in the last day
find . -type f -mtime -1

# Find files larger than 10 MiB
find . -type f -size +10M

# Enumerate tracked files
git ls-files
```

The important distinction between `fd` and `find` is that `fd` optimizes the common filename-search experience, whereas `find` is a general filesystem predicate engine.

`find` can combine metadata tests, traversal restrictions, logical conditions, and actions such as `-exec`, `-execdir`, and `-print0`. These are relevant when selection criteria involve permissions, timestamps, ownership, inode properties, or relationships not expressible through filename patterns alone.

`fd` uses regex matching by default, supports glob mode, respects Git-style ignore rules, and can execute commands over selected files using `-x` or batch them using `-X`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Why agents use several of these: listing a directory is a cheap orientation operation; filtering paths reduces search scope; reading filesystem metadata can establish which of several candidate files is relevant before any content is loaded.

Sources:

[https://www.gnu.org/software/findutils/manual/html_node/find_html/](https://www.gnu.org/software/findutils/manual/html_node/find_html/?utm_source=chatgpt.com)

[https://github.com/sharkdp/fd](https://github.com/sharkdp/fd?utm_source=chatgpt.com)

[https://git-scm.com/docs/git-ls-files](https://git-scm.com/docs/git-ls-files?utm_source=chatgpt.com)

## 3. Viewing exact content: `sed`, `cat`, `head`, `tail`, `nl`, `bat`, `less`

This is the family most directly relevant to your `sed` example.

### 3.1 `sed`: targeted reading and stream transformations

`sed` is a stream editor, not fundamentally a file viewer. It reads input, applies an ordered program of editing commands, and writes output.

However, one specific capability makes it extremely useful to agents:

```
sed -n '120,180p' src/parser.rs
```

This prints lines 120–180, inclusive.

The syntax has three parts:

| Component | Meaning                                      |
| --------- | -------------------------------------------- |
| `-n`      | Suppress automatic printing of every line    |
| `120,180` | Apply the command only to this address range |
| `p`       | Print the selected lines                     |

Without `-n`, `sed` normally prints every line after processing, which makes a simple range-inspection operation behave very differently.

Other useful `sed` capabilities:

| Capability                    | Example                          | Result                           |
| ----------------------------- | -------------------------------- | -------------------------------- |
| Print one line                | `sed -n '42p' file`              | Line 42                          |
| Print a range                 | `sed -n '20,80p' file`           | Lines 20–80                      |
| Print matching lines          | `sed -n '/TODO/p' file`          | Lines matching regex             |
| Print a pattern-bounded range | `sed -n '/^BEGIN/,/^END/p' file` | Text between delimiters          |
| Preview substitutions         | `sed 's/old/new/g' file`         | Transformed text on stdout       |
| Quit after a location         | `sed -n '120,180p;181q' file`    | Avoid processing beyond line 180 |
| Edit in place                 | `sed -i 's/old/new/g' file`      | Modifies the file                |

Two less obvious properties matter:

Sequential scanning. `sed -n '100000,100100p'` does not normally seek directly to line 100000. It reads through preceding lines to identify the requested range. Bounded output is not equivalent to indexed access.

Two-buffer programming model. GNU `sed` maintains a pattern space and a hold space. Commands such as `N`, `D`, `H`, `G`, `b`, and `t` permit multiline manipulation and conditional branching. Thus, it can perform considerably more sophisticated transformations than simple line substitutions, though complex programs are generally less transparent than structured parsers.

GNU `sed` also supports extended regex syntax with `-E`. POSIX and GNU variants have some behavioral and option differences, particularly in-place editing and advanced address syntax.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.gnu.org\&sz=32)

gnu.org+2



Primary manual: [https://www.gnu.org/software/sed/manual/sed.html](https://www.gnu.org/software/sed/manual/sed.html?utm_source=chatgpt.com)

### 3.2 The other content viewers

| Tool                  | Best use                            | Distinctive behavior                                           |
| --------------------- | ----------------------------------- | -------------------------------------------------------------- |
| `cat`                 | Read a complete small file          | Writes content unchanged to stdout                             |
| `head`                | Inspect beginning of a file         | Can stop reading after enough lines                            |
| `tail`                | Inspect end of a file               | Also supports following appended data                          |
| `nl`                  | Annotate lines with numbers         | Configurable line numbering and sections                       |
| `bat`                 | Read syntax-highlighted source      | Adds line numbers, syntax highlighting, Git markers            |
| `less`                | Navigate a large file interactively | Search, paging, and navigation without printing the whole file |
| `more`                | Simple forward paging               | Less flexible than `less`                                      |
| `fold`                | Reformat long lines for display     | Alters visual line representation                              |
| `cat -A` / `sed -n l` | Inspect hidden text characters      | Makes tabs, line endings, and nonprinting characters visible   |

Representative commands:

```
cat pyproject.toml
head -n 80 src/main.rs
tail -n 100 application.log
nl -ba src/main.rs | sed -n '120,180p'
bat --style=numbers --line-range 120:180 src/main.rs
less -N application.log
```

`nl -ba` numbers all lines, including blank ones. This is useful when the displayed content needs to preserve a stable relationship with actual source line numbers.

`bat` adds syntax highlighting, source-aware formatting, line ranges, and optionally Git change indicators. Its `--style` and `--color` options also allow relatively plain output, which is preferable when presentation escape codes would interfere with machine consumption.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/sharkdp/bat](https://github.com/sharkdp/bat?utm_source=chatgpt.com)

`less` is extremely useful for a human-operated terminal but less naturally suited to a noninteractive agent. Agents with shell output capture generally prefer deterministic, bounded commands such as `sed`, `head`, or an agent-native file reader. A dedicated `Read` tool may also avoid launching a shell process and preserve file metadata more explicitly.

### 3.3 Comparison of content-reading mechanisms

| Requirement                          | Tool                  | Why                                                        |
| ------------------------------------ | --------------------- | ---------------------------------------------------------- |
| Entire known small file              | `cat` / native `Read` | Minimal transformation                                     |
| Known line interval                  | `sed -n`              | Precise output range                                       |
| Beginning of file                    | `head`                | Simple bounded read                                        |
| End of file                          | `tail`                | Efficient access to recent content                         |
| Preserve visible source line numbers | `nl` / `bat`          | Output remains traceable to source                         |
| Human interactive inspection         | `less` / `bat`        | Navigation and presentation                                |
| Search with surrounding lines        | `rg -n -C`            | Search and contextual inspection in one call               |
| Query JSON fields                    | `jq`                  | Preserves data structure                                   |
| Inspect a historical source file     | `git show`            | Reads Git content rather than current working-tree content |

The last three are particularly important. `sed` is not automatically the best tool for every targeted read: when the location is unknown, `rg` can discover it; when the desired information is structured, a format-aware query is more precise; when the source version matters, Git can be authoritative.

## 4. Text filtering and processing: `grep`, `awk`, `cut`, `sort`, `uniq`, `tr`

These tools act on text streams. They are often composed with search tools to transform a large result into a smaller and more informative representation.

### 4.1 `grep` and its alternatives

| Tool       | Principal role                    | Differentiator                                                |
| ---------- | --------------------------------- | ------------------------------------------------------------- |
| `grep`     | Standard POSIX text filtering     | Extremely widespread, works well in pipelines                 |
| `rg`       | Recursive codebase search         | Ignore-aware traversal, optimized matching, structured output |
| `git grep` | Search tracked repository content | Search working tree, index, or Git tree objects               |
| `ag`       | Recursive source searching        | The Silver Searcher; older alternative to `rg`                |
| `ack`      | Developer-oriented text search    | Perl-based, developer-friendly file-type filtering            |
| `ugrep`    | Extended grep/search interface    | Rich matching, fuzzy search and archive-related functionality |

GNU `grep` itself has several expression modes: basic regex (`-G`), extended regex (`-E`), fixed strings (`-F`), and PCRE (`-P`, where supported). It also provides context, line numbers, matching-only output, null-delimited filenames, and recursive search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.gnu.org\&sz=32)

gnu.org



Source: [https://www.gnu.org/software/grep/manual/](https://www.gnu.org/software/grep/manual/?utm_source=chatgpt.com)

For most ordinary source-tree searches, `rg` is a strong default because its automatic path filtering is well suited to code repositories. But `grep` remains a natural choice when filtering text already flowing through a pipe, or when a portable shell script must work without ripgrep installed.

### 4.2 `awk`: a programmable record-processing engine

`awk` is substantially more capable than a simple filter. It combines record splitting, field extraction, pattern matching, arithmetic, associative arrays, aggregation, and output formatting.

The fundamental model is:

```
pattern { action }
```

By default, each input line is a record and whitespace-separated components are fields: `$1`, `$2`, and so forth. `$0` represents the complete record.

Examples:

```
# Extract the first field
awk '{print $1}' file.txt

# Extract one CSV-like field (not full CSV parsing)
awk -F ',' '{print $3}' file.txt

# Print a line interval
awk 'NR >= 120 && NR <= 180' file.txt

# Count records by first field
awk '{count[$1]++}
     END {for (key in count) print key, count[key]}' file.txt
```

The CSV-like example is only valid for simple delimited data: ordinary `awk -F,` does not implement quoted CSV field rules, escaped quotes, or embedded delimiters.

Useful advanced features include user-defined functions, conditional expressions, associative arrays, record and field separators (`RS`, `FS`), output separators (`ORS`, `OFS`), and `BEGIN`/`END` actions. GNU `gawk` adds capabilities beyond basic POSIX `awk`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.gnu.org\&sz=32)

gnu.org+1



Source: [https://www.gnu.org/software/gawk/manual/](https://www.gnu.org/software/gawk/manual/?utm_source=chatgpt.com)

### 4.3 Smaller transformation utilities

\| Tool | Function | Example |\
\|---|---|---|\
\| `cut` | Select character or delimiter-based fields | `cut -d: -f1 /etc/passwd` |\
\| `sort` | Sort records; supports numeric and keyed ordering | `sort -n values.txt` |\
\| `uniq` | Count or remove adjacent duplicate records | `uniq -c` |\
\| `comm` | Compare two sorted sets of lines | `comm -12 a.txt b.txt` |\
\| `join` | Join two sorted streams by a key | `join a.txt b.txt` |\
\| `paste` | Combine streams horizontally | `paste file1 file2` |\
\| `tr` | Translate or remove characters | `tr '[:upper:]' '[:lower:]'` |\
\| `wc` | Count lines, words, bytes, or characters | `wc -l file.txt` |\
\| `tee` | Copy a stream to stdout and one or more files | `command | tee results.txt` |\
\| `column` | Align tabular text for display | `column -t -s, file.csv` |

There are two particularly consequential distinctions:

- `uniq` works on adjacent duplicate lines. `sort | uniq` or `sort -u` is usually needed for deduplication of an arbitrary stream, but sorting destroys the original order.
- `cut` is a delimiter/position tool, not a CSV or JSON parser. For structured records, use tools that recognize the actual format.

Combined operations are frequently more useful than any single program:

```
# Find and count matching filenames by extension
rg -l 'TODO' |
  awk -F. 'NF > 1 {print $NF}' |
  sort |
  uniq -c |
  sort -nr
```

This is a convenient exploratory approximation, not a robust filename parser: files can have no extension, multiple suffixes, or newlines in their names. When those distinctions matter, structured or NUL-safe processing is preferable.

Source: [https://www.gnu.org/software/coreutils/manual/html_node/](https://www.gnu.org/software/coreutils/manual/html_node/?utm_source=chatgpt.com)

## 5. File identification and metadata: `file`, `stat`, `wc`, `du`

Agents often need to know what kind of content they are about to inspect before choosing a search or viewing strategy.

| Tool         | What it reveals                                       | Distinction                                       |
| ------------ | ----------------------------------------------------- | ------------------------------------------------- |
| `file`       | Likely format or file type                            | Uses content signatures and heuristics            |
| `stat`       | Logical size, timestamps, permissions, inode metadata | Does not interpret content                        |
| `wc`         | Lines, words, bytes, characters                       | Useful to estimate output volume                  |
| `du`         | Filesystem storage usage                              | Not necessarily logical file size                 |
| `readlink`   | Symlink destination                                   | Reveals indirection                               |
| `realpath`   | Resolved canonical path                               | Resolves relative path components and symlinks    |
| `sha256sum`  | Content digest                                        | Useful for detecting byte-identical content       |
| `xxd` / `od` | Hexadecimal or octal byte representation              | Inspect content that is not readable text         |
| `strings`    | Printable character sequences in binary content       | Heuristic extraction, not complete interpretation |

Examples:

```
file data.bin
stat data.bin
wc -l src/parser.rs
du -h build/output.bin
readlink -f current
xxd -l 256 artifact.bin
```

These can prevent common errors, such as treating a binary file as UTF-8, assuming a symlink points to the expected directory, or attempting to load a multi-gigabyte file into an agent's context.

One subtlety: `wc -l` counts newline characters, not strictly logical records. A final line without a terminating newline will not increment that count.

Another: `file` does not guarantee correct type identification. Its detection is useful evidence for choosing a parser or extractor, but not formal file-format validation.

## 6. Git as a content-search and historical inspection system

Git is a separate, extremely important source of information for coding agents. Its value is not limited to version control operations.

Git can answer questions that searching the current working tree cannot: what a file contained previously, which commit introduced a change, how a function evolved, and whether a file is tracked or staged.

### 6.1 Core inspection commands

| Tool                | Principal capability                        | Representative operation             |
| ------------------- | ------------------------------------------- | ------------------------------------ |
| `git status`        | Working-tree and index state                | `git status --short`                 |
| `git diff`          | Compare changed content                     | `git diff -- src/parser.rs`          |
| `git diff --cached` | Inspect staged changes                      | `git diff --cached`                  |
| `git show`          | View commits, Git objects, historical files | `git show HEAD:src/parser.rs`        |
| `git log`           | Search commit history                       | `git log --oneline -- src/parser.rs` |
| `git blame`         | Attribute lines to commits                  | `git blame -L 40,90 src/parser.rs`   |
| `git grep`          | Search Git-selected content                 | `git grep -n 'TODO'`                 |
| `git ls-files`      | Inspect index/working-tree file selection   | `git ls-files -z`                    |
| `git ls-tree`       | Enumerate files in historical trees         | `git ls-tree -r --name-only HEAD`    |
| `git cat-file`      | Examine Git object types and contents       | `git cat-file -p HEAD^{tree}`        |
| `git check-ignore`  | Explain Git exclusions                      | `git check-ignore -v target/file`    |

`git grep` deserves special attention. Unlike ordinary `rg`, it can search Git objects directly.

```
# Current tracked working-tree content
git grep -n 'unsafe'

# Staged/index content
git grep --cached -n 'unsafe'

# Contents at an earlier revision
git grep -n 'unsafe' HEAD~3
```

`git grep` also supports Boolean combinations using `--and`, `--or`, `--not`, and grouping. That is a distinct capability from ripgrep's usual multiple-pattern OR matching.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://git-scm.com\&sz=32)

git-grep Documentation



Source: [https://git-scm.com/docs/git-grep](https://git-scm.com/docs/git-grep?utm_source=chatgpt.com)

### 6.2 Git history search: `-S`, `-G`, and `-L`

These are especially valuable and are easy to overlook.

```
# Find commits that changed the count of a literal string
git log -S 'parse_schema' --oneline

# Find commits whose added/removed lines match a regex
git log -G 'parse_.*schema' --oneline

# Follow history of a recognized function
git log -L :parse_schema:src/parser.rs
```

The distinctions are substantial:

- `git log -S` searches for commits where the number of occurrences of a string changes.
- `git log -G` searches diffs for added or deleted lines matching an expression.
- `git log -L` traces changes to a specific line interval or recognized function, subject to Git's hunk/function matching machinery.

A refactoring could move an expression while preserving its total occurrence count. Such a commit may appear under `-G` without appearing under `-S`.

`git blame` identifies the commit that last modified surviving lines, not necessarily the original author or the first appearance of the concept. It cannot directly explain lines already deleted from the current file.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://git-scm.com\&sz=32)

Searching+1



Sources:

[https://git-scm.com/docs/git-log](https://git-scm.com/docs/git-log?utm_source=chatgpt.com)

[https://git-scm.com/docs/git-blame](https://git-scm.com/docs/git-blame?utm_source=chatgpt.com)

### 6.3 Viewing an earlier file without checkout

```
git show HEAD~2:src/parser.rs
git show HEAD~2:src/parser.rs |
  sed -n '100,160p'
```

This is a powerful combination: Git resolves the correct historical file content, while `sed` limits the displayed lines.

There is no need to modify the current working tree or switch revisions.

For an agent investigating a regression, historical file access is often more informative than searching only the latest code.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://git-scm.com\&sz=32)

git-show Documentation



Source: [https://git-scm.com/docs/git-show](https://git-scm.com/docs/git-show?utm_source=chatgpt.com)

### 6.4 Diff viewers and structural comparisons

| Tool                   | Distinguishing capability                                         |
| ---------------------- | ----------------------------------------------------------------- |
| `diff`                 | Compare text files and produce line-oriented differences          |
| `git diff`             | Compare Git revisions, index, or working-tree content             |
| `delta`                | Syntax-highlighted, word-level and side-by-side diff presentation |
| `difftastic` (`difft`) | Compare source files using syntax trees rather than only lines    |
| `cmp`                  | Determine byte-level equality or first differing byte             |
| `diff3`                | Three-way text comparison                                         |

`delta` is principally a presentation improvement over ordinary diff output. `difftastic` is different: it parses supported languages and compares simplified syntax trees, allowing it to distinguish syntactic changes from some formatting changes.

`difftastic` also supports `--check-only` for testing whether two files have equivalent simplified syntax representations. Its output is not intended to be an applicable patch, and it can be more resource-intensive than line-oriented diffing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/dandavison/delta](https://github.com/dandavison/delta?utm_source=chatgpt.com)

[https://github.com/Wilfred/difftastic](https://github.com/Wilfred/difftastic?utm_source=chatgpt.com)

## 7. Structured-data search: `jq`, `yq`, `duckdb`, `sqlite3`, and related tools

A significant limitation of `grep` and `sed` is that they operate on text rather than a file's logical data model.

For JSON, YAML, XML, CSV, Parquet, and databases, specialized tools can directly select the information of interest without losing important structure.

| Format                | Common tools                             | Capabilities                                                       |
| --------------------- | ---------------------------------------- | ------------------------------------------------------------------ |
| JSON / JSON Lines     | `jq`, `jless`, `fx`                      | Path selection, filtering, transformations, interactive inspection |
| YAML                  | `yq`, `dasel`                            | Nested queries, sequence filtering, multi-document processing      |
| XML                   | `xmlstarlet`, `xmllint`, `xq`            | XPath, validation, formatting, node extraction                     |
| TOML / INI / HCL      | `dasel`, `yq`, format-specific parsers   | Structured key and value retrieval                                 |
| CSV / TSV             | `csvkit`, `mlr`, `xsv`, `qsv`            | Schema-aware field selection and tabular processing                |
| Parquet / Arrow       | `duckdb`, Arrow tooling, `parquet-tools` | Column projection, metadata, SQL, statistics                       |
| SQLite                | `sqlite3`, `duckdb`                      | Schema inspection and SQL queries                                  |
| PostgreSQL            | `psql`                                   | SQL, catalogs, schema introspection                                |
| Mixed tabular formats | `duckdb`, `mlr`                          | Queries, filtering, joins, aggregations                            |

### 7.1 `jq`: the JSON equivalent of a programmable text filter

`jq` is probably the most useful complement to `rg` in agent-oriented workflows.

```
# Select a nested property
jq '.dependencies' package.json

# Filter JSON Lines
jq -c 'select(.level == "error")' events.jsonl

# Return only selected fields
jq '{name, version, dependencies}' package.json

# Inspect keys
jq 'keys' package.json
```

The difference from `sed` is semantic: `jq` operates on JSON objects, arrays, strings, numbers, booleans, and null values rather than matching textual representations.

Advanced capabilities include recursive descent, path enumeration, object transformations, grouping, reductions, and user-defined functions.

`jq --stream` also supports event-like processing of large JSON documents, emitting path/value data rather than constructing the full input representation in the usual manner. This changes the query model and can make very large files tractable, at the cost of more complex filters.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://jqlang.org\&sz=32)

jqlang.org



Source: [https://jqlang.org/manual/](https://jqlang.org/manual/?utm_source=chatgpt.com)

### 7.2 `yq` and `dasel`: structured configuration search

For YAML:

```
yq '.services.api.image' compose.yaml
yq '.services | keys' compose.yaml
```

The example uses Mike Farah's Go-based `yq`. Another tool also named `yq` is a Python wrapper that translates YAML/XML/TOML inputs for processing by `jq`; its CLI syntax and feature boundaries differ. The executable name alone does not uniquely identify the implementation.

Mike Farah's `yq` provides multi-format conversions, nested data access, and YAML-specific features such as tags, anchors, styling, and comment-related transformations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`dasel` offers a more uniform selector language spanning several structured formats. This can reduce the number of distinct query syntaxes needed for mixed configuration datasets, although different formats retain distinct semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Sources:

[https://github.com/mikefarah/yq](https://github.com/mikefarah/yq?utm_source=chatgpt.com)

[https://github.com/TomWright/dasel](https://github.com/TomWright/dasel?utm_source=chatgpt.com)

### 7.3 SQL engines as content-inspection tools

For tabular data, SQL can be a more natural search language than a chain of text filters.

For example:

```
duckdb -c "
SELECT customer_id, sum(amount) AS total
FROM 'transactions.parquet'
GROUP BY customer_id
ORDER BY total DESC
LIMIT 20
"
```

This performs direct Parquet querying through DuckDB without first exporting the file to text.

`duckdb` is useful for CSV, Parquet, Arrow-compatible data, and relational analysis. `sqlite3` is appropriate for directly inspecting SQLite databases and their schemas, with `-readonly` available to prevent writes.

These tools move search from the level of lines and substrings to typed columns, filters, joins, and aggregation. For a structured dataset, this can eliminate large amounts of intermediate output and fragile parsing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.duckdb.org\&sz=32)

DuckDB+1



Sources:

[https://duckdb.org/docs/clients/cli/overview](https://duckdb.org/docs/clients/cli/overview?utm_source=chatgpt.com)

[https://sqlite.org/cli.html](https://sqlite.org/cli.html?utm_source=chatgpt.com)

## 8. Logs, compressed content, archives, and document extraction

Some content is not stored as ordinary source text, and the distinction affects how agents should search it.

### 8.1 Logs and streaming content

| Tool                   | Capabilities                                                           |
| ---------------------- | ---------------------------------------------------------------------- |
| `tail -n`              | Inspect recent log entries                                             |
| `tail -f` / `tail -F`  | Follow appended data, with `-F` handling rotation by filename          |
| `journalctl`           | Query systemd journal entries by unit, time, priority, etc.            |
| `rg` / `grep`          | Search log text and filter output streams                              |
| `lnav`                 | Interactive, format-aware log analysis with SQL queries                |
| `zcat` / `zgrep`       | Read/search gzip-compressed content                                    |
| `zstdcat` / `zstdgrep` | Read/search Zstandard-compressed content where utilities are available |
| `rg -z`                | Search supported compressed streams using external decompressors       |

`lnav` is a notably richer alternative to ordinary log viewing. It can detect formats, merge files chronologically, build error/warning indexes, tail rotated logs, and execute SQLite queries against parsed log records.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

tstack/lnav: Log file navigator · GitHub



Source: [https://github.com/tstack/lnav](https://github.com/tstack/lnav?utm_source=chatgpt.com)

### 8.2 Archives and documents

| Tool                  | Principal purpose                             |
| --------------------- | --------------------------------------------- |
| `unzip -l`            | List ZIP archive members                      |
| `unzip -p`            | Stream a ZIP member to stdout                 |
| `tar -tf`             | List tar members                              |
| `tar -xOf`            | Stream a tar member to stdout                 |
| `pdftotext`           | Extract text from PDFs                        |
| `pandoc`              | Convert supported document formats            |
| `rga` (`ripgrep-all`) | Search supported document and archive formats |
| `pdfgrep`             | Search PDF text                               |
| `exiftool`            | Read embedded file/document metadata          |

A useful distinction is streaming extraction versus persistent extraction:

```
# Read a ZIP member without extracting it to disk
unzip -p archive.zip docs/README.md |
  sed -n '1,100p'

# Extract PDF text into a stream
pdftotext -layout report.pdf - |
  head -n 100
```

`rga` builds on ripgrep with adapters for formats including PDFs, Office documents, archives, and SQLite databases. Unlike ordinary `rg -z`, it can recurse into supported archive containers and search individual extracted members.

It also includes an extracted-content cache. This can improve repeated searches over expensive document formats, but introduces cache lifetime and freshness considerations absent from ordinary uncached `rg` searches.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/phiresky/ripgrep-all](https://github.com/phiresky/ripgrep-all?utm_source=chatgpt.com)

Text extraction has an important information-loss boundary. An extracted PDF text stream may omit layout, tables, graphics, reading order, or coordinate information. A textual match in an extracted document is not necessarily enough to establish where the relevant content appears visually.

### 8.3 Binary inspection

For compiled executables, shared libraries, and artifacts, typical tools include:

| Tool                       | Inspection role                                     |
| -------------------------- | --------------------------------------------------- |
| `file`                     | Identify likely binary format                       |
| `strings`                  | Find embedded printable text                        |
| `xxd`, `od`, `hexdump`     | Inspect raw bytes                                   |
| `readelf`                  | Inspect ELF headers, sections, symbols, relocations |
| `objdump`                  | Disassembly and object-file metadata                |
| `nm`                       | Enumerate symbol tables                             |
| `ldd`                      | Display shared-library dependencies on Linux        |
| `ar`, `readelf`, `objdump` | Inspect static archives and object files            |

For an ELF executable, for example, `readelf -Ws` provides symbol table information that searching the raw binary text would not meaningfully recover.

`strings` is useful for exploratory searching but is not a complete inventory of embedded strings, and its output does not establish how a program uses those strings.

## 9. Syntax-aware source inspection and structural search

This is where search moves beyond raw characters.

Tools in this category can identify language constructs such as function declarations, call expressions, imports, class members, and nested blocks without trying to approximate all of their syntax using regular expressions.

| Tool                      | Search model                                       | Particularly useful capability                      |
| ------------------------- | -------------------------------------------------- | --------------------------------------------------- |
| `ast-grep`                | Tree-sitter AST patterns                           | Syntax-aware matching, source outlines, rewrites    |
| `tree-sitter` CLI         | Parse trees and structural queries                 | Parse inspection, node captures, symbol tagging     |
| `Semgrep`                 | Language-aware rules                               | Structural patterns, security/static-analysis rules |
| `comby`                   | Structural templates and language-aware delimiters | Flexible template matching and rewriting            |
| `difftastic`              | Simplified syntax-tree comparison                  | Understanding structural changes                    |
| Language-specific parsers | Concrete/abstract syntax trees                     | Precise language-specific source introspection      |

### 9.1 `ast-grep`: structured search and compact source viewing

`ast-grep` can match code patterns with metavariables rather than relying only on text.

Its CLI provides `run` for individual queries, `scan` for rules, and `outline` for compact code summaries.

```
# Syntax-aware pattern search
ast-grep run -p 'print($A)' -l python

# Inspect declarations and source structure
ast-grep outline src/parser.rs

# Query and emit structured results
ast-grep run -p 'print($A)' -l python --json
```

`ast-grep outline` is especially relevant to agent-based content viewing. It can display declarations, imports, exports, and nested members before the agent reads the source in full. This provides an intermediate representation between a filename list and complete file contents.

Its outline feature was introduced as an experimental/alpha capability in version 0.44.0. It does not resolve types, follow references, or construct a call graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://ast-grep.github.io\&sz=32)

ast-grep+1



Sources:

[https://ast-grep.github.io/guide/tooling-overview](https://ast-grep.github.io/guide/tooling-overview?utm_source=chatgpt.com)

[https://ast-grep.github.io/guide/outline-code](https://ast-grep.github.io/guide/outline-code?utm_source=chatgpt.com)

### 9.2 `tree-sitter`: direct syntax-tree inspection

The Tree-sitter CLI provides a lower-level interface for language-aware exploration.

| Command             | Role                                           |
| ------------------- | ---------------------------------------------- |
| `tree-sitter parse` | Parse source and inspect syntax-tree structure |
| `tree-sitter query` | Evaluate structural capture queries            |
| `tree-sitter tags`  | Extract supported symbol/tag information       |

The query command supports byte-range and row-range restrictions, meaning a caller can inspect syntax within a bounded region rather than always examining an entire source file's query results.

Tree-sitter produces syntax information, not full compiler-grade semantic resolution. Scope, imports, name binding, trait/interface resolution, and type information generally require further analysis.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://tree-sitter.github.io\&sz=32)

Tree-sitter+1



Source: [https://tree-sitter.github.io/tree-sitter/cli/](https://tree-sitter.github.io/tree-sitter/cli/?utm_source=chatgpt.com)

### 9.3 `Semgrep` and `comby`

`Semgrep` provides structural patterns with metavariables, ellipses, rule composition, and language-specific analysis. It is more rule-oriented than an ordinary textual search utility, and can support static-analysis workflows.

Its `semgrep scan` command runs rules against files or directories. More advanced analysis capabilities, particularly interfile analysis, depend on the engine and product edition; they should not be assumed to be present in every CLI installation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://semgrep.dev\&sz=32)

Semgrep+1



`comby` offers structural template search and replacement with language-aware matching. It is useful when balanced delimiters and code-shaped placeholders are needed, but a full semantic index or dedicated AST rule is unnecessary.

Sources:

[https://docs.semgrep.dev/writing-rules/pattern-syntax](https://docs.semgrep.dev/writing-rules/pattern-syntax?utm_source=chatgpt.com)

[https://github.com/comby-tools/comby](https://github.com/comby-tools/comby?utm_source=chatgpt.com)

### 9.4 `sd`: a focused alternative to `sed` for replacements

Another small but relevant utility is `sd`, written in Rust.

It concentrates on search-and-replace operations, offering simpler regex replacement syntax, literal mode, capture substitution, and multiline replacement support.

Unlike `sed`, it is not a general-purpose stream-editing language with address ranges and hold-space programming.

A consequential operational detail: supplying a filename to `sd` normally means editing it. Its preview mode, `-p`, is available for inspection without committing the replacement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/chmln/sd](https://github.com/chmln/sd?utm_source=chatgpt.com)

## 10. Symbol indexing and semantic code navigation

Syntax-aware search can identify a function call's shape, but cannot necessarily determine which function it calls.

That is the territory of symbol indexes, language servers, and compiler-backed analysis.

| Tool / technology              | What it provides                                                                 | Semantic depth                     |
| ------------------------------ | -------------------------------------------------------------------------------- | ---------------------------------- |
| Universal Ctags                | Indexed declarations and named source constructs                                 | Mostly syntactic                   |
| `cscope`                       | C-oriented symbol, definition, caller/reference search                           | Language-specific                  |
| GNU GLOBAL / `gtags`           | Indexed source cross-references                                                  | Depends on language/parser backend |
| Language Server Protocol (LSP) | Definitions, references, implementations, symbols, diagnostics                   | Depends on language server         |
| SCIP                           | Portable code-intelligence index containing symbol occurrences and relationships | Depends on indexer                 |
| Rust-analyzer, Pyright, etc.   | Language-specific program analysis                                               | Potentially type-aware             |
| Compiler tooling               | HIR, typed IR, resolution, and related program representations                   | Compiler-specific                  |

### 10.1 Universal Ctags

Universal Ctags generates tags for language constructs, commonly including functions, methods, classes, fields, and other declarations.

It can produce traditional tag files, cross-reference output, and JSON output when built with the required JSON support.

This provides lightweight, reusable symbol navigation without requiring the entire language server or compiler analysis stack.

The tradeoff is that discovering a declaration's location is not equivalent to resolving references to that declaration. Syntax-based tagging generally does not establish full binding identity across overloads, scopes, and imports.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Source: [https://github.com/universal-ctags/ctags](https://github.com/universal-ctags/ctags?utm_source=chatgpt.com)

### 10.2 Language Server Protocol

LSP defines a client/server interface for language intelligence. The current published specification is 3.18.

Relevant operations can include definition lookup, type definition, implementation lookup, references, document symbols, workspace symbols, call hierarchy, and type hierarchy.

These operations expose a fundamentally different kind of search: search by program identity rather than textual occurrence.

For example, a textual search for `execute` may find declarations, unrelated methods, comments, and string literals. A language server's reference search can identify occurrences bound to a particular symbol, subject to the server's supported language analysis and project configuration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://microsoft.github.io\&sz=32)

microsoft.github.io+1



Source: [https://microsoft.github.io/language-server-protocol/](https://microsoft.github.io/language-server-protocol/?utm_source=chatgpt.com)

### 10.3 SCIP: persistent semantic navigation data

SCIP is a language-agnostic code-intelligence indexing protocol. It represents symbol occurrences, references, and other navigation information in a portable protobuf-based format.

It differs from LSP in lifecycle:

| LSP                                                  | SCIP                                           |
| ---------------------------------------------------- | ---------------------------------------------- |
| Interactive request/response service                 | Persisted index artifact                       |
| Often maintains an active project analysis state     | Usually produced by an indexing run            |
| Responds to current editor/workspace queries         | Allows subsequent queries over indexed content |
| Freshness depends on language-server synchronization | Freshness depends on index regeneration        |

The SCIP project documents indexers for Rust, Python, TypeScript/JavaScript, Java-family languages, C/C++, and several others.

The existence of a SCIP index does not guarantee fully resolved cross-language semantics or complete reference coverage. These depend on the individual indexer's capabilities and the code included in its analysis.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/scip-code/scip](https://github.com/scip-code/scip?utm_source=chatgpt.com)

## 11. Indexed, cross-repository search

For large collections of repositories, repeatedly walking every file can be less attractive than maintaining a dedicated search index.

| System                            | Principal approach                  | What it adds beyond local `rg`                                  |
| --------------------------------- | ----------------------------------- | --------------------------------------------------------------- |
| GitHub Code Search                | Hosted indexed code search          | Cross-repository search, qualifiers, Boolean queries            |
| Sourcegraph Code Search           | Repository-wide code search service | Repository/revision filters, symbol and commit/diff searches    |
| Zoekt                             | Positional trigram index            | Fast repeated full-text queries over indexed repositories       |
| Language-server/SCIP indexes      | Semantic source indexing            | Definitions and references rather than substring matches        |
| Embedding-based retrieval systems | Vector or hybrid retrieval          | Approximate conceptual discovery when the exact text is unknown |

### 11.1 GitHub Code Search

GitHub Code Search supports `repo:`, `path:`, `language:`, `symbol:`, exact-string and regex searches, and Boolean operations.

Its index is tied to GitHub-hosted content, making it useful for locating source across projects without cloning each repository locally.

The exact search scope, indexed revisions, access permissions, and query limits differ from a local working-tree search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.github.com\&sz=32)

GitHub Docs



Source: [https://docs.github.com/en/search-github/github-code-search/understanding-github-code-search-syntax](https://docs.github.com/en/search-github/github-code-search/understanding-github-code-search-syntax?utm_source=chatgpt.com)

### 11.2 Sourcegraph and Zoekt

Sourcegraph provides repository-scoped search, code navigation, commit/diff search, and search across revisions. Its indexed search infrastructure includes Zoekt.

Zoekt uses positional trigrams to narrow candidate content before matching. This changes the economics of repeated searches: indexing incurs preparation, storage, and refresh costs, but subsequent queries can avoid repeated whole-corpus traversal.

Its indexes may represent multiple repository branches. Search-result freshness depends on the indexing and synchronization process, unlike a direct `rg` invocation against the filesystem's current visible state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://sourcegraph.com\&sz=32)

Sourcegraph docs+2



Sources:

[https://sourcegraph.com/docs/code-search](https://sourcegraph.com/docs/code-search?utm_source=chatgpt.com)

[https://github.com/sourcegraph/zoekt](https://github.com/sourcegraph/zoekt?utm_source=chatgpt.com)

Important distinction: an inverted/trigram code-search index and a semantic symbol index solve different problems. Combining them can enable fast candidate discovery followed by precise reference resolution, but neither mechanism alone guarantees both.

## 12. Search orchestration: shell pipelines, `xargs`, `fzf`, and agent-native tools

This category does not introduce another search algorithm. Its purpose is to compose operations and manage how results move between them.

### 12.1 Core orchestration tools

| Tool                         | Purpose                                       | Agent-use distinction                                     |
| ---------------------------- | --------------------------------------------- | --------------------------------------------------------- |
| Shell pipe (`\\|`)           | Stream stdout into the next command           | Avoids intermediate files                                 |
| `xargs`                      | Convert input records into command arguments  | Supports batching and NUL-safe paths                      |
| `find -exec`                 | Run commands on discovered filesystem entries | Preserves path handling without parsing printed filenames |
| `fd -x` / `fd -X`            | Execute commands per result or batch          | Convenient discovery/execution composition                |
| GNU `parallel`               | Run independent commands concurrently         | Adds scheduling and ordering concerns                     |
| `tee`                        | Preserve a stream while passing it onward     | Useful for inspection and reproducibility                 |
| `fzf`                        | Interactive fuzzy selection and previews      | Primarily human/TUI-oriented                              |
| `man`, `--help`, `--version` | Discover command behavior                     | Useful to verify the installed interface                  |

`fzf` can combine file lists with live preview commands such as `bat`. This makes a powerful human exploration interface, although it is usually less appropriate for a noninteractive agent than deterministic search, selection, and file-read tools.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 12.2 Safe filename handling

The classic failure mode is treating newline-separated output as an unambiguous sequence of filenames.

For example, this is fragile:

```
find . -type f -name '*.rs' | xargs grep TODO
```

Filenames can contain spaces, tabs, and newlines, and `xargs` normally interprets its input according to whitespace and quoting rules.

The safer form is:

```
find . -type f -name '*.rs' -print0 |
  xargs -0 -r grep -n -- 'TODO'
```

The same approach applies to `rg --files -0`, `git ls-files -z`, and comparable NUL-delimited interfaces.

The `--` argument marker separates options from operands for supporting commands, preventing filenames beginning with a hyphen from being interpreted as flags.

GNU Findutils documents these details explicitly; `-print0` and `xargs -0` are also standardized by POSIX Issue 8 (2024).&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.gnu.org\&sz=32)

gnu.org



Source: [https://www.gnu.org/software/findutils/manual/html_node/find_html/Safe-File-Name-Handling.html](https://www.gnu.org/software/findutils/manual/html_node/find_html/Safe-File-Name-Handling.html?utm_source=chatgpt.com)

### 12.3 Why agent-native tools can be preferable

Dedicated file-reading and searching tools can expose more structured behavior than raw shell execution:

| Agent-native feature                 | Benefit over unstructured shell output           |
| ------------------------------------ | ------------------------------------------------ |
| Path-based `Read`                    | Known source identity and controlled file access |
| Offset/limit reads                   | Explicitly bounded content retrieval             |
| Glob search                          | Structured filename results                      |
| Grep search                          | Structured matches and locations                 |
| Symbol search                        | Direct navigation to source identities           |
| Structured diff/edit tools           | Explicit change boundaries                       |
| Built-in output truncation reporting | Better visibility into incomplete reads          |

However, a dedicated tool is not automatically more capable than the underlying CLI. Some wrappers expose only a subset of its options; some delegate to ripgrep internally; and others implement different ignore, regex, or truncation semantics.

For advanced workflows, the actual tool contract matters more than the name `Grep` or `Read`.

## 13. The important cross-cutting distinctions

A programming agent's choice of tool affects not only efficiency, but also what the returned result proves.

| Dimension           | Potential failure                                   | Better mechanism                                    |
| ------------------- | --------------------------------------------------- | --------------------------------------------------- |
| Completeness        | `rg` excludes ignored/hidden files                  | Explicit search scope and filtering controls        |
| Context truncation  | Shell output is cut off by the agent runtime        | Bounded reads, pagination, structured result limits |
| Source identity     | A result lacks a stable path or revision            | Path, commit hash, explicit source ranges           |
| Line accuracy       | Reformatted output changes displayed line positions | Unmodified source reads or structured offsets       |
| Symbol identity     | Same name refers to unrelated declarations          | LSP or semantic index                               |
| Format semantics    | Regex matches text that resembles structured data   | `jq`, `yq`, SQL, or format-specific parser          |
| Filesystem safety   | Filenames split incorrectly in pipes                | NUL-delimited records, argument vectors             |
| Freshness           | Search index does not reflect latest changes        | Live search or synchronized index                   |
| Read-only intent    | A replacement command changes files                 | Explicit preview mode and nonmutating commands      |
| Runtime portability | GNU-only flag missing on another system             | Verify installed implementation and options         |

A useful pattern is to distinguish three different outputs:

A discovery result identifies a candidate. An inspection result shows the relevant evidence. A semantic result establishes a relationship using a parser, index, or language analysis.

These are complementary rather than interchangeable.

## 14. What would constitute a comprehensive agent search toolkit?

For a capable terminal-based coding agent, I would separate the toolset into three tiers. This is a functional recommendation based on the capabilities above, not a claim that every agent needs every utility installed.

| Tier              | Tooling                                                                                                     | Role                                                                      |
| ----------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Core              | `rg`, `sed`, `cat`, `head`, `tail`, `find`, `ls`, `git`, `grep`, `awk`, `jq`, `file`, `stat`, `wc`, `xargs` | General discovery and exact text inspection                               |
| Enhanced          | `fd`, `bat`, `eza`, `yq`, `duckdb`, `rga`, `delta`, `difftastic`, `lnav`, `sd`, `fzf`                       | Better navigation, format coverage, structured querying, and presentation |
| Code intelligence | `ast-grep`, `tree-sitter`, Universal Ctags, LSP clients/servers, SCIP, Semgrep, Zoekt/Sourcegraph           | Structural and semantic navigation across larger codebases                |

One particularly valuable addition is a source-outline capability that operates between path discovery and full file reading. For example, it can expose a file's imports, declarations, methods, and significant source ranges before an agent chooses which portions to inspect.

This is a different improvement from simply replacing `cat` with a faster file viewer: it can reduce the amount of irrelevant source content returned to the model.

### A representative search-and-inspect sequence

```
# 1. Discover candidate files
rg --files -g '*.rs'

# 2. Locate relevant textual occurrences
rg -n 'parse_schema' src/

# 3. Inspect a selected source region
sed -n '120,200p' src/parser.rs

# 4. Inspect syntax-aware declarations
ast-grep outline src/parser.rs

# 5. Find changes introducing or removing the term
git log -S 'parse_schema' --oneline

# 6. Inspect historical content
git show HEAD~2:src/parser.rs |
  sed -n '120,200p'
```

A semantic language-server query can then distinguish the actual symbol's references from unrelated textual occurrences.

This demonstrates why agents commonly combine tools rather than relying exclusively on any one search engine.

## Conclusion

The traditional Unix toolkit remains highly useful for coding agents because it offers precise, composable operations with small interfaces.

But there are three broad levels of capability:

Text and filesystem retrieval

`rg`, `sed`, `awk`, `find`, `fd`, `jq` — find and inspect bytes, lines, paths, records, and values

Syntactic understanding

`ast-grep`, `tree-sitter`, Semgrep — find constructs, patterns, declarations, and source structure

Semantic and indexed understanding

LSP, SCIP, compiler analysis, Sourcegraph — navigate symbols, references, relationships, revisions, and indexed collections

The highest-value improvement is often not replacing the basic tools. It is giving agents a better way to move between discovery, targeted content inspection, syntax-aware summaries, and semantic navigation, while preserving precise source locations and avoiding unnecessary material in their context.