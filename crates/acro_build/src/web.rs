use std::{
    collections::HashSet,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn get_esbuild_binary_or_download() -> eyre::Result<PathBuf> {
    let mut esbuild_download_path = std::env::current_exe()?;
    esbuild_download_path.pop();
    esbuild_download_path.push("esbuild");

    if cfg!(windows) {
        esbuild_download_path.push(".exe");
    }

    if esbuild_download_path.exists() {
        return Ok(esbuild_download_path);
    }

    tracing::info!("esbuild binary not found, downloading...");

    // TODO: select different esbuild binary based on platform
    let url = "https://registry.npmjs.org/@esbuild/linux-x64/-/linux-x64-0.24.0.tgz";
    let res = ureq::get(url).call()?.into_reader();

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(res));
    let entries = archive.entries()?;

    let mut file = std::fs::File::create(&esbuild_download_path)?;

    for entry in entries {
        let mut entry = entry?;
        if entry.path()?.starts_with("package/bin/") {
            std::io::copy(&mut entry, &mut file)?;
            break;
        }
    }

    std::fs::set_permissions(
        &esbuild_download_path,
        std::fs::Permissions::from_mode(0o755),
    )?;

    Ok(esbuild_download_path)
}

fn create_directory_if_not_exists(path: impl AsRef<Path>) -> eyre::Result<PathBuf> {
    let path = path.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(path.into())
}

fn create_entry_file_content(
    project_base_path: &Path,
    files: impl Iterator<Item = PathBuf>,
) -> eyre::Result<String> {
    let mut content = r#"import { init } from "jsr:@acro/core";
init();
// deno-lint-ignore-file no-explicit-any
"#
    .to_string();
    for (index, file) in files.enumerate() {
        content.push_str(&format!(
            "import * as file_{index} from \"../{}\";\n",
            file.strip_prefix(project_base_path)?.display()
        ));
        content.push_str(&format!("(file_{index} as any).init?.();\n",));
    }
    Ok(content)
}

fn find_script_files(project_base_path: &Path) -> eyre::Result<Vec<PathBuf>> {
    Ok(walkdir::WalkDir::new(project_base_path)
        .into_iter()
        .filter(|entry| {
            entry
                .as_ref()
                .map(|entry| {
                    entry
                        .path()
                        .extension()
                        .map(|ext| ext == "ts")
                        .unwrap_or(false)
                })
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

fn generate_aliases() -> eyre::Result<Vec<String>> {
    let lib_dir = std::env::current_dir()?.join("lib");

    const MODULES: &[&str] = &["core", "math", "input", "ui"];

    let mut aliases = vec![];
    for module in MODULES {
        aliases.push(format!(
            "--alias:jsr:@acro/{}={}",
            module,
            lib_dir.join(format!("{}/mod.ts", module)).display()
        ));
    }

    Ok(aliases)
}

pub fn build_javascript_bundle(project_base_path: impl Into<PathBuf>) -> eyre::Result<()> {
    let project_base_path = project_base_path.into();
    let esbuild_path = get_esbuild_binary_or_download()?;
    let build_directory = create_directory_if_not_exists(project_base_path.join("build"))?;

    let entry_file = std::fs::canonicalize(&build_directory)?.join("entry.ts");
    let files = find_script_files(&project_base_path)?;
    std::fs::write(
        &entry_file,
        create_entry_file_content(&project_base_path, files.into_iter())?,
    )?;

    let mut args = vec![
        entry_file.to_string_lossy().to_string(),
        "--bundle".to_string(),
        "--minify".to_string(),
        "--define:import.meta.platform=\"web\"".to_string(),
        "--log-override:import-is-undefined=silent".to_string(),
        format!(
            "--outfile={}",
            build_directory.join("bundle.js").to_string_lossy()
        ),
    ];
    args.append(&mut generate_aliases()?);

    tracing::info!("running esbuild: {args:?}");

    // let path_as_string = esbuild_path.to_string_lossy().to_string();
    // tracing::info!("command: {} {}", path_as_string, args.join(" "));

    let mut child = Command::new(&esbuild_path)
        .current_dir(&project_base_path)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    child.wait()?;

    Ok(())
}
