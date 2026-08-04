# Simple English

`englishlint` is a deterministic Rust linter for clear technical English in Markdown.
It reports precise advice with file names, line numbers, columns, stable rule IDs, and
suggested fixes. It does not rewrite files and it is not an official ASD-STE100
compliance checker.

The writing guidance for coding agents lives in [`PROMPT.md`](PROMPT.md). The rule
catalog is in [`src/rules.rs`](src/rules.rs), and automated behavior tests are in
[`tests/integration.rs`](tests/integration.rs).

## Build and run

The project uses the pinned toolchain in `rust-toolchain.toml` and has one runtime
dependency: the standard Rust `ignore` crate for fast recursive traversal and
`.gitignore` support.

```bash
cargo build --release
./target/release/englishlint .
./target/release/englishlint docs/
./target/release/englishlint --config config/englishlint.ini docs/
```

The library also exposes `lint_text` for editors and other Rust callers. It returns
structured diagnostics with source spans; the CLI is only one renderer.

The command walks `.md` files in deterministic path order, respects `.gitignore`,
and follows no symlinks. It lints visible prose only. It ignores fenced code, inline
code, URLs, link destinations, HTML comments, and YAML front matter. Markdown headings
are not counted as sentences.

Files larger than 10 MiB are rejected by default. Set `max_file_bytes` in `[lint]` to
change this limit. The parser preserves the original UTF-8 source. Diagnostics use
one-based lines and Unicode-character columns.

Exit codes:

- `0`: no findings;
- `1`: one or more lint findings;
- `2`: invalid arguments, configuration errors, or file errors.

Example output:

```text
docs/setup.md:18:1: ENG001 procedural sentence has 24 words; maximum is 20 for procedural text
  suggestion: Split the sentence into two or more sentences.
docs/setup.md:24:17: ENG003 avoid modal 'should'; use 'must' when that is the intended meaning
  suggestion: Replace 'should' with 'must', or state the condition directly.
```

## Configuration

The default configuration file is `englishlint.ini` in the current working directory.
Use `-c` or `--config` to select another file. See
[`englishlint.ini.example`](englishlint.ini.example) for all supported settings.

```ini
[lint]
default_type = descriptive
procedural_limit = 20
descriptive_limit = 25
ignore_rules = ENG014
ignore_words = widget

[glossary]
check = make sure
config = configuration
```

Descriptive text has a 25-word sentence limit. Procedural text has a 20-word limit.
The linter uses procedural-looking headings such as `Install`, `Configure`, and
`Troubleshooting` as hints. Use an explicit directive when the classification matters:

```markdown
<!-- englishlint: procedural -->
<!-- englishlint: descriptive -->
```

Inline suppressions are available for intentional exceptions:

```markdown
<!-- englishlint: ignore-file ENG006 -->
<!-- englishlint: ignore-next-line ENG001 ENG003 -->
<!-- englishlint: ignore-line ENG006 -->
<!-- englishlint: ignore-word widget -->
```

Use suppressions sparingly. Heuristic findings, especially passive voice and long noun
chains, need human review.

Rule definitions and their stable IDs live in [`src/rules.rs`](src/rules.rs). Rules
provide deterministic advice, not a grammar proof or certification verdict.


## Agent instructions

Read [`PROMPT.md`](PROMPT.md) before writing technical prose. Preserve code, commands,
identifiers, paths, URLs, product names, configuration keys, quoted errors, and log
lines exactly. After writing, run `englishlint <directory>`, fix valid findings manually,
and run it again.

## License and status

MIT. This is an unofficial aid and is not affiliated with or endorsed by ASD or STEMG.
ASD-STE100 is a registered trademark of ASD.
