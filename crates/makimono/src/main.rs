use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use db_reader::version::detect_madara_db_version_for_db_path;

const DEFAULT_REPO: &str = "Mohiiit/makimono";

#[derive(Parser, Debug)]
#[command(name = "makimono")]
#[command(about = "Makimono: Madara DB Visualizer toolchain manager")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,

    /// Override where Makimono stores toolchains and state.
    /// If unset, uses platform-appropriate default.
    #[arg(long, env = "MAKIMONO_HOME")]
    home: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the visualizer for a given Madara base-path or RocksDB directory.
    Run {
        path: PathBuf,

        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Open the UI in a browser (best-effort)
        #[arg(long, action = clap::ArgAction::Set, default_value_t = default_open())]
        open: bool,

        /// Force a DB version instead of reading `.db-version`
        #[arg(long)]
        db_version: Option<u32>,

        /// Do not download anything; fail if the toolchain isn't installed.
        #[arg(long)]
        offline: bool,

        /// GitHub repo to download toolchains from, like owner/repo
        #[arg(long, default_value = DEFAULT_REPO)]
        repo: String,

        /// Explicit release tag to install/run (defaults to alias tag == db version)
        #[arg(long)]
        tag: Option<String>,
    },

    /// Manage installed toolchains
    Toolchain {
        #[command(subcommand)]
        cmd: ToolchainCmd,
    },

    /// Update the Makimono bootstrapper itself (best-effort)
    SelfUpdate {
        /// GitHub repo to download bootstrapper from
        #[arg(long, default_value = DEFAULT_REPO)]
        repo: String,

        /// Explicit bootstrapper release tag (e.g. makimono-0.1.0). If unset, uses latest.
        #[arg(long)]
        tag: Option<String>,

        /// Do not download anything.
        #[arg(long)]
        offline: bool,
    },
}

fn default_open() -> bool {
    cfg!(target_os = "macos") || cfg!(windows)
}

#[derive(Subcommand, Debug)]
enum ToolchainCmd {
    Install {
        /// Madara DB schema version (e.g. 9)
        db_version: u32,

        /// GitHub repo to download toolchain from
        #[arg(long, default_value = DEFAULT_REPO)]
        repo: String,

        /// Explicit release tag to install (defaults to alias tag == db version)
        #[arg(long)]
        tag: Option<String>,

        /// Do not download anything; fail if missing.
        #[arg(long)]
        offline: bool,
    },

    Uninstall {
        db_version: u32,
        #[arg(long)]
        tag: Option<String>,
    },

    List,
}

fn main() {
    let cli = Cli::parse();

    let home = cli
        .home
        .or_else(default_home)
        .unwrap_or_else(|| PathBuf::from(".makimono"));

    if let Err(e) = fs::create_dir_all(&home) {
        eprintln!(
            "error: failed to create makimono home {}: {e}",
            home.display()
        );
        std::process::exit(1);
    }

    let ctx = Ctx { home };

    let result = match cli.cmd {
        Commands::Run {
            path,
            host,
            port,
            open,
            db_version,
            offline,
            repo,
            tag,
        } => cmd_run(
            &ctx,
            &path,
            &host,
            port,
            open,
            db_version,
            offline,
            &repo,
            tag.as_deref(),
        ),
        Commands::Toolchain { cmd } => match cmd {
            ToolchainCmd::Install {
                db_version,
                repo,
                tag,
                offline,
            } => {
                let tag = tag
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| db_version.to_string());
                ensure_toolchain_installed(&ctx, &repo, db_version, &tag, offline).map(|_| ())
            }
            ToolchainCmd::Uninstall { db_version, tag } => {
                cmd_toolchain_uninstall(&ctx, db_version, tag.as_deref())
            }
            ToolchainCmd::List => cmd_toolchain_list(&ctx),
        },
        Commands::SelfUpdate { .. } => {
            // Keeping this as a stub for now; installing/updating a running binary is platform
            // specific and is better done via install scripts.
            Err(anyhow(
                "self update is not implemented yet; re-run install.sh/install.ps1",
            ))
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

struct Ctx {
    home: PathBuf,
}

fn default_home() -> Option<PathBuf> {
    let proj = ProjectDirs::from("xyz", "karnot", "makimono")?;
    Some(proj.data_dir().to_path_buf())
}

fn cmd_run(
    ctx: &Ctx,
    input: &Path,
    host: &str,
    port: u16,
    open: bool,
    forced_version: Option<u32>,
    offline: bool,
    repo: &str,
    tag: Option<&str>,
) -> Result<(), String> {
    let db_dir = resolve_rocksdb_dir(input)?;

    let detected = detect_madara_db_version_for_db_path(&db_dir);
    let db_version = forced_version.or(detected.version);

    let db_version = match db_version {
        Some(v) => v,
        None => {
            let fallback = highest_installed_toolchain(ctx).ok_or_else(|| {
                "could not detect DB version (missing .db-version) and no toolchains installed; pass --db-version".to_string()
            })?;
            eprintln!(
                "warning: could not detect DB version; defaulting to installed toolchain v{}",
                fallback
            );
            fallback
        }
    };

    let tag = match tag {
        Some(t) => t.to_string(),
        None => fs::read_to_string(current_tag_path(ctx, db_version))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| db_version.to_string()),
    };

    let toolchain_bin = ensure_toolchain_installed(ctx, repo, db_version, &tag, offline)?;

    let index_path = default_index_path(ctx, &db_dir);
    if let Some(parent) = index_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    eprintln!("db: {}", db_dir.display());
    if let Some(v) = detected.version {
        let src = detected
            .source_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(unknown)".to_string());
        eprintln!("detected .db-version: {v} (from {src})");
    } else if let Some(err) = detected.error {
        eprintln!(".db-version note: {err}");
    }

    let mut child = Command::new(&toolchain_bin)
        .arg("--db-path")
        .arg(&db_dir)
        .arg("--index-path")
        .arg(&index_path)
        .arg("--host")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to start toolchain: {e}"))?;

    if open {
        let url = format!("http://{host}:{port}");
        let _ = open_url(&url);
    }

    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for toolchain: {e}"))?;
    if !status.success() {
        return Err(format!("toolchain exited with status {status}"));
    }
    Ok(())
}

fn cmd_toolchain_list(ctx: &Ctx) -> Result<(), String> {
    let root = toolchains_dir(ctx);
    if !root.exists() {
        println!("(no toolchains installed)");
        return Ok(());
    }

    let mut versions: Vec<u32> = Vec::new();
    for ent in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let ent = ent.map_err(|e| e.to_string())?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if let Ok(v) = name.parse::<u32>() {
            versions.push(v);
        }
    }
    versions.sort_unstable();

    for v in versions {
        let cur = current_tag_path(ctx, v);
        let current = fs::read_to_string(&cur).ok().map(|s| s.trim().to_string());
        println!(
            "dbv{v} current={}",
            current.unwrap_or_else(|| "(none)".into())
        );

        let vdir = toolchains_dir(ctx).join(v.to_string());
        let entries = fs::read_dir(&vdir).map_err(|e| e.to_string())?;
        for ent in entries {
            let ent = ent.map_err(|e| e.to_string())?;
            if ent.file_type().map_err(|e| e.to_string())?.is_dir() {
                let tag = ent.file_name().to_string_lossy().to_string();
                println!("  - {tag}");
            }
        }
    }

    Ok(())
}

fn cmd_toolchain_uninstall(ctx: &Ctx, db_version: u32, tag: Option<&str>) -> Result<(), String> {
    let vdir = toolchains_dir(ctx).join(db_version.to_string());
    if !vdir.exists() {
        return Ok(());
    }

    if let Some(tag) = tag {
        let tdir = vdir.join(tag);
        if tdir.exists() {
            fs::remove_dir_all(&tdir).map_err(|e| e.to_string())?;
        }
    } else {
        fs::remove_dir_all(&vdir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn highest_installed_toolchain(ctx: &Ctx) -> Option<u32> {
    let root = toolchains_dir(ctx);
    let mut best: Option<u32> = None;
    let entries = fs::read_dir(&root).ok()?;
    for ent in entries {
        let ent = ent.ok()?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if let Ok(v) = name.parse::<u32>() {
            best = Some(best.map(|b| b.max(v)).unwrap_or(v));
        }
    }
    best
}

fn resolve_rocksdb_dir(input: &Path) -> Result<PathBuf, String> {
    let abs = abs_path(input);
    if !abs.is_dir() {
        return Err(format!("path is not a directory: {}", abs.display()));
    }

    if looks_like_rocksdb_dir(&abs) {
        return Ok(abs);
    }

    let candidate = abs.join("db");
    if looks_like_rocksdb_dir(&candidate) {
        return Ok(candidate);
    }

    Err(format!(
        "{} doesn't look like a RocksDB directory (no CURRENT / *.sst)",
        abs.display()
    ))
}

fn looks_like_rocksdb_dir(p: &Path) -> bool {
    p.join("CURRENT").is_file()
        || fs::read_dir(p)
            .ok()
            .and_then(|mut it| {
                it.find(|e| {
                    e.as_ref()
                        .ok()
                        .map(|x| x.path().extension() == Some(OsStr::new("sst")))
                        .unwrap_or(false)
                })
            })
            .is_some()
}

fn abs_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

fn toolchains_dir(ctx: &Ctx) -> PathBuf {
    ctx.home.join("toolchains")
}

fn current_tag_path(ctx: &Ctx, db_version: u32) -> PathBuf {
    toolchains_dir(ctx)
        .join(db_version.to_string())
        .join("current")
}

fn default_index_path(ctx: &Ctx, db_dir: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(db_dir.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let hex = hex::encode(digest);
    ctx.home
        .join("state")
        .join("index")
        .join(format!("{hex}.db"))
}

fn ensure_toolchain_installed(
    ctx: &Ctx,
    repo: &str,
    db_version: u32,
    tag: &str,
    offline: bool,
) -> Result<PathBuf, String> {
    let bin = toolchain_bin_path(ctx, db_version, tag);
    if bin.exists() {
        set_current_tag(ctx, db_version, tag)?;
        return Ok(bin);
    }

    if offline {
        return Err(format!(
            "toolchain not installed for dbv{db_version} tag {tag} (offline mode)"
        ));
    }

    download_and_install_toolchain(ctx, repo, db_version, tag)?;

    if !bin.exists() {
        return Err("toolchain installation finished but binary not found".to_string());
    }

    set_current_tag(ctx, db_version, tag)?;
    Ok(bin)
}

fn set_current_tag(ctx: &Ctx, db_version: u32, tag: &str) -> Result<(), String> {
    let path = current_tag_path(ctx, db_version);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, format!("{tag}\n")).map_err(|e| e.to_string())
}

fn toolchain_bin_path(ctx: &Ctx, db_version: u32, tag: &str) -> PathBuf {
    let exe = if cfg!(windows) {
        "makimono-viz.exe"
    } else {
        "makimono-viz"
    };
    toolchains_dir(ctx)
        .join(db_version.to_string())
        .join(tag)
        .join("bin")
        .join(exe)
}

fn download_and_install_toolchain(
    ctx: &Ctx,
    repo: &str,
    db_version: u32,
    tag: &str,
) -> Result<(), String> {
    let downloads = ctx.home.join("downloads");
    fs::create_dir_all(&downloads).map_err(|e| e.to_string())?;

    let (os, arch) = current_platform();
    let asset = if cfg!(windows) {
        format!("makimono-viz-{tag}-{os}-{arch}.zip")
    } else {
        format!("makimono-viz-{tag}-{os}-{arch}.tar.gz")
    };

    let base = format!("https://github.com/{repo}/releases/download/{tag}");
    let url = format!("{base}/{asset}");
    let sums_url = format!("{base}/SHA256SUMS");

    eprintln!("downloading toolchain: {url}");

    let sums = http_get_bytes(&sums_url)?;
    let expected = parse_sha256sums(&sums, &asset)
        .ok_or_else(|| format!("SHA256SUMS missing entry for {asset}"))?;

    let archive_path = downloads.join(&asset);
    let bytes = http_get_bytes(&url)?;
    fs::write(&archive_path, &bytes).map_err(|e| e.to_string())?;

    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {asset}: expected {expected} got {actual}"
        ));
    }

    let install_dir = toolchains_dir(ctx).join(db_version.to_string()).join(tag);
    fs::create_dir_all(&install_dir).map_err(|e| e.to_string())?;

    if cfg!(windows) {
        install_zip(&archive_path, &install_dir)?;
    } else {
        install_targz(&archive_path, &install_dir)?;
    }

    Ok(())
}

fn install_targz(archive: &Path, dest: &Path) -> Result<(), String> {
    let f = fs::File::open(archive).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(f);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn install_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let f = fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    zip.extract(dest).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn install_zip(_archive: &Path, _dest: &Path) -> Result<(), String> {
    Err("zip extraction not supported on this platform".to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn parse_sha256sums(contents: &[u8], asset: &str) -> Option<String> {
    let text = std::str::from_utf8(contents).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "<sha256>  <filename>"
        let mut parts = line.split_whitespace();
        let sum = parts.next()?;
        let name = parts.next()?;
        if name == asset {
            return Some(sum.to_string());
        }
    }
    None
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .set("User-Agent", "makimono")
        .call()
        .map_err(|e| format!("http error: {e}"))?;

    let mut r = resp.into_reader();
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn current_platform() -> (String, String) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "unknown"
    };

    (os.to_string(), arch.to_string())
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", url])
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("unsupported platform".to_string())
}

fn anyhow(msg: &str) -> String {
    msg.to_string()
}
