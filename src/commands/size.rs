use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::BinaryHeap;
use std::{
    cmp::Reverse,
    ffi::OsString,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

const MAX_SIZE_LEN: usize = 3;
const MAX_NAME_LEN: usize = 30;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct BasicFile {
    size: u64,
    name: String,
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
}
#[inline]
fn fmt_size(size: u64) -> (u64, u8, &'static str) {
    let (unit, suffix) = if size >= 1_000_000_000_000 {
        (1_000_000_000_000, "TB")
    } else if size >= 1_000_000_000 {
        (1_000_000_000, "GB")
    } else if size >= 1_000_000 {
        (1_000_000, "MB")
    } else if size >= 1_000 {
        (1_000, "kB")
    } else {
        return (size, 0, "B");
    };
    let int_part = size / unit;
    let rem = size % unit;
    let frac = ((rem * 10) / unit) as u8;

    (int_part, frac, suffix)
}
fn parallel_size(path: &Path, pb: &ProgressBar) -> u64 {
    if path.is_dir() {
        if let Ok(objects) = fs::read_dir(path) {
            let total: u64 = objects
                .flatten()
                .map(|x| x.path())
                .collect::<Vec<PathBuf>>()
                .into_par_iter()
                .map(|entry| {
                    pb.inc(1);
                    parallel_size(&entry, pb)
                })
                .sum::<u64>();
            return total;
        }
    } else if path.is_file() {
        return path.metadata().map(|x| x.len()).unwrap_or(0);
    }
    0
}

fn is_hdd(path: &Path) -> bool {
    let sys = sysinfo::Disks::new_with_refreshed_list();
    for disk in sys.list() {
        if path.starts_with(disk.mount_point()) {
            match disk.kind() {
                sysinfo::DiskKind::HDD => return true,
                sysinfo::DiskKind::SSD => return false,
                _ => return true,
            }
        }
    }
    true
}
fn children_size(path: &Path, limit: usize) -> Result<BinaryHeap<Reverse<BasicFile>>, io::Error> {
    let new_spinner = ProgressBar::new_spinner();
    let pb = new_spinner;
    let pb = Arc::new(pb);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let entries: Vec<_> = fs::read_dir(path).unwrap().flatten().collect();

    if is_hdd(path) {
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build_global()
            .expect("Failed")
    };
    let binheap: BinaryHeap<_> = entries
        .into_par_iter()
        .fold(
            || BinaryHeap::with_capacity(limit),
            |mut local_heap, entry| {
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
                let basic_file = BasicFile { name, size };
                if local_heap.len() < limit {
                    local_heap.push(Reverse(basic_file));
                } else if let Some(peek) = local_heap.peek() {
                    if basic_file.gt(&peek.0) {
                        local_heap.pop();
                        local_heap.push(Reverse(basic_file));
                    }
                }
                local_heap
            },
        )
        .reduce(
            || BinaryHeap::with_capacity(limit),
            |mut heap1, heap2| {
                for Reverse(v) in heap2 {
                    if heap1.len() < limit {
                        heap1.push(Reverse(v));
                    } else if let Some(Reverse(min)) = heap1.peek() {
                        if v > *min {
                            heap1.pop();
                            heap1.push(Reverse(v));
                        }
                    }
                }
                heap1
            },
        );
    pb.finish_and_clear();
    Ok(binheap)
}

pub fn print_sizes(path: &Path, limit: usize) -> std::io::Result<()> {
    let files = children_size(path, limit)?.into_sorted_vec();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    writeln!(
        &mut stdout,
        "File sizes in {}:",
        style(
            fs::canonicalize(path)
                .unwrap()
                .file_name()
                .unwrap_or(&OsString::from("Drive"))
                .display()
        )
        .bold()
    )?;
    let total_size = files.iter().fold(0, |acc, file| acc + file.0.size);

    for mut file in files {
        file.0.shorten_name();
        let (num, dec, suffix) = fmt_size(file.0.size);
        writeln!(
            &mut stdout,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$}.{} {}",
            file.0.name,
            style(num).cyan(),
            style(dec).cyan(),
            suffix
        )?;
    }
    let (num, dec, suffix) = fmt_size(total_size);
    writeln!(
        &mut stdout,
        "Total: {}.{} {}",
        style(num).cyan(),
        style(dec).cyan(),
        suffix
    )?;
    stdout.flush().unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_hdd() {
        assert_eq!(true, is_hdd(Path::new("E:\\"))); // unknown
        assert_eq!(false, is_hdd(Path::new("C:\\")));
        assert_eq!(false, is_hdd(Path::new("D:\\")));
    }
}
