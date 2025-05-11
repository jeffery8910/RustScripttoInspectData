// src/dep_check.rs
use duct::cmd;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyStatus {
    // << pub
    Ok(PathBuf),
    NotFound,
    PackageMissing(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Dependencies {
    // << pub
    pub r_script: DependencyStatus,
    pub python_interpreter: DependencyStatus,
    pub r_readr_pkg: DependencyStatus,
    pub r_dplyr_pkg: DependencyStatus,
    pub py_pandas_pkg: DependencyStatus,
    pub py_duckdb_pkg: DependencyStatus,
}

fn check_executable(name: &str) -> DependencyStatus {
    match which(name) {
        Ok(path) => DependencyStatus::Ok(path),
        Err(_) => DependencyStatus::NotFound,
    }
}

fn check_r_package(r_executable: &PathBuf, package_name: &str) -> DependencyStatus {
    let expr = format!(
        "if (!requireNamespace('{}', quietly = TRUE)) {{ stop('Package {} not found') }}",
        package_name, package_name
    );
    match cmd!(r_executable, "-e", expr)
        .stdout_null()
        .stderr_null()
        .run()
    {
        Ok(output) if output.status.success() => DependencyStatus::Ok(PathBuf::from(package_name)),
        Ok(output) => {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            if err_msg.contains(&format!("Package {} not found", package_name))
                || err_msg.contains("there is no package called")
            {
                DependencyStatus::PackageMissing(package_name.to_string())
            } else {
                DependencyStatus::Error(format!(
                    "R pkg '{}' check failed: {}",
                    package_name, err_msg
                ))
            }
        }
        Err(e) => {
            DependencyStatus::Error(format!("Failed to run R for pkg '{}': {}", package_name, e))
        }
    }
}

fn check_python_package(python_executable: &PathBuf, package_name: &str) -> DependencyStatus {
    let script = format!("import {}", package_name);
    match cmd!(python_executable, "-c", script)
        .stdout_null()
        .stderr_null()
        .run()
    {
        Ok(output) if output.status.success() => DependencyStatus::Ok(PathBuf::from(package_name)),
        Ok(output) => {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            if err_msg.contains("No module named") || err_msg.contains("ModuleNotFoundError") {
                DependencyStatus::PackageMissing(package_name.to_string())
            } else {
                DependencyStatus::Error(format!(
                    "Python pkg '{}' check failed: {}",
                    package_name, err_msg
                ))
            }
        }
        Err(e) => DependencyStatus::Error(format!(
            "Failed to run Python for pkg '{}': {}",
            package_name, e
        )),
    }
}

pub static CHECKED_DEPS: Lazy<Dependencies> = Lazy::new(|| {
    // << pub
    let r_script_status = check_executable("Rscript");
    // Try 'python' first, then 'python3' if 'python' is not found.
    let python_status = match check_executable("python") {
        DependencyStatus::Ok(path) => DependencyStatus::Ok(path),
        _ => check_executable("python3"), // Fallback to python3
    };

    let r_readr = match &r_script_status {
        DependencyStatus::Ok(r_path) => check_r_package(r_path, "readr"),
        _ => DependencyStatus::NotFound,
    };
    let r_dplyr = match &r_script_status {
        DependencyStatus::Ok(r_path) => check_r_package(r_path, "dplyr"),
        _ => DependencyStatus::NotFound,
    };
    let py_pandas = match &python_status {
        DependencyStatus::Ok(py_path) => check_python_package(py_path, "pandas"),
        _ => DependencyStatus::NotFound,
    };
    let py_duckdb = match &python_status {
        DependencyStatus::Ok(py_path) => check_python_package(py_path, "duckdb"),
        _ => DependencyStatus::NotFound,
    };

    Dependencies {
        r_script: r_script_status,
        python_interpreter: python_status,
        r_readr_pkg: r_readr,
        r_dplyr_pkg: r_dplyr,
        py_pandas_pkg: py_pandas,
        py_duckdb_pkg: py_duckdb,
    }
});

// Impl blocks for DependencyStatus and Dependencies
impl DependencyStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, DependencyStatus::Ok(_))
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        if let DependencyStatus::Ok(path) = self {
            Some(path)
        } else {
            None
        }
    }
}

impl Dependencies {
    pub fn all_ok(&self) -> bool {
        self.r_script.is_ok()
            && self.python_interpreter.is_ok()
            && self.r_readr_pkg.is_ok()
            && self.r_dplyr_pkg.is_ok()
            && self.py_pandas_pkg.is_ok()
            && self.py_duckdb_pkg.is_ok()
    }
}
