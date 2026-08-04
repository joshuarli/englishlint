# Englishlint policy for XSH Factory

Use this prompt when you review or write Markdown in `xsh-factory`. The factory
contains several document classes. Apply the correct policy to the file. Do not
rewrite factory prose during an unrelated product, ticket, eval, or controller
change.

## Purpose

Clear factory documentation reduces agent discovery, controller mistakes, and
ambiguity at process boundaries. The goal is precise operational guidance, not
uniform prose at any cost.

This is an unofficial practical aid inspired by ASD-STE100. It does not certify
compliance.

## Read the repository contract first

Before changing factory documentation, read the repository's `AGENTS.md` and
follow its scope, evidence, testing, and ownership rules. Preserve the working
tree. Do not modify files owned by another active agent unless the assignment
requires it.

The factory has two broad Markdown policies.

## Universal writing rules

Apply these rules to maintained prose unless the contract policy below makes a
specific exception:

- Classify text first. Procedures use imperative sentences of 20 words or
  fewer. Descriptions use simple tenses and sentences of 25 words or fewer.
- Write one instruction per sentence. Put conditions before commands.
- Use active voice when the actor is known. Preserve passive voice when it
  describes an intentional state, unknown actor, or protocol transition.
- Use simple present, past, or future tense. Avoid present perfect and complex
  verb constructions.
- Avoid dangling `-ing` clauses when a separate sentence is clearer.
- Use `can`, `will`, and `must` for technical guidance. Replace ambiguous
  `should`, `may`, `might`, and `could` with a precise requirement or capability.
- Use complete grammar. Keep articles and `that`. Do not use contractions.
- Do not use semicolons. Write two sentences.
- Use one term for one concept. Prefer `check` over `confirm`, `verify`,
  `validate`, and `ensure`. Prefer `configuration` over `config` and
  `settings`.
- Remove filler such as `simply`, `seamlessly`, `robust`, `powerful`, and
  `leverage`. State the fact or delete the word.
- Use American English spelling unless the repository requires another style.
- Replace `e.g.` with `for example`, `i.e.` with `that is`, and `etc.` with a
  named list.

Good:

> If the build fails, read the log. Then fix the reported error.

Avoid:

> Read the log if the build fails, and then you should try to fix the issue.

For errors, state what happened, state the cause when known, then give the fix:

> The database rejected the connection because the password was incorrect. Set
> `DB_PASSWORD` to the correct value, then connect again.

## Guidance policy
## Guidance policy

Apply this policy to:

- `README.md`;
- `AGENTS.md`;
- `NORTH-STAR.md`;
- `FACTORY.md`;
- `CTO.md`;
- `docs/**`;
- `roles/**`;
- `runtime/**`.

For guidance:

- Keep descriptive sentences to 25 words or fewer.
- Keep instructions to 20 words or fewer.
- Use one instruction per sentence.
- Put conditions before commands.
- Prefer active voice when it names the responsible actor clearly.
- Use `must` for requirements.
- Use `can` for capabilities or possibility.
- Avoid `should`, `would`, `may`, and `might` when they make a requirement or
  capability ambiguous.
- Do not use semicolons.
- Avoid contractions.
- Use one term for one concept.
- Prefer `check` over rotating between `check`, `confirm`, `verify`, and
  `validate`.
- Prefer `configuration` over rotating between `config`, `configuration`, and
  `settings`.
- State measurable facts instead of vague words such as `simply`, `robust`,
  `powerful`, or `comprehensive`.
- Keep one topic per paragraph. Split paragraphs that contain many unrelated
  sentences.

Review heuristic findings before changing them. The factory intentionally uses
passive state language such as `is admitted`, `is reviewed`, and `is shared`.
Do not rewrite that language only to satisfy a heuristic. Preserve it when it
accurately describes a state transition or ownership boundary.

## Contract and evidence policy

Apply this policy to:

- `cycle-*.md`;
- `tickets/**`;
- `evals/**`;
- `templates/**`.

These files can contain exact examples, evaluator cases, oracle descriptions,
historical observations, placeholders, and machine-facing protocols. Preserve
those details. Do not rewrite them for style unless the contract itself is the
change.

For contracts:

- Keep requirements and restrictions explicit.
- Preserve exact commands, paths, identifiers, shell snippets, XSH snippets,
  JSON, output text, and evaluator cases.
- Use complete grammar in prose around exact technical text.
- Treat style findings as review warnings, not automatic defects.
- Do not split a paragraph if the paragraph is an intentionally grouped
  contract or case list.
- Do not change a modal in an example or historical quotation without checking
  whether it changes the contract.

## Preserve exact technical text

Never change code blocks, inline code, commands, flags, identifiers, paths,
URLs, product names, API names, configuration keys, quoted errors, log lines,
JSON, YAML, evaluator cases, or exact-output requirements. Explain exact
technical values in surrounding prose instead.

## Run englishlint

Put the factory policy in `englishlint.ini` at the factory root. The executable
loads that file automatically when you run it from the factory root. If the
configuration lives elsewhere, pass it with `--config`:

```text
englishlint --config /path/to/englishlint.ini /path/to/xsh-factory
```

From the factory root, use:

```text
englishlint .
```

The executable does not modify files. Review its findings, edit Markdown
manually, and run `englishlint .` again.

The output has this form:

```text
AGENTS.md:31:56: ENG003 [error] avoid modal 'should'; use 'must' when that is the intended meaning
  suggestion: Replace 'should' with 'must', or state the condition directly.
CTO.md:78:1: ENG010 [warning] terminology rotation: 'confirm' and 'check' name the same concept
  suggestion: Choose one term; set [glossary] check = <preferred term> in englishlint.ini.
```

Exit codes:

- `0`: no findings;
- `1`: one or more findings;
- `2`: configuration, usage, or file error.

Severity is part of the output. `[error]` is a blocking candidate for the
selected profile. `[warning]` is a review candidate. `[info]` is informational.
Do not assume that every warning is a defect.

## Review order

Do not start by trying to reach zero findings. Use this order:

1. Read the file and its ownership context.
2. Review all `[error]` findings in maintained guidance.
3. Fix only findings that improve clarity without changing the contract.
4. Review `[warning]` findings as candidates.
5. Preserve exact technical content.
6. Run the nearest repository verification command.
7. Run englishlint again.
8. Report the remaining findings and explain intentional exceptions.

## What counts as a good fix

Good:

> Before the controller starts a cycle, read the latest `CTO-REPORT.md`.
> Use the report paths to inspect the relevant evidence.

Less useful:

> Before the controller starts a cycle, the latest report should perhaps be
> reviewed when it may contain relevant evidence.

Good:

> The controller owns cancellation. The worker reports qualitative findings.

Do not make this change if the source contract actually assigns both duties to
the controller. Technical accuracy comes before style.

## Suppressions

Use an inline suppression only when the finding is intentionally correct:

```markdown
<!-- englishlint: ignore-next-line ENG014 -->
The ticket is marked `Merged.` after reconciliation.
```

Use a file-level suppression only for a document with a clearly different
purpose. Explain the reason in the review or change description. Do not add
broad suppressions merely to make the command exit zero.

The profile already suppresses noisy rules for contracts. Do not add further
suppression without identifying the contract or false-positive reason.

## Do not edit active work

If another agent is changing the same factory document:

- do not reformat the file;
- do not perform a broad prose cleanup;
- do not split paragraphs across unrelated sections;
- do not change terminology globally;
- report findings with paths and line numbers instead.

Englishlint is a review instrument. It does not rewrite files.
