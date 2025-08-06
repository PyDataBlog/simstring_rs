use std::env;
use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore]
fn run_python_tests() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let venv_dir = project_root.join("target").join("pytest_venv");
    let python_executable_path;
    let maturin_executable_path;
    let pytest_executable_path;

    if cfg!(windows) {
        python_executable_path = venv_dir.join("Scripts").join("python.exe");
        maturin_executable_path = venv_dir.join("Scripts").join("maturin.exe");
        pytest_executable_path = venv_dir.join("Scripts").join("pytest.exe");
    } else {
        python_executable_path = venv_dir.join("bin").join("python");
        maturin_executable_path = venv_dir.join("bin").join("maturin");
        pytest_executable_path = venv_dir.join("bin").join("pytest");
    }

    // Create virtual environment if it doesn't exist
    if !venv_dir.exists() {
        println!("--- Creating Python virtual environment in `target` directory ---");
        let venv_cmd = Command::new("python3")
            .args(["-m", "venv", venv_dir.to_str().unwrap()])
            .status()
            .expect("Failed to execute `python3 -m venv`. Is `python3` in your PATH?");
        if !venv_cmd.success() {
            panic!("Failed to create python venv. Is `python3` and `venv` module installed?");
        }
    }

    // Install dependencies into the virtual environment
    println!("--- Installing maturin and pytest using pip ---");
    let pip_cmd = Command::new(python_executable_path.to_str().unwrap())
        .args(["-m", "pip", "install", "-U", "pip", "maturin", "pytest"])
        .status()
        .expect("Failed to run pip install. Is the venv corrupted?");
    if !pip_cmd.success() {
        panic!("Failed to install maturin and pytest in the venv.");
    }

    // Build the python wheel
    println!("--- Building wheel with maturin ---");
    let maturin_build_cmd = Command::new(maturin_executable_path.to_str().unwrap())
        .args(["build", "--release", "--out", "target/wheels"])
        .status()
        .expect("Failed to run `maturin build`.");
    if !maturin_build_cmd.success() {
        panic!("`maturin build` failed.");
    }

    // Install the generated Python wheel
    println!("--- Installing wheel with pip ---");
    let wheel_path = std::fs::read_dir("target/wheels")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .path()
                .extension()
                .map_or_else(|| false, |ext| ext == "whl")
        })
        .unwrap()
        .path();

    let pip_install_wheel_cmd = Command::new(python_executable_path.to_str().unwrap())
        .args([
            "-m",
            "pip",
            "install",
            wheel_path.to_str().unwrap(),
            "--force-reinstall",
        ])
        .status()
        .expect("Failed to install wheel.");
    if !pip_install_wheel_cmd.success() {
        panic!("Failed to install wheel.");
    }

    // Run pytest
    println!("--- Running pytest ---");
    let pytest_cmd = Command::new(pytest_executable_path.to_str().unwrap())
        .arg("tests/python/")
        .status()
        .expect("Failed to run `pytest`.");
    if !pytest_cmd.success() {
        panic!("Pytest failed.");
    }
}

