//! Reads a recorder directory in canonical order, tolerating the truncated gzip tail a killed
//! recorder leaves behind (the readable prefix is kept and the file is counted as truncated).

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

/// `<stem>.jsonl.gz` then `.gz.1`, `.gz.2`, ... by numeric suffix, hours in name order.
pub fn ordered_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<(String, u32, PathBuf)> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_string();
            let (stem, rest) = name.split_once(".jsonl.gz")?;
            let n = match rest.strip_prefix('.') {
                None if rest.is_empty() => 0,
                Some(d) => d.parse().ok()?,
                None => return None,
            };
            Some((stem.to_string(), n, p))
        })
        .collect();
    files.sort();
    Ok(files.into_iter().map(|(_, _, p)| p).collect())
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ReadStats {
    pub files: usize,
    pub truncated_files: Vec<String>,
    pub lines: u64,
}

/// Calls `f` with every complete line of every file, in order.
pub fn for_each_line(files: &[PathBuf], stats: &mut ReadStats, mut f: impl FnMut(&str)) -> io::Result<()> {
    let mut buf = String::new();
    for path in files {
        stats.files += 1;
        let mut r = BufReader::new(GzDecoder::new(File::open(path)?));
        loop {
            buf.clear();
            match r.read_line(&mut buf) {
                Ok(0) => break,
                // A line without '\n' at EOF is a cut-off write; drop it.
                Ok(_) if !buf.ends_with('\n') => {
                    stats.truncated_files.push(path.display().to_string());
                    break;
                }
                Ok(_) => {
                    stats.lines += 1;
                    f(buf.trim_end());
                }
                Err(_) => {
                    stats.truncated_files.push(path.display().to_string());
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn orders_restart_suffixes_numerically() {
        let dir = std::env::temp_dir().join(format!("tq-mm-order-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for n in ["H1.jsonl.gz.10", "H1.jsonl.gz", "H2.jsonl.gz", "H1.jsonl.gz.2", "um-recorder.log"] {
            File::create(dir.join(n)).unwrap();
        }
        let names: Vec<String> =
            ordered_files(&dir).unwrap().iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_string()).collect();
        assert_eq!(names, ["H1.jsonl.gz", "H1.jsonl.gz.2", "H1.jsonl.gz.10", "H2.jsonl.gz"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn keeps_readable_prefix_of_a_truncated_gzip() {
        let path = std::env::temp_dir().join(format!("tq-mm-trunc-{}.jsonl.gz", std::process::id()));
        let mut enc = GzEncoder::new(File::create(&path).unwrap(), Compression::default());
        enc.write_all(b"one\ntwo\npartial").unwrap();
        enc.flush().unwrap(); // sync flush, no gzip trailer: what a killed recorder leaves
        std::mem::forget(enc);
        let mut stats = ReadStats::default();
        let mut got = vec![];
        for_each_line(&[path.clone()], &mut stats, |l| got.push(l.to_string())).unwrap();
        assert_eq!(got, ["one", "two"]);
        assert_eq!(stats.truncated_files.len(), 1);
        std::fs::remove_file(&path).unwrap();
    }
}
