use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{cmp::Reverse, fs, io::Write, path::PathBuf, sync::Arc};

const MAX_SIZE_LEN: usize = 3;
const MAX_NAME_LEN: usize = 30;

struct BasicFile {
    name: String,
    size: u64,
}

impl BasicFile {
    fn shorten_name(&mut self) {
        let name_len = self.name.chars().count();
        if name_len > MAX_NAME_LEN {
            let cropped_name: String = self.name.chars().take(MAX_NAME_LEN - 5).collect();
            let cropped_name = format!("{}...", cropped_name);
            self.name = cropped_name;
        }
    }
    #[inline]
    fn fmt_size(&self) -> (u64, u8, &'static str) {
        let (unit, suffix) = if self.size >= 1_000_000_000_000 {
            (1_000_000_000_000, "TB")
        } else if self.size >= 1_000_000_000 {
            (1_000_000_000, "GB")
        } else if self.size >= 1_000_000 {
            (1_000_000, "MB")
        } else if self.size >= 1_000 {
            (1_000, "kB")
        } else {
            return (self.size, 0, "B");
        };
        let int_part = self.size / unit;
        let rem = self.size % unit;
        let frac = ((rem * 10) / unit) as u8;

        (int_part, frac, suffix)
    }
}
fn parallel_size(path: &PathBuf, pb: &ProgressBar) -> u64 {
    if path.is_dir() {
        if let Ok(objects) = fs::read_dir(path) {
            // Instead of for_each + Atomic, you can do:
            let total: u64 = objects
                .flatten()
                .map(|x| x.path())
                .collect::<Vec<PathBuf>>()
                .into_par_iter()
                .map(|entry| {
                    pb.inc(1);
                    parallel_size(&entry, pb)
                }) // Each thread returns a number
                .sum::<u64>(); // Rayon adds them all up at the end
            return total;
        }
    } else if path.is_file() {
        return path.metadata().map(|x| x.len()).unwrap_or(0);
    }
    0
}

fn children_size(path: &PathBuf) -> std::io::Result<Vec<BasicFile>> {
    let new_spinner = ProgressBar::new_spinner();
    let pb = new_spinner;
    let pb = Arc::new(pb);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let entries: Vec<_> = fs::read_dir(path).unwrap().flatten().collect();

    // 1. calculate everything in parallel first!
    // we store the (path, size) pairs in a new vec.
    let mut results: Vec<BasicFile> = entries
        .into_par_iter()
        .map(|entry| {
            let pb = Arc::clone(&pb);
            let path = entry.path();
            let name: String = path
                .file_name()
                .map(|x| x.to_string_lossy().into_owned())
                .unwrap_or("UnknownName".to_owned());
            if pb.length().unwrap_or(0) % 15 == 0 {
                pb.set_message(format!("Reading {}...", name));
            }
            let size = parallel_size(&path, &pb);
            BasicFile { name, size }
        })
        .collect();
    pb.finish_and_clear();
    results.sort_by_key(|x| Reverse(x.size));
    Ok(results)
}

pub fn print_sizes(path: &PathBuf, limit: Option<usize>) -> std::io::Result<()> {
    let files = children_size(path);
    let mut print_buffer: Vec<u8> = Vec::new();
    for (i, mut file) in files?.into_iter().enumerate() {
        file.shorten_name();
        let (num, dec, suffix) = file.fmt_size();
        if i + 1 >= limit.unwrap_or(10) {
            break;
        }
        writeln!(
            &mut print_buffer,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$}.{} {}",
            file.name, num, dec, suffix
        )?;
    }
    let mut stdout = std::io::BufWriter::new(std::io::stdout().lock());
    stdout.write_all(&print_buffer)?;
    stdout.flush().unwrap();
    Ok(())
}
