use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_aet(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_aet"))
        .current_dir(workspace_root())
        .args(args)
        .output()
        .expect("aet command should run");

    assert!(
        output.status.success(),
        "aet {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_round_trip_creates_outputs_for_both_phase1_fixtures() {
    let root = workspace_root();
    for article in ["vietnam-two-child-policy", "dengue-bangladesh"] {
        let article_path = format!("data/articles/{article}");
        run_aet(&["validate", &article_path]);
        run_aet(&["build", &article_path]);

        let out_dir = root.join("dist").join(article);
        for file in [
            "vocabulary.pdf",
            "anki-basic.tsv",
            "anki-cloze.tsv",
            "anki-production.tsv",
        ] {
            assert!(
                out_dir.join(file).is_file(),
                "expected {}",
                out_dir.join(file).display()
            );
        }
    }
}
