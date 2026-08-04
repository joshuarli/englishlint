use crate::analysis::{occurrences, tokenize, Analysis};
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;
use crate::source::Span;
use crate::text::TextMode;

pub fn lint_document(document: &Document, config: &Config) -> Vec<Diagnostic> {
    let analysis = Analysis::build(document);
    let mut diagnostics = Vec::new();
    check_sentence_length(&analysis, document, config, &mut diagnostics);
    check_procedural(&analysis, document, config, &mut diagnostics);
    check_paragraphs(&analysis, document, config, &mut diagnostics);
    check_contractions(&analysis, document, config, &mut diagnostics);
    check_modals(&analysis, document, config, &mut diagnostics);
    check_semicolons(document, config, &mut diagnostics);
    check_latin(document, config, &mut diagnostics);
    check_filler(&analysis, document, config, &mut diagnostics);
    check_tenses(&analysis, document, config, &mut diagnostics);
    check_ing(&analysis, document, config, &mut diagnostics);
    check_passive(&analysis, document, config, &mut diagnostics);
    check_spelling(document, config, &mut diagnostics);
    check_noun_chains(&analysis, document, config, &mut diagnostics);
    check_terminology(&analysis, document, config, &mut diagnostics);
    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.rule));
    diagnostics
}

fn push(
    out: &mut Vec<Diagnostic>,
    document: &Document,
    config: &Config,
    rule: RuleId,
    span: Span,
    message: impl Into<String>,
    suggestion: impl Into<String>,
) {
    let line = document.source.location(span.start).line;
    if !document.directives.suppresses(rule, line, config) {
        out.push(Diagnostic::new(rule, span, message, suggestion));
    }
}

fn check_sentence_length(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for sentence in &analysis.sentences {
        let limit = match sentence.mode {
            TextMode::Descriptive => config.descriptive_limit,
            TextMode::Procedural => config.procedural_limit,
        };
        if sentence.words.len() > limit {
            push(
                out,
                document,
                config,
                RuleId::SentenceLength,
                sentence.span,
                format!(
                    "{} sentence has {} words; maximum is {} for {} text: '{}'",
                    sentence.mode.name(),
                    sentence.words.len(),
                    limit,
                    sentence.mode.name(),
                    quote(document.source.text_for(sentence.span))
                ),
                "Split the sentence into two or more sentences.",
            );
        }
    }
}

fn check_procedural(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    let actions = [
        "run",
        "set",
        "open",
        "close",
        "make",
        "check",
        "verify",
        "confirm",
        "validate",
        "ensure",
        "install",
        "configure",
        "create",
        "delete",
        "remove",
        "read",
        "write",
        "add",
        "enable",
        "disable",
        "copy",
        "update",
        "select",
        "enter",
        "use",
        "press",
        "replace",
        "save",
        "increase",
        "decrease",
    ];
    for sentence in analysis
        .sentences
        .iter()
        .filter(|sentence| sentence.mode == TextMode::Procedural)
    {
        let text = document.source.text_for(sentence.span);
        for condition in ["if", "when"] {
            let found = occurrences(text, condition, sentence.span.start.0);
            if found.len() > 1
                || found
                    .first()
                    .is_some_and(|span| span.start.0 > sentence.span.start.0 + first_word_len(text))
            {
                let span = found.last().copied().unwrap_or(sentence.span);
                push(
                    out,
                    document,
                    config,
                    RuleId::TrailingCondition,
                    span,
                    format!(
                        "put the '{}' condition before the command: '{}'",
                        condition,
                        quote(text)
                    ),
                    format!(
                        "Start the sentence with '{}', then write the command.",
                        condition
                    ),
                );
            }
        }
        let count = sentence
            .words
            .iter()
            .filter(|word| actions.contains(&word.text.to_ascii_lowercase().as_str()))
            .count();
        if count > 1 && (text.contains(" and ") || text.contains(',')) {
            push(
                out,
                document,
                config,
                RuleId::MultipleInstructions,
                sentence.span,
                format!("sentence contains multiple instructions: '{}'", quote(text)),
                "Write one instruction per sentence.",
            );
        }
    }
}

fn first_word_len(text: &str) -> usize {
    text.char_indices()
        .find(|(_, character)| character.is_whitespace())
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn check_paragraphs(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for paragraph in &analysis.paragraphs {
        if paragraph.sentences.len() > 6 {
            push(
                out,
                document,
                config,
                RuleId::ParagraphLength,
                paragraph.span,
                format!(
                    "paragraph has {} sentences; maximum is 6",
                    paragraph.sentences.len()
                ),
                "Split the paragraph around its topics.",
            );
        }
    }
}

fn check_contractions(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    let special = [
        "it's", "that's", "what's", "who's", "there's", "here's", "let's",
    ];
    for word in &analysis.words {
        let lower = word.text.to_ascii_lowercase();
        if (lower.contains('\'') && !lower.ends_with("'s")) || special.contains(&lower.as_str()) {
            push(
                out,
                document,
                config,
                RuleId::Contraction,
                word.span,
                format!("contraction '{}' is not allowed", word.text),
                "Write the complete form.",
            );
        }
    }
}

fn check_modals(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for (bad, replacement) in [
        ("should", "must"),
        ("would", "rewrite the hypothetical condition"),
        ("may", "can"),
        ("might", "can"),
        ("could", "can"),
    ] {
        for sentence in &analysis.sentences {
            let text = document.source.text_for(sentence.span);
            for span in occurrences(text, bad, sentence.span.start.0) {
                let shown = document.source.text_for(span);
                push(
                    out,
                    document,
                    config,
                    RuleId::BannedModal,
                    span,
                    format!(
                        "avoid modal '{}'; use '{}' when that is the intended meaning",
                        shown, replacement
                    ),
                    format!(
                        "Replace '{}' with '{}', or state the condition directly.",
                        shown, replacement
                    ),
                );
            }
        }
    }
}

fn check_semicolons(document: &Document, config: &Config, out: &mut Vec<Diagnostic>) {
    for prose in &document.prose {
        for (offset, character) in document.source.text_for(prose.span).char_indices() {
            if character == ';' {
                let span = Span::new(prose.span.start.0 + offset, prose.span.start.0 + offset + 1);
                push(
                    out,
                    document,
                    config,
                    RuleId::Semicolon,
                    span,
                    "semicolon joins two sentences",
                    "Replace the semicolon with a period.",
                );
            }
        }
    }
}

fn check_latin(document: &Document, config: &Config, out: &mut Vec<Diagnostic>) {
    for (term, replacement) in [
        ("e.g.", "for example"),
        ("i.e.", "that is"),
        ("etc.", "name the remaining items"),
    ] {
        for span in document.occurrences(term) {
            push(
                out,
                document,
                config,
                RuleId::LatinAbbreviation,
                span,
                format!("avoid Latin abbreviation '{}'", term),
                format!("Use '{}'.", replacement),
            );
        }
    }
}

fn check_filler(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    let terms = [
        ("simply", "delete it or state the fact"),
        ("just", "delete it or state the fact"),
        ("seamlessly", "delete it or describe the behavior"),
        ("effortlessly", "delete it or describe the behavior"),
        ("robust", "give the measurable property"),
        ("powerful", "give the measurable property"),
        ("comprehensive", "name what it includes"),
        ("leverage", "use"),
        ("utilize", "use"),
        ("in order to", "to"),
        ("prior to", "before"),
        ("facilitate", "help"),
        ("functionality", "function or feature"),
        ("streamline", "make simpler"),
        ("performant", "give a measurement"),
        ("crucial", "state the fact"),
        ("pivotal", "state the fact"),
        ("delve", "examine"),
        ("myriad", "many"),
        ("plethora", "many"),
        ("etc", "name the items"),
        ("ensure", "make sure that"),
        ("out of the box", "by default"),
    ];
    for word in &analysis.words {
        let lower = word.text.to_ascii_lowercase();
        for (term, replacement) in terms {
            if lower == term && !config.ignored_words.contains(term) {
                push(
                    out,
                    document,
                    config,
                    RuleId::FillerWord,
                    word.span,
                    format!("replace filler or vague word '{}'", word.text),
                    format!("Use '{}'.", replacement),
                );
            }
        }
    }
}

fn check_tenses(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for phrase in ["has been", "have been", "had been"] {
        for sentence in &analysis.sentences {
            for span in occurrences(
                document.source.text_for(sentence.span),
                phrase,
                sentence.span.start.0,
            ) {
                push(
                    out,
                    document,
                    config,
                    RuleId::ComplexTense,
                    span,
                    format!("complex or perfect tense starts with '{}'", phrase),
                    "Use a simple tense.",
                );
            }
        }
    }
}

fn check_ing(analysis: &Analysis, document: &Document, config: &Config, out: &mut Vec<Diagnostic>) {
    for sentence in &analysis.sentences {
        let text = document.source.text_for(sentence.span);
        for (offset, character) in text.char_indices() {
            if character == ',' {
                let tail = text[offset + 1..].trim_start();
                if let Some(word) = tail.split_whitespace().next() {
                    if word.ends_with("ing") && word != "including" {
                        let span = Span::new(
                            sentence.span.start.0 + offset,
                            sentence.span.start.0 + offset + 1,
                        );
                        push(
                            out,
                            document,
                            config,
                            RuleId::IngClause,
                            span,
                            format!("dangling '-ing' clause '{}...'", quote(tail)),
                            "Write the action as a separate sentence.",
                        );
                    }
                }
            }
        }
    }
}

fn check_passive(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for phrase in ["is ", "are ", "was ", "were ", "be ", "been "] {
        for sentence in &analysis.sentences {
            for span in occurrences(
                document.source.text_for(sentence.span),
                phrase,
                sentence.span.start.0,
            ) {
                let tail = &document.source.text[span.end.0..sentence.span.end.0];
                if let Some(word) = tokenize(tail, span.end.0).first() {
                    if word.text.ends_with("ed") {
                        let construction = document
                            .source
                            .text_for(Span::new(span.start.0, word.span.end.0));
                        push(
                            out,
                            document,
                            config,
                            RuleId::PassiveVoice,
                            span,
                            format!("likely passive construction '{}'", construction),
                            "Name the actor and use an active verb.",
                        );
                    }
                }
            }
        }
    }
}

fn check_spelling(document: &Document, config: &Config, out: &mut Vec<Diagnostic>) {
    for (term, replacement) in [
        ("colour", "color"),
        ("behaviour", "behavior"),
        ("organisation", "organization"),
        ("optimise", "optimize"),
        ("initialise", "initialize"),
        ("centre", "center"),
        ("analyse", "analyze"),
        ("licence", "license"),
    ] {
        for span in document.occurrences(term) {
            push(
                out,
                document,
                config,
                RuleId::AmericanSpelling,
                span,
                format!("use American spelling '{}'", term),
                format!("Use '{}'.", replacement),
            );
        }
    }
}

fn check_noun_chains(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    let stop = [
        "a", "an", "the", "of", "for", "on", "in", "to", "with", "and", "or", "that", "is", "are",
        "can", "will", "must", "from", "by", "as", "at",
    ];
    for window in analysis.words.windows(5) {
        let between = &document.source.text[window[0].span.end.0..window[4].span.start.0];
        if ["a", "an", "the"].contains(&window[0].text.to_ascii_lowercase().as_str())
            && !between.contains(['.', '!', '?', ':', ',', '`', '[', ']'])
            && window[1..]
                .iter()
                .all(|word| !stop.contains(&word.text.to_ascii_lowercase().as_str()))
        {
            let span = Span::new(window[1].span.start.0, window[4].span.end.0);
            push(
                out,
                document,
                config,
                RuleId::LongNounChain,
                span,
                format!(
                    "technical noun chain has at least four words: '{}'",
                    quote(
                        document
                            .source
                            .text_for(Span::new(window[0].span.start.0, window[4].span.end.0))
                    )
                ),
                "Break the noun chain with a preposition such as 'of' or 'for'.",
            );
            break;
        }
    }
}

fn check_terminology(
    _analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for (group, terms) in [
        (
            "check",
            &["check", "verify", "confirm", "validate", "ensure"] as &[&str],
        ),
        (
            "config",
            &["config", "configuration", "settings", "options"],
        ),
        ("delete", &["delete", "remove", "drop", "destroy"]),
    ] {
        let mut occurrences_found = Vec::new();
        for term in terms {
            for span in document.occurrences(term) {
                occurrences_found.push((span, *term));
            }
        }
        occurrences_found.sort_by_key(|(span, _)| span.start);
        let distinct: std::collections::HashSet<&str> =
            occurrences_found.iter().map(|(_, term)| *term).collect();
        if distinct.len() > 1 {
            let preferred = config
                .glossary
                .get(group)
                .map(String::as_str)
                .unwrap_or(occurrences_found[0].1);
            for (span, term) in occurrences_found {
                if term != preferred {
                    let suggestion = if config.glossary.contains_key(group) {
                        format!("Use '{}' for this concept.", preferred)
                    } else {
                        format!("Choose one term; set [glossary] {} = <preferred term> in englishlint.ini.", group)
                    };
                    push(
                        out,
                        document,
                        config,
                        RuleId::TerminologyRotation,
                        span,
                        format!(
                            "terminology rotation: '{}' and '{}' name the same concept",
                            term, preferred
                        ),
                        suggestion,
                    );
                }
            }
        }
    }
}

fn quote(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > 100 {
        format!("{}…", compact.chars().take(97).collect::<String>())
    } else {
        compact
    }
}
