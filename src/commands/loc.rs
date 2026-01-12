//! A module to check the project's lines of code, **including** comments.
use std::{
    cmp::Reverse,
    fmt::Write as fmtWrite,
    fs,
    io::{self, Write},
    path::PathBuf,
};
const MAX_SIZE_LEN: usize = 10;
const MAX_NAME_LEN: usize = 30;
const CODE_EXTENSIONS: &[&'static str] = &[
    "rs", "js", "ts", "py", "java", "c", "cpp", "h", "hpp", "go", "rb", "php", "swift", "kt", "cs",
    "lua", "pl", "sh", "bash", "html", "css", "scss", "sass", "less", "xml", "json", "yaml", "yml",
    "toml", "md", "txt",
];
struct CodeFile {
    name: String,
    lines: usize,
}
fn walk_dir(path: &PathBuf, files: &mut Vec<CodeFile>) -> io::Result<()> {
    fn is_compatible(path: &PathBuf) -> (bool, &str) {
        if let Some(ext) = path.extension() {
            let extension = ext.to_str().unwrap();
            if CODE_EXTENSIONS.contains(&extension) {
                return (true, extension);
            }
        }
        (false, "skip")
    }
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
            walk_dir(&path, files)?;
        } else if is_suit {
            get_loc(&path, files, ext)?;
        }
    }
    Ok(())
}
pub fn print_loc(path: &PathBuf, top: Option<usize>) -> io::Result<()> {
    let mut print_buf: Vec<u8> = Vec::with_capacity(4096);
    let mut files: Vec<CodeFile> = Vec::new();
    walk_dir(path, &mut files)?;
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
fn get_loc(file: &PathBuf, files: &mut Vec<CodeFile>, ext_name: &str) -> io::Result<()> {
    fn fmt_name(buffer: &mut String, extension: &str, file_name: &str) -> std::fmt::Result {
        buffer.write_fmt(format_args!("{}...{}", &file_name[..5], extension))?;
        Ok(())
    }
    let content = fs::read_to_string(file)?;
    let loc = content.split("\n").filter(|x| !x.trim().is_empty()).count();
    let mut file_buf = String::new();
    let file_name: &str = file.file_name().unwrap().to_str().unwrap();
    if file_name.len() > MAX_NAME_LEN {
        fmt_name(&mut file_buf, ext_name, file_name).unwrap();
    } else {
        file_buf.write_str(file_name).unwrap();
    }
    files.push(CodeFile {
        name: file_buf,
        lines: loc,
    });
    Ok(())
}
