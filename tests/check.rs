use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn dataset(root: &Path, metadata: &str) {
    fs::create_dir_all(root.join("metadata")).unwrap();
    fs::create_dir_all(root.join("msdata/raw/nested")).unwrap();
    fs::write(root.join("metadata/metadata.tsv"), metadata).unwrap();
    fs::write(root.join("msdata/raw/a.mzXML"), "spectrum").unwrap();
    fs::write(root.join("msdata/raw/nested/b.mzML"), "spectrum").unwrap();
    fs::write(root.join("msdata/raw/nested/processed.mgf"), "not raw").unwrap();
}

fn check(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_machaca"))
        .arg("check")
        .arg(root)
        .args([
            "--metadata",
            "metadata/metadata.tsv",
            "--ms-dir",
            "msdata/raw",
        ])
        .output()
        .unwrap()
}

#[test]
fn counts_both_columns_and_matches_nested_files_without_including_processed_data() {
    let temp = tempfile::tempdir().unwrap();
    dataset(
        temp.path(),
        "sample\tpositive_filename\tnegative_filename\n1\ta.mzXML\t\n2\t\tb.mzML\n",
    );
    let output = check(temp.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("positive_filename: 1 nonempty filename entries"));
    assert!(stdout.contains("negative_filename: 1 nonempty filename entries"));
    assert!(stdout.contains("Mass-spectrometry files: 2"));
    assert!(stdout.contains("Validation passed"));
}

#[test]
fn rejects_duplicates_missing_references_and_unlisted_files() {
    let temp = tempfile::tempdir().unwrap();
    dataset(temp.path(), "filename\nmissing.mzXML\na.mzXML\na.mzXML\n");
    let output = check(temp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("duplicate reference \"a.mzXML\""),
        "{stderr}"
    );
    assert!(
        stderr.contains("metadata references missing file \"missing.mzXML\""),
        "{stderr}"
    );
    assert!(stderr.contains("file without metadata entry:"), "{stderr}");
}

#[test]
fn rejects_ambiguous_basenames_and_invalid_entries() {
    let temp = tempfile::tempdir().unwrap();
    dataset(temp.path(), "filename\na.mzXML\nb.mzML\nwrong.txt\n");
    fs::write(temp.path().join("msdata/raw/nested/a.mzXML"), "other").unwrap();
    let output = check(temp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("ambiguous filename \"a.mzXML\""),
        "{stderr}"
    );
    assert!(
        stderr.contains("invalid spectrum filename \"wrong.txt\""),
        "{stderr}"
    );
}

fn massive_dataset(root: &Path, filename: &str) {
    fs::create_dir_all(root.join("metadata/metadata")).unwrap();
    fs::create_dir_all(root.join("peak/peak")).unwrap();
    fs::write(root.join("peak/peak/sample_positive.mzML"), "spectrum").unwrap();
    fs::write(root.join("peak/peak/sample_negative.mzML"), "spectrum").unwrap();
    fs::write(
        root.join("metadata/metadata/dbgi_massive_metadata_20260806.tsv"),
        format!(
            "filename\tfile_path\tdirectus_filename\ttaxon_scientific_name\tionization_mode\n\
             {filename}\tdbgi_msdata/sample_positive.mzML\tsample_positive\tSapayoa aenigma\tpositive\n\
             sample_negative.mzML\tdbgi_msdata/sample_negative.mzML\tsample_negative\tSapayoa aenigma\tnegative\n"
        ),
    )
    .unwrap();
}

#[test]
fn massive_filename_column_ignores_path_and_stem_metadata() {
    let temp = tempfile::tempdir().unwrap();
    massive_dataset(temp.path(), "sample_positive.mzML");
    let metadata = temp
        .path()
        .join("metadata/metadata/dbgi_massive_metadata_20260806.tsv");
    let checked = machaca::check(&metadata, &temp.path().join("peak/peak")).unwrap();
    assert_eq!(checked.filename_columns, vec![0]);
    assert_eq!(checked.files.len(), 2);
    assert!(checked.files.contains_key("sample_positive.mzML"));
    assert!(checked.files.contains_key("sample_negative.mzML"));

    let output = Command::new(env!("CARGO_BIN_EXE_machaca"))
        .arg("check")
        .arg(temp.path())
        .args([
            "--metadata",
            "metadata/metadata/dbgi_massive_metadata_20260806.tsv",
            "--ms-dir",
            "peak/peak",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("filename: 2 nonempty filename entries"));
    assert!(!stdout.contains("file_path:"));
    assert!(!stdout.contains("directus_filename:"));
    assert!(stdout.contains("Validation passed"));
}

#[test]
fn massive_filename_mismatch_is_not_masked_by_matching_path_metadata() {
    let temp = tempfile::tempdir().unwrap();
    massive_dataset(temp.path(), "missing.mzML");
    let metadata = temp
        .path()
        .join("metadata/metadata/dbgi_massive_metadata_20260806.tsv");
    let error = machaca::check(&metadata, &temp.path().join("peak/peak"))
        .err()
        .expect("mismatched filename must fail");
    assert!(error.contains("metadata references missing file \"missing.mzML\""));
    assert!(error.contains("file without metadata entry:"));
}

#[test]
fn massive_invalid_filename_fails_even_with_valid_path_metadata() {
    let temp = tempfile::tempdir().unwrap();
    massive_dataset(temp.path(), "sample_positive.txt");
    let metadata = temp
        .path()
        .join("metadata/metadata/dbgi_massive_metadata_20260806.tsv");
    let error = machaca::check(&metadata, &temp.path().join("peak/peak"))
        .err()
        .expect("invalid filename must fail");
    assert!(error.contains("column filename: invalid spectrum filename \"sample_positive.txt\""));
}

#[test]
fn first_rows_ignore_later_metadata_and_unselected_spectra() {
    let temp = tempfile::tempdir().unwrap();
    dataset(
        temp.path(),
        "filename\na.mzXML\nwrong.txt\nb.mzML\n",
    );
    fs::write(temp.path().join("msdata/raw/nested/b.mzML"), "other").unwrap();
    fs::write(temp.path().join("msdata/raw/b.mzML"), "ambiguous").unwrap();
    let metadata = temp.path().join("metadata/metadata.tsv");
    let checked = machaca::check_first(&metadata, &temp.path().join("msdata/raw"), 1).unwrap();
    assert_eq!(checked.filename_columns, vec![0]);
    assert_eq!(checked.files.len(), 1);
    assert_eq!(
        checked.files["a.mzXML"],
        temp.path().join("msdata/raw/a.mzXML")
    );
    assert!(machaca::check(&metadata, &temp.path().join("msdata/raw")).is_err());
}

#[test]
fn first_rows_reject_missing_and_ambiguous_selected_files() {
    let temp = tempfile::tempdir().unwrap();
    dataset(temp.path(), "filename\nmissing.mzML\nb.mzML\n");
    let metadata = temp.path().join("metadata/metadata.tsv");
    let ms_dir = temp.path().join("msdata/raw");
    let missing = machaca::check_first(&metadata, &ms_dir, 1).err().unwrap();
    assert!(missing.contains("metadata references missing file \"missing.mzML\""));

    fs::write(&metadata, "filename\na.mzXML\nb.mzML\n").unwrap();
    fs::write(ms_dir.join("nested/a.mzXML"), "duplicate").unwrap();
    let ambiguous = machaca::check_first(&metadata, &ms_dir, 1).err().unwrap();
    assert!(ambiguous.contains("ambiguous filename \"a.mzXML\""));
}

#[test]
fn first_rows_reject_duplicate_selected_references_and_zero_limit() {
    let temp = tempfile::tempdir().unwrap();
    dataset(temp.path(), "filename\na.mzXML\na.mzXML\nb.mzML\n");
    let metadata = temp.path().join("metadata/metadata.tsv");
    let ms_dir = temp.path().join("msdata/raw");
    assert!(machaca::check_first(&metadata, &ms_dir, 0).is_err());
    assert!(machaca::check_first(&metadata, &ms_dir, 2)
        .err()
        .unwrap()
        .contains("duplicate reference \"a.mzXML\""));
}
