//! Data manifest: file name, data-row count and SHA-256 of every input CSV of a run.
//!
//! The manifest is CSV (`file,rows,sha256`, files sorted). Its own SHA-256 is cited in the
//! report, and it is written as `manifests/manifest-<first 12 hex>.csv` next to the report, so
//! runs on identical inputs share one file. `data/` itself is not committed; the manifest is
//! what lets a later run prove it used the same bytes.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::CliError;

const SHORT_HASH: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub text: String,
    pub sha256: String,
    pub files: usize,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Rows after the header line (a trailing newline does not count as a row).
fn data_rows(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes).lines().filter(|l| !l.trim().is_empty()).count().saturating_sub(1)
}

/// Builds the manifest over `paths` (duplicates removed). Every path must be readable.
pub fn build(paths: &[PathBuf]) -> Result<Manifest, CliError> {
    let mut sorted: Vec<&PathBuf> = paths.iter().collect();
    sorted.sort();
    sorted.dedup();
    let mut text = String::from("file,rows,sha256\n");
    for p in &sorted {
        let bytes = fs::read(p).map_err(|e| CliError::Usage(format!("manifest: cannot read {}: {e}", p.display())))?;
        let _ = writeln!(text, "{},{},{}", p.display(), data_rows(&bytes), sha256_hex(&bytes));
    }
    Ok(Manifest { sha256: sha256_hex(text.as_bytes()), text, files: sorted.len() })
}

/// Writes `<report dir>/manifests/manifest-<hash12>.csv` and returns its path.
pub fn write(manifest: &Manifest, report: &Path) -> Result<PathBuf, CliError> {
    let dir = report.parent().unwrap_or(Path::new(".")).join("manifests");
    let path = dir.join(format!("manifest-{}.csv", &manifest.sha256[..SHORT_HASH]));
    let err = |e: std::io::Error| CliError::Write { path: path.display().to_string(), msg: e.to_string() };
    fs::create_dir_all(&dir).map_err(err)?;
    fs::write(&path, &manifest.text).map_err(err)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_the_standard_test_vector() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn counts_data_rows_without_header_or_blank_lines() {
        assert_eq!(data_rows(b"date,close\n2012-01-02,1\n2012-01-03,2\n\n"), 2);
        assert_eq!(data_rows(b""), 0);
    }

    #[test]
    fn manifest_lists_sorted_unique_files_and_hashes_itself() {
        let dir = std::env::temp_dir().join(format!("tq-manifest-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("A.csv"), dir.join("B.csv"));
        fs::write(&a, "date,close\n2012-01-02,1\n").unwrap();
        fs::write(&b, "abc").unwrap();
        let m = build(&[b.clone(), a.clone(), b.clone()]).unwrap();
        let lines: Vec<&str> = m.text.lines().collect();
        assert_eq!(m.files, 2);
        assert!(lines[1].starts_with(&format!("{},1,", a.display())), "{}", m.text);
        assert!(lines[2].ends_with(",0,ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert_eq!(m.sha256, sha256_hex(m.text.as_bytes()));
        let written = write(&m, &dir.join("report.md")).unwrap();
        assert_eq!(fs::read_to_string(&written).unwrap(), m.text);
        assert!(build(&[dir.join("missing.csv")]).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
