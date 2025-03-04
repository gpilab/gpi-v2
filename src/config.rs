use std::{
    env,
    ffi::OsString,
    fs::{self, read_to_string},
    iter,
    path::{Path, PathBuf},
    process::Command,
};

use log::{info, warn};
use pyo3::{py_run, types::PyAnyMethods, PyResult, Python};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    venv_dir: PathBuf,
    nodes_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let user_dirs =
            directories::UserDirs::new().expect("application configuration folder is accessible");
        let config_dir = user_dirs.home_dir().join(".config/gpi");
        let config_file = config_dir.join("config.toml");

        match read_to_string(&config_file).map(|s| toml::from_str::<Config>(&s)) {
            Ok(Ok(c)) => {
                info!("Loaded config: {config_file:?}");
                c
            }
            Ok(Err(e)) => {
                panic!("Error reading config {config_file:?}:\n{e}");
            }
            _ => {
                // TODO: Prompt for venv path
                // TEMP: create default node location
                let nodes_dir = user_dirs.home_dir().join("gpi_default");
                let venv_dir = nodes_dir.join(".venv");

                print!("No configuration file found at {config_file:?} creating default config");
                if fs::read_dir(&nodes_dir).is_err() {
                    print!("creating default node directory at {nodes_dir:?}");
                    fs::create_dir(&nodes_dir)
                        .unwrap_or_else(|_| panic!("couldn't create default folder{nodes_dir:?}"));

                    let output = Command::new("python3")
                        .arg("-m")
                        .arg("venv")
                        .arg(&venv_dir)
                        .output()
                        .expect("failed to execute process");
                    info!("{output:?}");
                }

                print!("Creating default config file");
                let config = Config {
                    venv_dir,
                    nodes_dir,
                };
                std::fs::write(config_dir, toml::to_string_pretty(&config).unwrap())
                    .expect("Could not write config file");
                config
            }
        }
    }

    pub fn setup_environment(&self) {
        self.setup_python();
    }

    fn setup_python(&self) {
        //// Shell ENV variables
        {
            env::set_var("VIRTUAL_ENV", &self.venv_dir);
            env::set_var(
                "Path",
                prepend_env("PATH", self.venv_dir.join("bin")).unwrap(),
            );
            env::set_var("PYO3_PYTHON", self.venv_dir.join("bin/python"));

            // Set PYTHONPATH to appropriate paths in the venv directory
            // needed to address open pyo3 issue https://github.com/PyO3/pyo3/issues/1741
            if let Ok(paths) = glob::glob(
                self.venv_dir
                    .join("lib/python3*")
                    .to_str()
                    .expect("valid python virtual environment directory"),
            ) {
                let paths: Vec<_> = paths.filter_map(|p| p.ok()).collect();
                if paths.len() > 1 {
                    warn!("Multiple python versions detected in venv {:?}, this has not been tested. Unexpected results may occur",self.venv_dir)
                }
                paths.into_iter().for_each(|path| {
                    env::set_var(
                        "PYTHONPATH",
                        prepend_env("PYTHONPATH", path.join("site-packages"))
                            .unwrap()
                            .to_str()
                            .unwrap(),
                    );
                });
            }
        }

        //// PYO3 init
        pyo3::prepare_freethreaded_python();

        // Check python is working correctly, and display
        // venv location
        Python::with_gil(|py| {
            let list = 0u32;
            py_run!(
                py,
                list,
                r#"
import sys
print("Using python virtual environment:",sys.path[0])
"#
            );
        });

        // Configure python to close the program when
        // SIGINT (ctrl-c) is received. Otherwise ctrl-c won't work!
        let _ = Python::with_gil(|py| -> PyResult<()> {
            let signal = py.import("signal")?;
            signal
                .getattr("signal")?
                .call1((signal.getattr("SIGINT")?, signal.getattr("SIG_DFL")?))?;
            Ok(())
        });
    }

    //TODO: support mulitiple nodes directories
    pub fn nodes_dir(&self) -> &PathBuf {
        &self.nodes_dir
    }
}

/// Create a new env string that has the given value prepended
fn prepend_env<P: AsRef<Path>>(env: &str, p: P) -> Result<OsString, env::JoinPathsError> {
    let new_path = p.as_ref();
    if let Some(path) = env::var_os(env) {
        let old = env::split_paths(&path);
        Ok(env::join_paths(iter::once(new_path.to_owned()).chain(old))?)
    } else {
        Ok(new_path.into())
    }
}
