# Simple technical English for coding agents

Use this prompt when you write or review technical prose. It describes the project rules; it is not an official ASD-STE100 compliance document.

## Why this matters

Technical readers can be tired, rushed, or non-native English speakers. A vague sentence can cause a failed deployment, a lost backup, or an unsafe repair. Write so that a reader can understand and execute each sentence on the first read.

Apply these rules to documentation, READMEs, runbooks, procedures, error messages, release notes, incident reports, API guides, comments, and agent instructions. Do not apply them to marketing copy, brand writing, or quoted text that must remain exact.

## Before you write

Classify each passage:

- **Procedural text** tells the reader what to do. Use imperative sentences. Keep each sentence to 20 words or fewer. Write one instruction per sentence.
- **Descriptive text** explains a system or event. Use simple present, past, or future tense. Keep each sentence to 25 words or fewer. Keep one topic per paragraph and no more than six sentences per paragraph.

Use a heading such as `## Troubleshooting` or an inline directive when the passage is procedural. The linter also supports explicit directives:

```markdown
<!-- englishlint: procedural -->
<!-- englishlint: descriptive -->
```

Do not mix a command and its explanation in one sentence when separate sentences are clearer.

## Writing rules

- Use active voice. Name the actor: “The service retries the request,” not “The request is retried.”
- Use simple tenses. Avoid present perfect and complex verb constructions: “We updated the file,” not “We have updated the file.”
- Do not use verb forms ending in `-ing` as clauses: “The service retries the request. This prevents a duplicate,” not “..., preventing a duplicate.”
- Use only `can`, `will`, and `must` as modal verbs. Replace `should` with `must` when it is a requirement. Replace `may`, `might`, and `could` with `can` when they describe a possibility. Rewrite hypothetical `would` sentences as direct conditions.
- Put a condition before its command: “If the build fails, read the log,” not “Read the log if the build fails.”
- Use complete grammar. Keep articles and the word `that`. Do not use contractions.
- Do not use semicolons. Write two sentences.
- Replace Latin abbreviations: use “for example” instead of `e.g.`, “that is” instead of `i.e.`, and name items instead of `etc.`.
- Remove filler and vague claims such as “simply,” “seamlessly,” “robust,” “powerful,” “comprehensive,” and “leverage.” State the measurable fact or delete the word.
- Use one term for one concept throughout a document. Do not rotate between `check`, `verify`, `confirm`, `validate`, and `ensure`. Do not rotate between `config`, `configuration`, and `settings`.
- Keep technical noun chains short. Break a chain with `of`, `for`, `in`, or another preposition.
- Use American English spelling unless the project requires another spelling.
- Put a warning or condition before the risk: “CAUTION: Do not use `--force` in production. The command deletes unmatched rows.”
- Use vertical lists for multiple steps or items.

## Preserve exact technical text

Never change code blocks, inline code, identifiers, commands, flags, file paths, URLs, product names, API names, configuration keys, quoted errors, or log lines. Preserve their spelling and punctuation. Explain an exact technical value in prose instead of rewriting it.

## Examples

### Configuration documentation

Bad:

> Leveraging the robust configuration architecture, users can seamlessly configure the service, making it easy to get started. You should ensure that the settings have been properly configured before running the command.

Good:

> The service uses one configuration file.
>
> Before you run the command, make sure that the configuration file is correct. Run the command after you correct the file.

### Troubleshooting

Bad:

> If the upload fails, check the credentials and retry the request if the issue persists.

Good:

> If the upload fails, make sure that the credentials are correct. Retry the request if the upload fails again.

Better when the second condition is important:

> If the upload fails, make sure that the credentials are correct. If the upload fails again, read the error log.

### Incident report

Bad:

> We have identified an issue that may have impacted some users, and our team is working diligently to resolve it.

Good:

> From 14:02 to 14:31 UTC, 12% of API requests failed with HTTP 502. A deploy at 14:00 removed the cache warmup step. We reverted the deploy at 14:27.

### Error message

Bad:

> Oops! Something went wrong while attempting to establish a connection. Please ensure that your credentials are properly configured and try again.

Good:

> The database rejected the connection because the password was incorrect. Set `DB_PASSWORD` to the correct value, then connect again.

## Run the linter

From the project root, run:

```text
cargo run -- <directory>
```

For an installed binary:

```text
englishlint <directory>
```

The linter walks Markdown files and respects `.gitignore`. It checks visible prose and ignores fenced code, inline code, URLs, Markdown link destinations, HTML comments, and front matter. It reports stable rule IDs, file names, line numbers, columns, and a suggested fix.

Example:

```text
docs/setup.md:18:1: ENG001 procedural sentence has 24 words; maximum is 20
  suggestion: Split the sentence into two or more sentences.
docs/setup.md:24:17: ENG003 avoid modal 'should'; use 'must' when that is the intended meaning
  suggestion: Replace 'should' with 'must', or state the condition directly.
```

Exit codes are deterministic:

- `0`: no findings;
- `1`: one or more lint findings;
- `2`: invalid arguments, configuration errors, or file errors.

Use a project configuration file at `englishlint.ini`, or select one with `-c FILE.ini`:

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

The glossary makes terminology-rotation findings actionable. If the linter reports `ENG010` without a glossary preference, add the preferred term to `[glossary]`.

Use inline exceptions only when the text is intentionally correct:

```markdown
<!-- englishlint: ignore-file ENG006 -->
<!-- englishlint: ignore-next-line ENG001 ENG003 -->
<!-- englishlint: ignore-line ENG006 -->
<!-- englishlint: ignore-word widget -->
```

Do not suppress a finding merely to make CI pass. Fix the prose when the finding is valid. Review heuristic findings, especially passive voice and noun-chain findings, because deterministic text checks cannot understand every grammatical context.

## Agent workflow

1. Classify the passage.
2. Preserve all exact technical text.
3. Choose one term for each repeated concept.
4. Write short, complete sentences.
5. Put conditions before commands.
6. Run `englishlint <directory>`.
7. Fix each valid finding manually.
8. Run the linter again and review the diff.

The linter gives advice. It does not rewrite files and it does not certify ASD-STE100 compliance.
