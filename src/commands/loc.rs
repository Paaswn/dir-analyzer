//! ## A module to check a project's lines of code, *including* comments.
use std::{
    cmp::Reverse,
    fmt::Write as fmtWrite,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write, stdout},
    path::PathBuf,
};

use crate::utils::{Processor, walk_dir};
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
struct LocScanner {
    code_files: Vec<CodeFile>,
    name_buffer: String,
}
impl LocScanner {
    fn add_file(&mut self, file: CodeFile) {
        self.code_files.push(file);
    }
}
impl Processor for LocScanner {
    type Custom = io::Result<()>;

    fn process_file(&mut self, file: &PathBuf, pb: &ProgressBar) -> io::Result<()> {
        fn name_shorten(buffer: &mut String, extension: &str, file_name: &str) -> std::fmt::Result {
            buffer.write_fmt(format_args!(
                "{}...{}",
                &file_name.get(..5).unwrap_or(file_name),
                extension
            ))?;
            Ok(())
        }
        if let Some((name, ext)) = getf_name_ext(file) {
            if name.chars().count() > MAX_NAME_LEN {
                name_shorten(&mut self.name_buffer, ext, name).unwrap();
                pb.set_message(format!("Reading {}...", self.name_buffer));
            } else {
                pb.set_message(format!("Reading {}", name));
                self.name_buffer
                    .write_fmt(format_args!("{}", name))
                    .unwrap();
            }
        }
        pb.tick();
        let content = fs::File::open(file)?;
        let reader = BufReader::new(content);
        let loc = reader.split(b'\n').count();
        self.add_file(CodeFile {
            name: self.name_buffer.clone(),
            lines: loc,
        });
        self.name_buffer.clear();
        Ok(())
    }

    fn is_dir_compatible(&self, path: &PathBuf) -> bool {
        let dir_name = path.file_name().and_then(|x| x.to_str()).unwrap();
        if IGNORE_DIRS.contains(&dir_name) {
            return false;
        }
        true
    }

    fn is_file_compatible(&self, path: &PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if CODE_EXTENSIONS.contains(&ext_str) {
                    return true;
                }
            }
        }
        false
    }
}

fn getf_name_ext(path: &PathBuf) -> Option<(&str, &str)> {
    let name = path.file_name();
    let ext = path.extension();
    match (name, ext) {
        (Some(name), Some(ext)) => {
            return Some((
                name.to_str().unwrap_or("default-name"),
                ext.to_str().unwrap_or("dflt"),
            ));
        }
        (_, _) => {
            eprintln!("skip this file!");
        }
    }
    None
}

pub fn print_loc(path: &PathBuf, top: Option<usize>) -> io::Result<()> {
    let mut processor = LocScanner {
        code_files: Vec::new(),
        name_buffer: String::new(),
    };
    let mut print_buf = BufWriter::new(stdout().lock());
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    walk_dir(path, &mut processor, &pb)?;
    processor.code_files.sort_by_key(|x| Reverse(x.lines));
    let total_loc = total_loc(&processor);
    writeln!(
        &mut print_buf,
        "Locs written in {}:",
        fs::canonicalize(path)
            .unwrap()
            .file_name()
            .unwrap()
            .display()
    )?;
    for (rank, f) in processor.code_files.into_iter().enumerate() {
        if f.lines == 0 {
            continue;
        }
        if rank + 1 >= top.unwrap_or(100) {
            break;
        }
        writeln!(
            &mut print_buf,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$} loc",
            f.name, f.lines
        )?;
    }
    writeln!(&mut print_buf, "Total: {} locs", total_loc)?;
    pb.finish_and_clear();
    print_buf.flush().unwrap();
    Ok(())
}

fn total_loc(scanner: &LocScanner) -> usize {
    scanner
        .code_files
        .iter()
        .fold(0, |acc, loc| acc + loc.lines)
}
