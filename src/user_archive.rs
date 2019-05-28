use crate::pins::BackingStore;
use failure::Error;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn generate_archive_for_user(username: String) -> Result<PathBuf, Error> {
    let dir_str = BackingStore::pin_directory(&username);
    let dir_path = Path::new(&dir_str);

    if !dir_path.exists() {
        bail!("No data to archive for user {}", username);
    }

    let archive_filename = format!("recense_archive_{}.zip", username);
    let mut out_path = PathBuf::from(dir_path);
    out_path.push(&archive_filename);

    if out_path.exists() {
        let _ = std::fs::remove_file(&out_path);
    }

    let buf: Vec<u8> = vec![];
    let w = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(w);

    let file_options = zip::write::FileOptions::default();

    zip.add_directory("recense_data", file_options)?;

    for file in std::fs::read_dir(dir_path)? {
        if !file.is_ok() {
            continue;
        }
        let file = file.as_ref().unwrap();
        let path = file.path();
        let filename_opt = path.as_path().file_name().clone();

        if filename_opt.is_none() {
            continue;
        }

        let filename = filename_opt.unwrap().to_str().unwrap().to_string();

        if filename == archive_filename {
            continue;
        }

        let bytes = std::fs::read(&Path::new(&format!(
            "{}/{}",
            dir_path.to_str().unwrap(),
            filename
        )))?;

        zip.start_file(format!("recense_data/{}", filename), file_options)?;

        zip.write_all(&bytes)?;
    }

    let zip_data = zip.finish()?;

    std::fs::write(&out_path, zip_data.get_ref())?;
    Ok(PathBuf::from(out_path))
}
