use std::path::PathBuf;

use human_bytes::human_bytes;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::utils::{self, find_files_by_predicate};

pub fn pack_project(
    project_base_path: impl Into<PathBuf>,
    include_script_files: bool,
) -> eyre::Result<()> {
    let project_base_path = project_base_path.into();
    let build_directory = utils::create_directory_if_not_exists(project_base_path.join("build"))?;

    let filename = build_directory.join("assets.zip");
    let file = std::fs::File::create(&filename)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    let files = find_files_by_predicate(&project_base_path, |entry| {
        if entry
            .path()
            .strip_prefix(&project_base_path)
            .ok()
            .expect("failed to strip path")
            .starts_with("build")
        {
            Some(false)
        } else {
            entry.path().extension().map(|ext| {
                if !include_script_files {
                    ext != "ts"
                } else {
                    true
                }
            })
        }
    })?
    .iter()
    .map(|path| {
        path.strip_prefix(&project_base_path)
            .expect("failed to strip path")
            .to_owned()
    })
    .collect::<Vec<_>>();

    tracing::info!("packing ({}) files:", files.len());

    for path in &files {
        let file = std::fs::File::open(project_base_path.join(path))?;
        tracing::info!(
            "- {:?}: {}",
            path,
            human_bytes(file.metadata()?.len() as f64)
        );
        zip.start_file(path.to_string_lossy(), options)?;
        std::io::copy(&mut std::io::BufReader::new(file), &mut zip)?;
    }

    let result_file = zip.finish()?;
    tracing::info!(
        "output: {:?} ({})",
        filename,
        human_bytes(result_file.metadata()?.len() as f64)
    );

    Ok(())
}
