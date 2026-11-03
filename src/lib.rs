use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

struct Options {
    root: PathBuf,
    metadata: Option<PathBuf>,
    ms_dir: Option<PathBuf>,
    max_rows: Option<usize>,
}

struct Report {
    rows: usize,
    columns: Vec<(String, usize)>,
    files: usize,
    issues: Vec<String>,
    checked: Option<Checked>,
}

/// Filename columns and basename-to-path mapping for validated metadata rows.
pub struct Checked {
    /// Zero-based metadata column positions containing spectrum filenames.
    pub filename_columns: Vec<usize>,
    /// Validated basenames mapped to their paths under the selected spectrum directory.
    pub files: BTreeMap<String, PathBuf>,
}

/// Validate every metadata row and require every spectrum in the directory to be referenced.
pub fn check(metadata: &Path, ms_dir: &Path) -> Result<Checked, String> {
    checked(validate(metadata, ms_dir, None)?)
}

/// Validate the first `max_rows` metadata records and only their referenced spectra.
pub fn check_first(metadata: &Path, ms_dir: &Path, max_rows: usize) -> Result<Checked, String> {
    if max_rows == 0 {
        return Err("max_rows must be greater than zero".into());
    }
    checked(validate(metadata, ms_dir, Some(max_rows))?)
}

fn checked(report: Report) -> Result<Checked, String> {
    if !report.issues.is_empty() {
        return Err(report.issues.join("\n"));
    }
    Ok(report.checked.expect("validated report contains filenames"))
}

fn usage() -> &'static str {
    "Usage: machaca check <dataset-dir> [--metadata <file>] [--ms-dir <directory>] [--max-rows N]"
}

fn options() -> Result<Option<Options>, String> {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        return Err(usage().into());
    };
    if command == "--help" || command == "-h" {
        println!("{}", usage());
        return Ok(None);
    }
    if command != "check" {
        return Err(usage().into());
    }
    let root = args.next().ok_or_else(|| usage().to_string())?;
    let (mut metadata, mut ms_dir, mut max_rows) = (None, None, None);
    while let Some(arg) = args.next() {
        if arg == "--max-rows" && max_rows.is_none() {
            let value = args.next().ok_or_else(|| usage().to_string())?;
            max_rows = Some(
                value
                    .to_str()
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|&value| value > 0)
                    .ok_or_else(|| "--max-rows must be a positive integer".to_string())?,
            );
            continue;
        }
        let target = match arg.to_str() {
            Some("--metadata") if metadata.is_none() => &mut metadata,
            Some("--ms-dir") if ms_dir.is_none() => &mut ms_dir,
            _ => return Err(usage().into()),
        };
        *target = Some(PathBuf::from(
            args.next().ok_or_else(|| usage().to_string())?,
        ));
    }
    Ok(Some(Options {
        root: PathBuf::from(root),
        metadata,
        ms_dir,
        max_rows,
    }))
}

fn spectrum_name(name: &str) -> bool {
    let uncompressed = name
        .get(..name.len().saturating_sub(3))
        .filter(|_| {
            name.get(name.len().saturating_sub(3)..)
                .is_some_and(|s| s.eq_ignore_ascii_case(".gz"))
        })
        .unwrap_or(name);
    let Some((stem, extension)) = uncompressed.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && (extension.eq_ignore_ascii_case("mzml")
            || extension.eq_ignore_ascii_case("mzxml")
            || extension.eq_ignore_ascii_case("mzdata"))
}

fn discover(root: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let (mut metadata, mut ms_dirs) = (BTreeSet::new(), BTreeSet::new());
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if entry.file_type().is_dir() {
            if entry
                .file_name()
                .to_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("raw"))
            {
                ms_dirs.insert(path.to_path_buf());
            }
        } else if entry.file_type().is_file() {
            let name = entry.file_name().to_string_lossy();
            let tabular = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("tsv") || extension.eq_ignore_ascii_case("csv")
                });
            let processed = path
                .parent()
                .expect("file has parent")
                .strip_prefix(root)
                .expect("walked path is inside root")
                .components()
                .any(|component| {
                    component.as_os_str().to_str().is_some_and(|name| {
                        name.eq_ignore_ascii_case("processed") || name.eq_ignore_ascii_case("raw")
                    })
                });
            if tabular && !processed {
                metadata.insert(path.to_path_buf());
            }
            if spectrum_name(&name) {
                ms_dirs.insert(path.parent().expect("file has parent").to_path_buf());
            }
        }
    }
    Ok((
        metadata.into_iter().collect(),
        ms_dirs.into_iter().collect(),
    ))
}

fn select(
    root: &Path,
    given: Option<PathBuf>,
    candidates: &[PathBuf],
    flag: &str,
    is_file: bool,
) -> Result<PathBuf, String> {
    let path = if let Some(given) = given {
        if given.is_absolute() {
            given
        } else {
            root.join(given)
        }
    } else {
        if candidates.is_empty() {
            return Err(format!("No candidates found; supply {flag}"));
        }
        if !io::stdin().is_terminal() {
            return Err(format!("Supply {flag} when running non-interactively"));
        }
        eprintln!("Choose {flag}:");
        for (index, candidate) in candidates.iter().enumerate() {
            eprintln!("  {}: {}", index + 1, candidate.display());
        }
        eprint!("Selection [1-{}]: ", candidates.len());
        io::stderr().flush().map_err(|error| error.to_string())?;
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| error.to_string())?;
        let index: usize = answer
            .trim()
            .parse()
            .map_err(|_| "Invalid selection".to_string())?;
        candidates
            .get(index.wrapping_sub(1))
            .ok_or_else(|| "Invalid selection".to_string())?
            .clone()
    };
    let path = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !path.starts_with(root) {
        return Err(format!("{} is outside the dataset", path.display()));
    }
    if (is_file && !path.is_file()) || (!is_file && !path.is_dir()) {
        return Err(format!("{} has the wrong file type", path.display()));
    }
    Ok(path)
}

fn reader(path: &Path) -> Result<csv::Reader<std::fs::File>, String> {
    let delimiter = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
    {
        b','
    } else {
        b'\t'
    };
    csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_path(path)
        .map_err(|error| error.to_string())
}

fn metadata_only_column(header: &str) -> bool {
    header.eq_ignore_ascii_case("file_path") || header.eq_ignore_ascii_case("directus_filename")
}

fn validate(metadata: &Path, ms_dir: &Path, max_rows: Option<usize>) -> Result<Report, String> {
    let mut first = reader(metadata)?;
    let headers = first.headers().map_err(|error| error.to_string())?.clone();
    let mut detected = vec![false; headers.len()];
    for (index, header) in headers.iter().enumerate() {
        if !metadata_only_column(header) {
            detected[index] = header
                .as_bytes()
                .windows(b"filename".len())
                .any(|part| part.eq_ignore_ascii_case(b"filename"));
        }
    }
    for result in first.records().take(max_rows.unwrap_or(usize::MAX)) {
        let record = result.map_err(|error| error.to_string())?;
        for (index, value) in record.iter().enumerate() {
            if !metadata_only_column(headers.get(index).expect("CSV record matches header")) {
                detected[index] |= spectrum_name(value.trim());
            }
        }
    }
    let columns: Vec<_> = detected
        .iter()
        .enumerate()
        .filter_map(|(i, found)| found.then_some(i))
        .collect();
    if columns.is_empty() {
        return Err("No mass-spectrometry filename column found in metadata".into());
    }

    let mut report = Report {
        rows: 0,
        columns: columns
            .iter()
            .map(|&i| (headers[i].to_owned(), 0))
            .collect(),
        files: 0,
        issues: Vec::new(),
        checked: None,
    };
    let mut references = BTreeMap::<String, usize>::new();
    let mut second = reader(metadata)?;
    for result in second.records().take(max_rows.unwrap_or(usize::MAX)) {
        let record = result.map_err(|error| error.to_string())?;
        report.rows += 1;
        for (column_index, &index) in columns.iter().enumerate() {
            let value = record.get(index).expect("CSV record matches header").trim();
            if value.is_empty() {
                continue;
            }
            report.columns[column_index].1 += 1;
            if !spectrum_name(value)
                || Path::new(value)
                    .file_name()
                    .is_none_or(|name| name != value)
            {
                report.issues.push(format!(
                    "row {} column {}: invalid spectrum filename {value:?}",
                    report.rows + 1,
                    headers.get(index).expect("header index exists")
                ));
                continue;
            }
            if let Some(previous) = references.insert(value.to_owned(), report.rows + 1) {
                report.issues.push(format!(
                    "row {}: duplicate reference {value:?} (first on row {previous})",
                    report.rows + 1
                ));
            }
        }
    }
    let mut files = BTreeMap::<String, PathBuf>::new();
    for entry in WalkDir::new(ms_dir) {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if !spectrum_name(&name) {
            continue;
        }
        if max_rows.is_some() && !references.contains_key(name.as_ref()) {
            continue;
        }
        report.files += 1;
        if let Some(previous) = files.insert(name.to_string(), entry.path().to_path_buf()) {
            report.issues.push(format!(
                "ambiguous filename {name:?}: {} and {}",
                previous.display(),
                entry.path().display()
            ));
        }
    }
    if report.files == 0 {
        report
            .issues
            .push("No mass-spectrometry files found in selected directory".into());
    }
    for name in references.keys() {
        if !files.contains_key(name) {
            report
                .issues
                .push(format!("metadata references missing file {name:?}"));
        }
    }
    if max_rows.is_none() {
        for (name, path) in &files {
            if !references.contains_key(name) {
                report.issues.push(format!(
                    "file without metadata entry: {} ({name})",
                    path.display()
                ));
            }
        }
    }
    report.checked = Some(Checked {
        filename_columns: columns,
        files,
    });
    Ok(report)
}

/// Run the interactive or explicit-path CLI and print its validation report.
pub fn run() -> Result<(), String> {
    let Some(options) = options()? else {
        return Ok(());
    };
    let root = options
        .root
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }
    let (metadata_candidates, ms_candidates) =
        if options.metadata.is_none() || options.ms_dir.is_none() {
            discover(&root)?
        } else {
            (Vec::new(), Vec::new())
        };
    let metadata = select(
        &root,
        options.metadata,
        &metadata_candidates,
        "--metadata",
        true,
    )?;
    let ms_dir = select(&root, options.ms_dir, &ms_candidates, "--ms-dir", false)?;
    let report = validate(&metadata, &ms_dir, options.max_rows)?;
    println!("Metadata: {} ({} rows)", metadata.display(), report.rows);
    for (name, count) in &report.columns {
        println!("  {name}: {count} nonempty filename entries");
    }
    println!("Mass-spectrometry files: {}", report.files);
    for issue in &report.issues {
        eprintln!("ERROR: {issue}");
    }
    if !report.issues.is_empty() {
        return Err(format!(
            "validation failed: {} issue(s)",
            report.issues.len()
        ));
    }
    println!("Validation passed");
    Ok(())
}
