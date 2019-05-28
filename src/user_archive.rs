use failure::Error;
use std::path::{Path, PathBuf};
use crate::pins::BackingStore;
use std::io::Write;


fn generate_zip_file(username: &str) -> Result<String, Error> {
    let buf : Vec<u8> = vec!();
    let w = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(w); 

    let file_options = zip::write::FileOptions::default();

    zip.add_directory("recense_data", file_options)?;

    let dir_str = BackingStore::pin_directory(username);
    let dir_path = Path::new(&dir_str);

    if !dir_path.exists() {
        bail!("No data to archive for user {}", username);
    }
    
    let archive_filename = format!("recense_archive_{}.zip", username);

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

        let filename = String::from(filename_opt.unwrap().to_str().unwrap());

        if filename == archive_filename {
            continue;
        }
      
        let bytes = std::fs::read(&Path::new(&format!("{}/{}", dir_path.to_str().unwrap(), filename)))?;

        zip.start_file(format!("recense_data/{}", filename), file_options)?;

        zip.write_all(&bytes)?;
    }

    let zip_data = zip.finish()?;
  
    let mut out_path = PathBuf::from(dir_path);
    out_path.push(&archive_filename);

    std::fs::write(&out_path, zip_data.get_ref())?;
    Ok(archive_filename)
}

pub fn generate_archive_for_user(username: String, callback: fn(Result<String, Error>)) {

    std::thread::spawn(move || {
        callback(generate_zip_file(&username));
    });
}
