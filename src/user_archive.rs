use crate::pins::BackingStore;
use failure::Error;
use std::io::Write;
use std::path::Path;

pub fn generate_archive_for_user(username: String) -> Result<Vec<u8>, Error> {
    let dir_str = BackingStore::pin_directory(&username);
    let dir_path = Path::new(&dir_str);

    if !dir_path.exists() {
        bail!("No data to archive for user {}", username);
    }

    let mut buf: Vec<u8> = vec![];
    {
        let w = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(w);

        let file_options = zip::write::FileOptions::default();

        zip.add_directory("recense_user_archive", file_options)?;

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

            let bytes = std::fs::read(&Path::new(&format!(
                "{}/{}",
                dir_path.to_str().unwrap(),
                filename
            )))?;

            zip.start_file(format!("recense_user_archive/{}", filename), file_options)?;
            zip.write_all(&bytes)?;
        }
        zip.finish()?;
    }
    Ok(buf)
}
