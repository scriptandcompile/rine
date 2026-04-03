use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use goblin::Object;
use thiserror::Error;
use tracing::{error, info};

const IMAGE_FILE_MACHINE_I386: u16 = 0x014c;

#[derive(Parser, Debug)]
#[command(
    name = "rine32",
    version,
    about = "rine32 - 32-bit Windows PE runtime launcher",
    override_usage = "rine32 <EXE_PATH> [EXE_ARGS]..."
)]
struct Cli {
    /// Path to the Windows .exe to run.
    exe_path: Option<PathBuf>,

    /// Arguments to pass to the Windows executable.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    exe_args: Vec<String>,
}

#[derive(Debug, Error)]
enum Run32Error {
    #[error("failed to read executable `{path}`: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("failed to parse executable `{path}`: {source}")]
    Parse {
        source: goblin::error::Error,
        path: PathBuf,
    },

    #[error("`{path}` is not a PE executable")]
    NotPe { path: PathBuf },

    #[error("`{path}` is not a 32-bit PE (machine=0x{machine:04x})")]
    NotPe32 { path: PathBuf, machine: u16 },

    #[error(
        "this rine32 binary was built for a non-32-bit host architecture ({host}); build rine32 for x86 Linux"
    )]
    WrongHost { host: &'static str },

    #[error("32-bit runtime loader is not implemented yet")]
    NotImplemented,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    let Some(exe_path) = cli.exe_path else {
        let _ = Cli::command().print_help();
        return ExitCode::FAILURE;
    };

    match run(&exe_path, &cli.exe_args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(exe_path: &Path, exe_args: &[String]) -> Result<i32, Run32Error> {
    let resolved = resolve_exe_path(exe_path);
    ensure_pe32(&resolved)?;

    info!(
        exe = %resolved.display(),
        arg_count = exe_args.len(),
        "validated 32-bit executable"
    );

    if cfg!(target_pointer_width = "32") {
        return Err(Run32Error::NotImplemented);
    }

    Err(Run32Error::WrongHost {
        host: std::env::consts::ARCH,
    })
}

fn resolve_exe_path(path: &Path) -> PathBuf {
    if !path.exists() && path.extension().is_none() {
        path.with_extension("exe")
    } else {
        path.to_path_buf()
    }
}

fn ensure_pe32(path: &Path) -> Result<(), Run32Error> {
    let bytes = std::fs::read(path).map_err(|source| Run32Error::Io {
        source,
        path: path.to_path_buf(),
    })?;

    let pe = match Object::parse(&bytes).map_err(|source| Run32Error::Parse {
        source,
        path: path.to_path_buf(),
    })? {
        Object::PE(pe) => pe,
        _ => {
            return Err(Run32Error::NotPe {
                path: path.to_path_buf(),
            });
        }
    };

    let machine = pe.header.coff_header.machine;
    if machine != IMAGE_FILE_MACHINE_I386 {
        return Err(Run32Error::NotPe32 {
            path: path.to_path_buf(),
            machine,
        });
    }

    Ok(())
}
