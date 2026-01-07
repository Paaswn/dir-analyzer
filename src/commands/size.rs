use std::{
    cmp::Reverse,
    fmt::Write as FmtWrite,
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

fn print_fmt_metadata(dir: &PathBuf) -> io::Result<()> {
    fn get_fmt_name(buffer: &mut String, name: &str) {
        let name_len = name.chars().count();
        if name_len > MAX_NAME_LEN {
            let cropped_name: String = name.chars().take(MAX_NAME_LEN - 5).collect();
            let cropped_name = format!("{}...", cropped_name);
            write!(buffer, "{}", cropped_name).unwrap();
        } else {
            write!(buffer, "{}", name).unwrap();
        }
    }

    #[inline]
    fn get_fmt_size(file_size: u128) -> (u128, u8, &'static str) {
        let (unit, suffix) = if file_size >= 1_000_000_000_000 {
            (1_000_000_000_000, "TB")
        } else if file_size >= 1_000_000_000 {
            (1_000_000_000, "GB")
        } else if file_size >= 1_000_000 {
            (1_000_000, "MB")
        } else if file_size >= 1_000 {
            (1_000, "kB")
        } else {
            return (file_size, 0, "B");
        };
        let int_part = file_size / unit;
        let rem = file_size % unit;
        let frac = ((rem * 10) / unit) as u8;

        (int_part, frac, suffix)
    }
    let mut print_buf: Vec<u8> = Vec::with_capacity(4096);
    let mut fmt_name: String = String::with_capacity(MAX_NAME_LEN);
    let mut files = list_top_level(dir)?;
    files.sort_by_key(|x| Reverse(x.size));
    for f in files {
        let (size, dec, suffix) = get_fmt_size(f.size);
        fmt_name.clear();
        get_fmt_name(&mut fmt_name, &f.name);
        writeln!(
            &mut print_buf,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$}.{} {}",
            fmt_name, size, dec, suffix
        )?;
    }
    let mut out = io::BufWriter::new(io::stdout());
    out.write_all(&print_buf).unwrap();
    out.flush().unwrap();
    Ok(())
}
fn list_top_level(path: &PathBuf) -> io::Result<Vec<BasicFile>> {
    fn analyze_dir(path: &PathBuf, items: &mut Vec<BasicFile>) -> io::Result<u128> {
        let mut total_size = 0u128;
        let dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return Ok(0),
        };

        for file in dir {
            let file = file?;
            let metadata = file.metadata()?;
            if metadata.is_dir() {
                analyze_dir(&file.path(), items)?;
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
            let size = analyze_dir(&entry.path(), &mut out)?;
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

pub fn get_top_sizes(path: &PathBuf) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        print_fmt_metadata(path)?
    } else {
        println!("{}", metadata.len());
    };
    Ok(())
}
