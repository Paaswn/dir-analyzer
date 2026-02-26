//! ## A module to check a project's lines of code, *including* comments.
use super::constant::*;
use crate::utils::Processor;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    cmp::Reverse,
    ffi::OsString,
    fmt::Write as fmtWrite,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, StdoutLock, Write},
    path::Path,
};
pub struct NestedPad(u16);
impl NestedPad {
    fn pad(&self) -> u16 {
        self.0 * PAD
    }
}
pub struct CodeFile {
    name: String,
    lines: usize,
}
pub struct LocScanner {
    pub extension: Extension,
    pub code_files: Vec<CodeFile>,
    pub name_buffer: String,
    pub print_buf: BufWriter<StdoutLock<'static>>,
    pub limit: usize,
    pub layout: PrintLayout,
    pub order: PrintOrder,
}
pub enum PrintOrder {
    Ascending,
    Descending,
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
impl Processor for LocScanner {
    type Custom = io::Result<()>;

    fn process_file(&mut self, file: &Path, pb: &ProgressBar) -> io::Result<()> {
        if let Some((name, ext)) = get_file_name(file) {
            if name.chars().count() > MAX_NAME_LEN {
                cut_name(&mut self.name_buffer, ext, name).unwrap();
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
impl LocScanner {
    pub fn walk_dir_nested(&mut self, path: &Path, pb: &indicatif::ProgressBar) -> io::Result<()> {
        //! # Reminder #
        //! this method is an alternative to `walk_dir`. How I'm gonna implement nested structure is padding
        //! everytime the tool walks into a new directory, and reset when it got back to the Root dir.
        //! # Might required #
        //! My future self probably need to implement an alternative of processfile function too!
        let mut dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };
        let mut helper = NestedPad(0);
        for file in dir {
            let file = file?;
            let metadata = file.metadata()?;
            let objec_path = file.path();
            if metadata.is_dir() {
                if self.is_dir_compatible(&objec_path) {
                    helper.0 += 1;
                    self.walk_dir(&objec_path, pb)?;
                };
            } else if self.is_file_compatible(&objec_path) {
                self.process_file(&objec_path, pb)?;
            }
        }
        Ok(())
    }
    pub fn walk_dir(&mut self, path: &Path, pb: &indicatif::ProgressBar) -> io::Result<()> {
        let mut dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };
        for file in dir {
            let file = file?;
            let metadata = file.metadata()?;
            let objec_path = file.path();
            if metadata.is_dir() {
                if self.is_dir_compatible(&objec_path) {
                    self.walk_dir(&objec_path, pb)?;
                };
            } else if self.is_file_compatible(&objec_path) {
                self.process_file(&objec_path, pb)?;
            }
        }
        Ok(())
    }
    fn add_file(&mut self, file: CodeFile) {
        self.code_files.push(file);
    }
    fn print_layout(&mut self, path: &Path, pad: u16) -> io::Result<()> {
        self.sort_files();
        self.print_header(path)?;
        match self.layout {
            PrintLayout::Nested => (),
            PrintLayout::OneLine => {
                for (rank, f) in self.code_files.iter().enumerate() {
                    if f.lines == 0 {
                        continue;
                    }
                    if rank + 1 >= self.limit {
                        break;
                    }
                    writeln!(
                        self.print_buf,
                        "{:<MAX_NAME_LEN$} {:>MAX_SIZE_LEN$} loc",
                        f.name,
                        style(f.lines).cyan()
                    )?;
                }
            }
        };
        self.print_footer()?;
        Ok(())
    }
    fn sort_files(&mut self) {
        match self.order {
            PrintOrder::Descending => {
                self.code_files.sort_by_key(|f| Reverse(f.lines));
            }
            PrintOrder::Ascending => {
                self.code_files.sort_by_key(|f| f.lines);
            }
            _ => (),
        }
    }
    fn print_header(&mut self, path: &Path) -> io::Result<()> {
        writeln!(
            self.print_buf,
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
        Ok(())
    }
    fn print_footer(&mut self) -> io::Result<()> {
        writeln!(
            self.print_buf,
            "Total: {} locs",
            style(self.total_loc()).cyan()
        )?;
        Ok(())
    }
    fn total_loc(&self) -> usize {
        self.code_files.iter().fold(0, |acc, loc| acc + loc.lines)
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

fn cut_name(buffer: &mut String, extension: &str, file_name: &str) -> std::fmt::Result {
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
pub fn print_loc(path: &Path, mut processor: LocScanner) -> io::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    processor.walk_dir(path, &pb)?;
    processor.print_layout(path, 0)?;
    pb.finish_and_clear();
    processor.print_buf.flush().unwrap();
    Ok(())
}
