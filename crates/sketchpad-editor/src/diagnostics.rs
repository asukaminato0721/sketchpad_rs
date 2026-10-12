//! Local diagnostics for failures before the first frame appears.
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

struct Diagnostics {
    file: Mutex<Option<File>>,
    started: Instant,
}

static DIAGNOSTICS: OnceLock<Diagnostics> = OnceLock::new();

impl log::Log for Diagnostics {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            self.write(&format!(
                "{} {}: {}",
                record.level(),
                record.target(),
                record.args()
            ));
        }
    }

    fn flush(&self) {
        self.sync();
    }
}

impl Diagnostics {
    fn write(&self, message: &str) {
        let line = format!(
            "[+{:.3}s] {message}\n",
            self.started.elapsed().as_secs_f64()
        );
        // Write each record directly. A process crash must not discard a buffer.
        let mut guard = self.file.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(file) = guard.as_mut() {
            let _ = file.write_all(line.as_bytes());
        }
        let _ = std::io::stderr().write_all(line.as_bytes());
    }

    fn sync(&self) {
        let guard = self.file.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(file) = guard.as_ref() {
            let _ = file.sync_all();
        }
    }
}

fn open_log(directories: &[PathBuf], name: &str) -> std::io::Result<(PathBuf, File)> {
    let mut last_error = std::io::Error::other("No log directory is available");
    for directory in directories {
        let path = directory.join(name);
        match std::fs::create_dir_all(directory)
            .and_then(|()| OpenOptions::new().write(true).create_new(true).open(&path))
        {
            Ok(file) => return Ok((path, file)),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn log_directories(executable: Option<&Path>) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    if let Some(parent) = executable.and_then(Path::parent) {
        directories.push(parent.join("logs"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        directories.push(PathBuf::from(local).join("Sketchpad RS").join("logs"));
    }
    directories.push(std::env::temp_dir().join("sketchpad-rs").join("logs"));
    directories
}

pub(crate) fn init() {
    if DIAGNOSTICS.get().is_some() {
        return;
    }
    let executable = std::env::current_exe().ok();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let name = format!(
        "sketchpad-{}-{}.log",
        timestamp.as_millis(),
        std::process::id()
    );
    let opened = open_log(&log_directories(executable.as_deref()), &name);
    let (path, file) = match opened {
        Ok((path, file)) => (Some(path), Some(file)),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "Cannot create a diagnostic log: {error}");
            (None, None)
        }
    };
    let diagnostics = DIAGNOSTICS.get_or_init(|| Diagnostics {
        file: Mutex::new(file),
        started: Instant::now(),
    });
    if log::set_logger(diagnostics).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Use the file directly, even if a host already installed a logger.
        diagnostics.write(&format!("PANIC: {info}"));
        diagnostics.write(&format!(
            "Backtrace:\n{}",
            std::backtrace::Backtrace::force_capture()
        ));
        diagnostics.sync();
        previous_hook(info);
    }));
    log::info!(
        "Sketchpad RS {}; pid={}; unix_time_ms={}",
        env!("CARGO_PKG_VERSION"),
        std::process::id(),
        timestamp.as_millis()
    );
    log::info!("Log file: {path:?}");
    log::info!(
        "OS={}; architecture={}; executable={executable:?}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    log::info!("WGPU_BACKEND={:?}", std::env::var_os("WGPU_BACKEND"));
    if let Some(parent) = executable.as_deref().and_then(Path::parent)
        && let Ok(build) = std::fs::read_to_string(parent.join("BUILD-INFO.json"))
    {
        log::info!("Build information: {}", build.trim());
    }
    // The OS query must not delay window creation.
    let _ = std::thread::Builder::new()
        .name("system-diagnostics".into())
        .spawn(|| {
            #[cfg(windows)]
            let output = {
                use std::os::windows::process::CommandExt;
                std::process::Command::new("cmd.exe")
                    .args(["/D", "/C", "ver"])
                    .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
                    .output()
            };
            #[cfg(not(windows))]
            let output = std::process::Command::new("uname")
                .args(["-s", "-r", "-m"])
                .output();
            match output {
                Ok(output) => log::info!(
                    "System version: {}",
                    String::from_utf8_lossy(&output.stdout).trim()
                ),
                Err(error) => log::warn!("Cannot query the system version: {error}"),
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_fall_back_and_do_not_replace_previous_runs() {
        let root = std::env::temp_dir().join(format!(
            "sketchpad-diagnostics-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let blocked = root.join("file");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let destination = root.join("logs");
        let (path, file) = open_log(&[blocked, destination.clone()], "session.log").unwrap();
        let diagnostics = Diagnostics {
            file: Mutex::new(Some(file)),
            started: Instant::now(),
        };
        diagnostics.write("last completed startup stage");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("last completed startup stage"));
        assert!(open_log(&[destination], "session.log").is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), content);
        drop(diagnostics);
        std::fs::remove_dir_all(root).unwrap();
    }
}
