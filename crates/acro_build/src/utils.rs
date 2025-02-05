use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

pub fn create_directory_if_not_exists(path: impl AsRef<Path>) -> eyre::Result<PathBuf> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(path.into())
}

pub fn find_files_by_predicate(
    base: &Path,
    predicate: impl Fn(&DirEntry) -> Option<bool>,
) -> eyre::Result<Vec<PathBuf>> {
    Ok(WalkDir::new(base)
        .into_iter()
        .filter(|entry| {
            entry
                .as_ref()
                .map(|entry| predicate(entry).unwrap_or(false))
                .unwrap_or(false)
        })
        .map(|result| match result {
            Ok(entry) => Ok(entry.path().into()),
            Err(err) => Err(err.into()),
        })
        .collect::<eyre::Result<HashSet<_>>>()?
        .into_iter()
        .collect::<Vec<PathBuf>>())
}
