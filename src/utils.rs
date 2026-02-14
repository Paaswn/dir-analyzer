use std::{fs, io, path::Path};

pub trait Processor {
    type Custom;
    fn process_file(&mut self, path: &Path, pb: &indicatif::ProgressBar) -> Self::Custom;
    fn is_dir_compatible(&self, _path: &Path) -> bool {
        true
    }
    fn is_file_compatible(&self, _path: &Path) -> bool {
        true
    }
}
pub fn walk_dir(
    path: &Path,
    processor: &mut impl Processor,
    pb: &indicatif::ProgressBar,
) -> io::Result<()> {
    let dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };
    for file in dir {
        let file = file?;
        let metadata = file.metadata()?;
        let objec_path = file.path();
        if metadata.is_dir() {
            if processor.is_dir_compatible(&objec_path) {
                walk_dir(&objec_path, processor, pb)?;
            };
        } else if processor.is_file_compatible(&objec_path) {
            processor.process_file(&objec_path, pb);
        }
    }
    Ok(())
}
