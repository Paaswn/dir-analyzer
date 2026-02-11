//! ## A module to check a project's lines of code, *including* comments.
use super::constant::*;
use crate::utils::{Processor, walk_dir};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    cmp::{Ordering, Reverse},
    ffi::OsString,
    fmt::Write as fmtWrite,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write, stdout},
    path::Path,
};
struct CodeFile {
    name: String,
    lines: usize,
}
pub struct LocScanner {
    code_files: Vec<CodeFile>,
    name_buffer: String,
    extension: Extension,
    layout: PrintLayout,
    limit: usize,
}
pub enum PrintLayout {
    OneLine,
    Nested,
}
pub enum Extension {
    Only(Vec<String>),
    Ignore(Vec<String>),
    Default,
}
impl LocScanner {
    fn add_file(&mut self, file: CodeFile) {
        self.code_files.push(file);
    }
    fn new() -> Self {
        Self {
            code_files: Vec::new(),
            name_buffer: String::new(),
            extension: Extension::Default,
            layout: PrintLayout::OneLine,
            limit: 100,
        }
    }
    fn layout(&mut self, layout: PrintLayout) {
        self.layout = layout;
    }
    fn extension(&mut self, ext: Extension) {
        self.extension = ext;
    }
    fn print_layout(&self) {
        match self.layout {
            PrintLayout::Nested => (),
            PrintLayout::OneLine => (),
        };
    }
    fn sort_files(&mut self, cmp: Ordering) {
        match cmp {
            Ordering::Greater => {
                self.code_files.sort_by_key(|f| Reverse(f.lines));
            }
            Ordering::Less => {
                self.code_files.sort_by_key(|f| f.lines);
            }
            _ => (),
        }
    }
}
impl Processor for LocScanner {
    type Custom = io::Result<()>;

    fn process_file(&mut self, file: &Path, pb: &ProgressBar) -> io::Result<()> {
        fn name_shorten(buffer: &mut String, extension: &str, file_name: &str) -> std::fmt::Result {
            buffer.write_fmt(format_args!(
                "{}...{}",
                &file_name.get(..5).unwrap_or(file_name),
                extension
            ))?;
            Ok(())
        }
        fn get_loc(reader: BufReader<File>) -> usize {
            reader
                .split(b'\n')
                .filter_map(Result::ok)
                .filter(|line| {
                    let line = line.strip_suffix(b"\r").unwrap_or(line);
                    !line.is_empty()
                })
                .count()
        }
        if let Some((name, ext)) = get_file_name(file) {
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
        let loc = get_loc(reader);
        self.add_file(CodeFile {
            name: self.name_buffer.clone(),
            lines: loc,
        });
        self.name_buffer.clear();
        Ok(())
    }

    fn is_dir_compatible(&self, path: &Path) -> bool {
        let dir_name = path.file_name().and_then(|x| x.to_str()).unwrap();
        if IGNORE_DIRS.contains(&dir_name) {
            return false;
        }
        true
    }

    fn is_file_compatible(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext_str| match &self.extension {
                Extension::Ignore(ignores) => {
                    CODE_EXTENSIONS.contains(&ext_str) && !ignores.contains(&ext_str.to_owned())
                }
                Extension::Only(onlys) => onlys.contains(&ext_str.to_owned()),
                Extension::Default => CODE_EXTENSIONS.contains(&ext_str),
            })
    }
}

fn get_file_name(path: &Path) -> Option<(&str, &str)> {
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

pub fn print_loc(path: &Path, top: usize, exclusive: Extension) -> io::Result<()> {
    let mut processor = LocScanner::new();
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
        style(
            fs::canonicalize(path)
                .unwrap()
                .file_name()
                .unwrap_or(&OsString::from("Drive"))
                .display()
        )
        .bold()
    )?;
    for (rank, f) in processor.code_files.into_iter().enumerate() {
        if f.lines == 0 {
            continue;
        }
        if rank + 1 >= top {
            break;
        }
        writeln!(
            &mut print_buf,
            "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$} loc",
            f.name,
            style(f.lines).cyan()
        )?;
    }
    writeln!(&mut print_buf, "Total: {} locs", style(total_loc).cyan())?;
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
