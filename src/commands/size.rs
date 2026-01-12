use std::{
    cmp::Reverse,
    fs,
    io::{self, Write},
    path::PathBuf,
};

const MAX_SIZE_LEN: usize = 3;
const MAX_NAME_LEN: usize = 30;

struct BasicFile {
    name: String,
    size: u128,
}

impl BasicFile {
    fn fmt_name(&mut self) {
        let name_len = self.name.chars().count();
        if name_len > MAX_NAME_LEN {
            let cropped_name: String = self.name.chars().take(MAX_NAME_LEN - 5).collect();
            let cropped_name = format!("{}...", cropped_name);
            self.name = cropped_name;
        }
    }
    #[inline]
    fn fmt_size(&self) -> (u128, u8, &'static str) {
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
//
fn print_size(dir: &PathBuf, limit: Option<usize>) -> io::Result<()> {
    let mut print_buf: Vec<u8> = Vec::with_capacity(4096);
    let mut files = list_top_level(dir)?;
    files.sort_by_key(|x| Reverse(x.size));
    for (rank, mut f) in files.into_iter().enumerate() {
        if f.size == 0 {
            continue;
        }
        if rank + 1 > limit.unwrap_or(10) {
            break;
        }
        let (size, dec, suffix) = f.fmt_size();
        f.fmt_name();
        writeln!(
            &mut print_buf,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$}.{} {}",
            f.name, size, dec, suffix
        )?;
    }
    let mut out = io::BufWriter::new(io::stdout());
    out.write_all(&print_buf).unwrap();
    out.flush().unwrap();
    Ok(())
}
//

fn list_top_level(path: &PathBuf) -> io::Result<Vec<BasicFile>> {
    fn get_filelike_size(path: &PathBuf, items: &mut Vec<BasicFile>) -> io::Result<u128> {
        let mut total_size = 0u128;
        let dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return Ok(0),
        };

        for file in dir {
            let file = file?;
            let metadata = file.metadata()?;
            if metadata.is_dir() {
                get_filelike_size(&file.path(), items)?;
            } else {
                total_size += file.metadata()?.len() as u128;
            }
        }
        Ok(total_size)
    }
    let mut out = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        let name = entry.file_name().to_string_lossy().into_owned();

        if meta.is_dir() {
            let size = get_filelike_size(&entry.path(), &mut out)?;
            out.push(BasicFile { name, size });
        } else if meta.is_file() {
            out.push(BasicFile {
                name,
                size: meta.len() as u128,
            });
        }
    }

    Ok(out)
}

pub fn get_top_sizes(path: &PathBuf, limit: Option<usize>) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        print_size(path, limit)?
    } else {
        println!("{}", metadata.len());
    };
    Ok(())
}
