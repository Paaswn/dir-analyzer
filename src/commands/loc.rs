//! A module to check the project's lines of code, **including** comments.
use std::{
    cmp::Reverse,
    fmt::Write as fmtWrite,
    fs,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};
const MAX_SIZE_LEN: usize = 10;
const MAX_NAME_LEN: usize = 30;
const CODE_EXTENSIONS: &[&'static str] = &[
    "rs", "js", "ts", "py", "java", "c", "cpp", "h", "hpp", "go", "rb", "php", "swift", "kt", "cs",
    "lua", "pl", "sh", "bash", "html", "css", "scss", "sass", "less", "md", "txt",
];
const IGNORE_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".idea",
    "dist",
    ".venv",
    ".env",
];
struct CodeFile {
    name: String,
    lines: usize,
}
fn walk_dir(path: &PathBuf, files: &mut Vec<CodeFile>, pb: &ProgressBar) -> io::Result<()> {
    fn is_compatible(path: &PathBuf) -> (bool, &str) {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if CODE_EXTENSIONS.contains(&ext_str) {
                    return (true, ext_str);
                }
            }
        }
        (false, "skip")
    }
    let mut file_buf = String::new();
    let dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };
    for file in dir {
        let file = file?;
        let metadata = file.metadata()?;
        let path = file.path();
        let (is_suit, ext) = is_compatible(&path);
        if metadata.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                if IGNORE_DIRS.contains(&dir_name) {
                    continue;
                }
            }
            walk_dir(&path, files, pb)?;
        } else if is_suit {
            get_loc(&path, files, ext, &mut file_buf, pb).unwrap();
        }
    }
    Ok(())
}
fn get_loc(
    file: &PathBuf,
    files: &mut Vec<CodeFile>,
    ext_name: &str,
    file_name_buf: &mut String,
    pb: &ProgressBar,
) -> io::Result<()> {
    fn name_shorten(buffer: &mut String, extension: &str, file_name: &str) -> std::fmt::Result {
        buffer.write_fmt(format_args!(
            "{}...{}",
            &file_name.get(..5).unwrap_or(file_name),
            extension
        ))?;
        Ok(())
    }
    if let Some(file_name) = file.file_name().and_then(|s| s.to_str()) {
        if file_name.len() > MAX_NAME_LEN {
            name_shorten(file_name_buf, ext_name, &file_name).expect("Fail to shorten file name");
        } else {
            file_name_buf
                .write_str(&file_name)
                .expect("Fail to write file_name to buffer");
        }
    };
    pb.set_message(format!("Reading {}...", file_name_buf));
    pb.tick();
    let content = fs::File::open(file)?;
    let reader = BufReader::new(content);
    let loc = reader.split(b'\n').count();
    files.push(CodeFile {
        name: file_name_buf.clone(),
        lines: loc,
    });
    file_name_buf.clear();
    Ok(())
}
pub fn print_loc(path: &PathBuf, top: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let mut print_buf: Vec<u8> = Vec::with_capacity(4096);
    let mut files: Vec<CodeFile> = Vec::new();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    walk_dir(path, &mut files, &pb)?;
    files.sort_by_key(|x| Reverse(x.lines));
    for (rank, f) in files.into_iter().enumerate() {
        if f.lines == 0 {
            continue;
        }
        if rank >= top.unwrap_or(10) {
            break;
        }
        writeln!(
            &mut print_buf,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$} loc",
            f.name, f.lines
        )?;
    }
    let mut outbuf = std::io::BufWriter::new(io::stdout());
    outbuf.write_all(&print_buf).unwrap();
    outbuf.flush().unwrap();
    Ok(())
}
