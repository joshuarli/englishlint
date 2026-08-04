use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after Unix epoch")
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("englishlint-test-{timestamp}-{id}"));
        fs::create_dir_all(&path).expect("create temporary project");
        Self { path }
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, contents).expect("write fixture");
    }

    fn run(&self, args: &[&str]) -> Output {
        let binary = env!("CARGO_BIN_EXE_englishlint");
        let output = Command::new(binary)
            .args(args)
            .current_dir(&self.path)
            .output()
            .expect("run englishlint");
        Output {
            status: output.status.code().expect("process has an exit code"),
            stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8"),
            stderr: String::from_utf8(output.stderr).expect("stderr is UTF-8"),
        }
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct Output {
    status: i32,
    stdout: String,
    stderr: String,
}

fn assert_has(output: &Output, needle: &str) {
    assert!(
        output.stdout.contains(needle),
        "expected stdout to contain {needle:?}\nstdout:\n{}\nstderr:\n{}",
        output.stdout,
        output.stderr
    );
}

fn assert_not_has(output: &Output, needle: &str) {
    assert!(
        !output.stdout.contains(needle),
        "expected stdout not to contain {needle:?}\nstdout:\n{}",
        output.stdout
    );
}

#[test]
fn clean_project_returns_zero_and_does_not_lint_code_or_metadata() {
    let project = TempProject::new();
    project.write(
        "README.md",
        "---\ntitle: should be ignored\n---\n\n# Install\n\nThe service uses a file.\n\n```text\nshould simply use robust settings; it's code\n```\n\n[manual](https://example.com/should)\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 0, "{:#?}", output.stdout);
    assert_has(&output, "englishlint: no findings");
    assert_not_has(&output, "ENG003");
    assert_not_has(&output, "ENG004");
}

#[test]
fn reports_rules_with_relative_file_line_column_and_suggestion() {
    let project = TempProject::new();
    project.write(
        "docs/setup.md",
        "# Install\n\nThe service should use the file.\n",
    );

    let output = project.run(&["docs"]);

    assert_eq!(output.status, 1);
    assert_has(&output, "setup.md:3:13: ENG003");
    assert_has(&output, "suggestion: Replace 'should' with 'must'");
}

#[test]
fn custom_ini_changes_modes_limits_headings_and_ignored_rules() {
    let project = TempProject::new();
    project.write(
        "englishlint.ini",
        "[lint]\ndefault_type = procedural\nprocedural_limit = 3\ndescriptive_limit = 2\nprocedural_headings = how-to\nignore_rules = ENG003\n",
    );
    project.write(
        "README.md",
        "A short descriptive sentence.\n\n## How-to\n\nInstall the new service now.\n\nThe service should work.\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_has(&output, "ENG001");
    assert_not_has(&output, "ENG003");
    assert_has(&output, "maximum is 3 for procedural text");
}

#[test]
fn explicit_directives_select_mode_and_support_inline_exceptions() {
    let project = TempProject::new();
    project.write(
        "README.md",
        "<!-- englishlint: procedural -->\n<!-- englishlint: ignore-next-line ENG001 ENG003 -->\nIf the service fails, the operator should read the very long log file now and restart the service.\n<!-- englishlint: ignore-word widget -->\nThe widget is simply useful.\n<!-- englishlint: ignore-file ENG004 -->\nThe service uses a file; it starts quickly.\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_not_has(&output, "ENG001");
    assert_not_has(&output, "ENG003");
    assert_not_has(&output, "ENG004");
    assert_has(&output, "ENG006");
}

#[test]
fn glossary_preference_makes_terminology_rotation_actionable() {
    let project = TempProject::new();
    project.write(
        "englishlint.ini",
        "[glossary]\ncheck = verify\nconfig = configuration\ndelete = remove\n",
    );
    project.write(
        "README.md",
        "Verify the account. Check the account.\nUse the config and configuration file.\nDelete the file and remove the file.\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_has(&output, "ENG010");
    assert_has(&output, "Use 'verify' for this concept.");
    assert_has(&output, "Use 'configuration' for this concept.");
    assert_has(&output, "Use 'remove' for this concept.");
    assert_not_has(&output, "set [glossary]");
}

#[test]
fn missing_glossary_preference_suggests_configuration() {
    let project = TempProject::new();
    project.write("README.md", "Check the account. Verify the account.\n");

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_has(&output, "ENG010");
    assert_has(
        &output,
        "set [glossary] check = <preferred term> in englishlint.ini",
    );
}

#[test]
fn config_can_ignore_words_and_rules_from_both_supported_sections() {
    let project = TempProject::new();
    project.write(
        "custom.ini",
        "[lint]\nignore_words = simply\n[ignore]\nrules = ENG003\nwords = robust\n",
    );
    project.write(
        "README.md",
        "The service should run. The robust service runs. The service simply runs.\n",
    );

    let output = project.run(&["--config", "custom.ini", "."]);

    assert_eq!(output.status, 0, "{:#?}", output.stdout);
    assert_has(&output, "englishlint: no findings");
}

#[test]
fn gitignore_excludes_markdown_files_and_walks_nested_files() {
    let project = TempProject::new();
    project.write(".gitignore", "ignored/\n*.generated.md\n");
    project.write("ignored/bad.md", "The service should fail.\n");
    project.write("ignored.generated.md", "The service should fail.\n");
    project.write("nested/good.md", "The service works.\n");

    let output = project.run(&["."]);

    assert_eq!(output.status, 0, "{:#?}", output.stdout);
    assert_has(&output, "englishlint: no findings");
    assert_not_has(&output, "ignored");
}

#[test]
fn invalid_config_returns_usage_error_code_two() {
    let project = TempProject::new();
    project.write("bad.ini", "[lint]\nprocedural_limit = not-a-number\n");
    project.write("README.md", "The service works.\n");

    let output = project.run(&["--config", "bad.ini", "."]);

    assert_eq!(output.status, 2);
    assert!(output.stderr.contains("invalid procedural_limit"));
}

#[test]
fn missing_directory_and_unknown_option_return_usage_error_code_two() {
    let project = TempProject::new();

    let missing = project.run(&["missing"]);
    assert_eq!(missing.status, 2);
    assert!(missing.stderr.contains("is not a directory"));

    let unknown = project.run(&["--unknown", "."]);
    assert_eq!(unknown.status, 2);
    assert!(unknown.stderr.contains("unknown option"));
}

#[test]
fn catches_its_contraction_but_does_not_treat_a_possessive_as_one() {
    let project = TempProject::new();
    project.write("README.md", "It's ready. The service's file is ready.\n");

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_has(&output, "ENG002 contraction 'It's'");
    assert_not_has(&output, "contraction 'service's'");
}

#[test]
fn every_catalog_rule_has_a_cli_regression_fixture() {
    let project = TempProject::new();
    project.write(
        "README.md",
        "This sentence contains more than twenty five words and exists only to exercise the sentence length rule in the deterministic linter fixture for this production regression test.\n\nIt's ready; e.g. simply use the robust configuration before the service has been updated, making the result clear if the request fails.\n\nThe connection pool timeout configuration value is documented here.\n\nOne. Two. Three. Four. Five. Six. Seven.\n\nThe file is updated. The colour is wrong.\n\n<!-- englishlint: procedural -->\nRun the command and check the file if the build fails. The service should run.\n\nCheck the account. Verify the account.\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    for rule in 1..=15 {
        let id = format!("ENG{rule:03}");
        assert_has(&output, &id);
    }
}

#[test]
fn markdown_table_pipes_do_not_join_cells_into_one_sentence() {
    let project = TempProject::new();
    project.write(
        "README.md",
        "| Rule | Description |\n| --- | --- |\n| ENG001 | The service should work. |\n",
    );

    let output = project.run(&["."]);

    assert_eq!(output.status, 1);
    assert_has(&output, "ENG003");
    assert_not_has(&output, "ENG001");
}
