# Simple technical English for coding agents

Use this prompt when you write or review technical prose. It describes practical
rules inspired by ASD-STE100 Simplified Technical English. It is not an official
compliance document, and `englishlint` cannot certify compliance.

## Why this matters

Technical readers can be tired, rushed, or non-native English speakers. A vague
sentence can cause a failed deployment, a lost backup, or an unsafe repair.
Write so that a reader can understand and execute each sentence on the first
read.

Apply these rules to documentation, READMEs, runbooks, procedures, error
messages, release notes, incident reports, API guides, comments, and agent
instructions. Do not apply them mechanically to marketing copy, brand writing,
exact quotations, historical evidence, or machine-facing contract text.

## Classify before writing

Classify each passage. Do not silently apply procedural rules to descriptive text.

- **Procedural text** tells the reader what to do. Use imperative sentences.
  Keep each sentence to 20 words or fewer. Write one instruction per sentence.
- **Descriptive text** explains a system or event. Use simple present, past, or
  future tense. Keep each sentence to 25 words or fewer. Keep one topic per
  paragraph and no more than six sentences per paragraph.

Use a heading or an explicit directive when a passage is procedural:

```markdown
<!-- englishlint: procedural -->
<!-- englishlint: descriptive -->
```

Do not mix a command and its explanation in one sentence when separate
sentences are clearer.

## Core writing rules

### Sentences and structure

- Keep sentences short and complete.
- Use one instruction per procedural sentence.
- Put a condition before its command: “If the build fails, read the log,” not
  “Read the log if the build fails.”
- Use connecting words between related sentences, such as `Then`, `Because`,
  and `As a result`.
- Use a vertical list for multiple steps or items.
- Keep one topic per paragraph.
- Split a paragraph when it contains unrelated topics or more than six
  descriptive sentences.

### Verbs and voice

- Use active voice when it names the actor clearly: “The service retries the
  request,” not “The request is retried.”
- Preserve passive voice when it precisely describes an intentional state,
  unknown actor, or protocol transition. Review the context before changing it.
- Use simple present, past, or future tense.
- Avoid present perfect and complex constructions: “We updated the file,” not
  “We have updated the file.”
- Do not use `-ing` clauses when a separate sentence is clearer: “The service
  retries the request. This prevents a duplicate,” not “..., preventing a
  duplicate.”
- Use `can`, `will`, and `must` for technical guidance. Replace `should` with
  `must` when it states a requirement. Replace `may`, `might`, and `could` with
  `can` when they describe a capability or possibility. Rewrite hypothetical
  `would` sentences as direct conditions.

### Words and terminology

- Use complete grammar. Keep articles and the word `that`.
- Do not use contractions.
- Use one term for one concept throughout a document.
- Choose one term from each terminology group. Do not rotate between:
  - `check`, `confirm`, `verify`, `validate`, and `ensure`;
  - `config`, `configuration`, `settings`, and `options`;
  - `delete`, `remove`, `drop`, and `destroy`.
- Keep technical noun chains short. Break long chains with `of`, `for`, `in`,
  or another preposition.
- Use American English spelling unless the project requires another spelling.
- Replace Latin abbreviations: use `for example` instead of `e.g.`, `that is`
  instead of `i.e.`, and name items instead of `etc.`.
- Remove filler and vague claims such as `simply`, `just`, `seamlessly`,
  `effortlessly`, `robust`, `powerful`, `comprehensive`, `leverage`, and
  `utilize`. State the measurable fact or delete the word.

### Warnings and errors

Put the command or condition before the risk:

> CAUTION: Do not use `--force` in production. The command deletes unmatched rows.

For errors, use this order:

1. State what happened.
2. State the cause when it is known.
3. Give the fix as an imperative.

## Preserve exact technical text

Never change the content of:

- fenced code blocks;
- inline code;
- commands and flags;
- identifiers and configuration keys;
- file paths and URLs;
- product and API names;
- quoted errors and log lines;
- JSON, YAML, TOML, or other exact data;
- evaluator cases and exact-output requirements.

Explain an exact technical value in prose instead of rewriting the value. When a
contract or historical record contains bad English intentionally, preserve the
record and review the surrounding prose instead.

## Before-and-after examples

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

## Run englishlint

From the project root:

```text
cargo run -- <directory>
```

For an installed binary:

```text
englishlint <directory>
```

The linter walks Markdown files in deterministic path order and respects
`.gitignore`. It checks visible prose and ignores code, URLs, link destinations,
HTML, and front matter. It reports stable rule IDs, file names, one-based line
numbers, Unicode-character columns, and severity. It lists rule explanations
once in an aggregate summary.

Example:

```text
docs/setup.md:
  18:1: ENG001 [error]
  24:17: ENG003 [error]
englishlint: rule summary
  ENG001 [error] Sentence length: A sentence exceeds the configured word limit. Suggestion: Split the sentence into two or more sentences.
  ENG003 [error] Banned modal: The prose uses should, would, may, might, or could. Suggestion: Use must or can when that is the intended meaning.
```

Exit codes are deterministic:

- `0`: no findings;
- `1`: one or more findings;
- `2`: invalid arguments, configuration errors, or file errors.

## Configuration and profiles

The default configuration file is `englishlint.ini` in the current working
directory. Select another file with `--config`:

```text
cargo run -- --config path/to/englishlint.ini docs/
```

A configuration can define path-specific profiles:

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

[profile.guidance]
paths = README.md, docs/**, roles/**
ignore_rules = ENG011, ENG014
severity = ENG001:error, ENG012:warning

[profile.contracts]
paths = tickets/**, evals/**, templates/**
ignore_rules = ENG001, ENG009, ENG011, ENG012, ENG013, ENG014
severity = ENG003:warning
```

Profiles are useful when a project contains both maintained guidance and exact
contracts. A profile can use `paths` or `include`, `ignore_rules`,
`enable_rules` or `rules`, `ignore_words`, and `severity`. Severity accepts
`error`, `warning`, or `info`. A trailing `/**` matches descendants.

You can select a named profile in a Markdown file:

```markdown
<!-- englishlint: profile contracts -->
```

Use inline exceptions only when the text is intentionally correct:

```markdown
<!-- englishlint: ignore-file ENG006 -->
<!-- englishlint: ignore-next-line ENG001 ENG003 -->
<!-- englishlint: ignore-line ENG006 -->
<!-- englishlint: ignore-word widget -->
```

Do not add a suppression merely to make a command exit zero. Explain broad
exceptions in the project policy.

## Agent workflow

1. Read the relevant repository instructions and ownership guidance.
2. Classify the passage.
3. Preserve exact technical text.
4. Choose one term for each repeated concept.
5. Write short, complete sentences.
6. Put conditions before commands.
7. Run the repository's narrowest relevant verification command.
8. Run `englishlint <directory>`.
9. Fix valid findings manually.
10. Run the linter again and review the diff.

The linter gives advice. It does not rewrite files and it does not certify
ASD-STE100 compliance.

## Factory-specific note

When working in a repository with contracts, templates, evals, historical
records, or active agents, use a project-specific policy. Read the repository's
agent guide first. Do not perform broad prose cleanup during an unrelated task.
Preserve exact contract content and treat heuristic findings as review items.
See `FACTORY-PROMPT.md` for the full XSH Factory policy.
