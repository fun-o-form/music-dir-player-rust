/// Utilities for reading directories and music files from disk
pub mod file_utils {
    use std::ffi::OsStr;
    use std::fs;
    use std::fs::DirEntry;
    use std::io;
    use std::path::PathBuf;

    /// Gets a collection of all subdirectories in the specified starting directory.
    pub fn sub_directories(starting_dir: &str) -> io::Result<Vec<String>> {
        let dir = PathBuf::from(&starting_dir);
        let mut subdirs = Vec::new();

        for dir_entry_res in fs::read_dir(dir)? {
            match dir_entry_res {
                Ok(maybe_dir) => {
                    if maybe_dir.metadata()?.is_dir() {
                        match maybe_dir.file_name().to_str() {
                            Some(final_dir_name) => subdirs.push(final_dir_name.to_string()),
                            None => continue,
                        }
                    }
                },
                Err(_) => continue,
            }
        }
        Ok(subdirs)
    }

    /// Allows getting a list of all the music files in the specified directory
    /// # Parameters
    /// dir_to_scan = The absolute or relative path to the directory you want to query.
    /// recursive = If false, this method only returns music files found in the specified directory.
    ///             If true, this method also includes files found in subdirectories.
    pub fn list_music_files(dir_to_scan: &str, recursive: bool) -> io::Result<Vec<PathBuf>> {
        let directory = PathBuf::from(&dir_to_scan);
        _list_music_files(&directory, recursive)
    }

    /// Annoyingly, rust doesn't allow method overrides. So we keep our public method with the nice
    /// name and use this underscore version for our private version.
    fn _list_music_files(dir_to_scan: &PathBuf, recursive: bool) -> io::Result<Vec<PathBuf>> {
        let mut mp3_files = Vec::new();
        let mut had_recursive_dirs: bool = false;

        // Read all files in the directory
        if dir_to_scan.is_dir() {
            for entry in fs::read_dir(&dir_to_scan)? {
                let entry: DirEntry = entry?;
                let path: PathBuf = entry.path();

                if recursive && entry.file_type()?.is_dir() {
                    // if we were asked to recursively list files and our directory contains a
                    // subdirectory then get all the files from the subdirectory
                    mp3_files.append(&mut _list_music_files(&path, true)?);
                    // Just for logging purposes, note that the file count from this directory
                    // included recursive directories
                    had_recursive_dirs = true;
                } else {
                    // This is just a file. Check if it is a supported music file
                    if is_supported_audio_file(&path) {
                        mp3_files.push(path);
                    }
                }
            }
        }

        // print out the count of music files in this directory, and an indicator if some of that
        // count came from subdirectories
        let mut recursively: &str = "";
        if had_recursive_dirs {
            recursively = "recursively";
        }
        println!("Found {} music files in {} {}", mp3_files.len(), dir_to_scan.display(), recursively);

        Ok(mp3_files)
    }

    /// Returns true if the specified file is a music file this app can play back, false otherwise.
    /// Note, supported file types are those supported by rodio. Currently, that means mp3, wav,
    /// flac, and vorbis (ogg).
    fn is_supported_audio_file(file_path: &PathBuf) -> bool {
        match file_path.extension().and_then(OsStr::to_str) {
            Some(ext) => {
                // note, start with the most common file extensions so we don't waste cycles checking for
                // less common extensions
                return ext.eq_ignore_ascii_case("mp3") || ext.eq_ignore_ascii_case("flac") ||
                    ext.eq_ignore_ascii_case("wav") || ext.eq_ignore_ascii_case("ogg");
            }
            None => false,
        }
    }
}