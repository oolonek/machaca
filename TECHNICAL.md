# machaca technical guide

[Back to the machaca overview](README.md).

Machaca checks mass-spectrometry filenames in tabular sample metadata against an extracted sapayoa dataset. The first target is the Zenodo record `10018590` (`enpkg_toy_dataset.zip`); extract the archive outside the crate before running machaca. It does not modify the dataset.

## Check an extracted dataset

Set `DATASET_DIR` to the directory containing the extracted dataset (outside
the crate), then run from the `machaca` crate:

```sh
DATASET_DIR=/path/to/your/extracted/zenodo-10018590
cargo run -- check "$DATASET_DIR"
```

In a terminal, choose the metadata TSV/CSV and mass-spectrometry directory from the displayed candidates. For scripts, specify both paths relative to the extracted dataset directory (or use absolute paths within it):

```sh
cargo run -- check "$DATASET_DIR" \
  --metadata metadata/metadata.tsv --ms-dir msdata/raw
```

Machaca detects spectrum filename columns by `filename` in the header or a spectrum filename in their cells, except that MassIVE's `file_path` and `directus_filename` are ordinary metadata and never spectrum-reference columns. The `filename` column is still validated strictly: entries must be spectrum basenames, not paths or extensionless stems. Zenodo's `positive_filename` and `negative_filename` remain spectrum-reference columns. It reports the number of nonempty entries per detected column and recursively scans the selected directory for `.mzML`, `.mzXML`, `.mzData` and their `.gz` variants (case-insensitive extension). Matches use exact, case-sensitive basenames. It reports invalid entries, duplicate references, missing referenced files, files without entries, and ambiguous duplicate basenames as failures (nonzero exit). Processed `.mgf`/`.csv` files are not spectra and are excluded; interactive metadata discovery also omits TSV/CSV files inside `processed` and `raw` directories. Supply `--metadata` explicitly if the metadata file lives in either directory.

The current Zenodo toy dataset has three metadata rows, three positive and three negative filename entries, and six matching raw `.mzXML` files.

## MassIVE metadata

For the original MassIVE `MSV000102993` extraction, select its original metadata TSV and the `peak/peak` raw tree:

```sh
MASSIVE_DIR=/path/to/your/extracted/MSV000102993
cargo run -- check "$MASSIVE_DIR" \
  --metadata metadata/metadata/dbgi_massive_metadata_20260806.tsv --ms-dir peak/peak
```

When the extraction has all referenced spectra, validation succeeds without rewriting or dropping any metadata fields. `file_path` values such as `dbgi_msdata/sample.mzML`, extensionless `directus_filename` stems, taxon names, and `ionization_mode` remain in the source TSV for ingestion; only `filename` is used to map spectra. Missing or malformed `filename` references still fail.

For a first-N rehearsal against the unchanged original MassIVE extraction, add
`--max-rows 100` to the check command. Only the first 100 metadata records and
their referenced physical spectra are validated; unreferenced raw files and
later metadata rows are ignored. Within the selected records, malformed or
duplicate filenames and missing or ambiguous physical matches still fail.
Omit the option to check every metadata row and require every raw spectrum to
have a reference, as before. `N` must be a positive integer.

## Rust callers

Rust callers such as peregrino can use `machaca::check(metadata_path, ms_dir)` for complete one-to-one validation or `machaca::check_first(metadata_path, ms_dir, max_rows)` for the first N records only. Each returns the validated filename-column indexes and basename-to-file-path mapping; failures never publish a partial mapping.
