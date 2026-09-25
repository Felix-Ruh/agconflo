//! Compiling a snippet against this crate, for the test cases whose property is
//! that some code does not compile.
//!
//! Each snippet becomes the body of `main` in its own project under
//! `<target>/compile-fail/<case>/`, depending on this crate by path, checked
//! with `cargo check`. A snippet that compiles, a refusal with another error
//! code, and cargo not starting at all each fail the assertion.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Asserts that `body` does not compile, refused with error `code` and a
/// message containing `phrase`.
// @Snippets checked in projects of their own,TRACE_COMPILE_FAIL_HARNESS,trace,[],[NOTE_COMPILE_FAIL_HARNESS]
pub(crate) fn assert_refused(case: &str, body: &str, code: &str, phrase: &str) {
    let stderr = match check(case, body) {
        Ok(()) => panic!("`{case}` compiled, and was expected to be refused with {code}"),
        Err(stderr) => stderr,
    };
    assert!(
        stderr.contains(&format!("error[{code}]")),
        "`{case}` was refused, but not with {code}:\n{stderr}"
    );
    assert!(
        stderr.contains(phrase),
        "`{case}` was refused with {code}, but no message says \"{phrase}\":\n{stderr}"
    );
}

/// Asserts that `body` compiles.
pub(crate) fn assert_compiles(case: &str, body: &str) {
    if let Err(stderr) = check(case, body) {
        panic!("`{case}` was expected to compile:\n{stderr}");
    }
}

/// `Ok` when the snippet compiles, otherwise cargo's error output.
fn check(case: &str, body: &str) -> Result<(), String> {
    let root = target_dir().join("compile-fail");
    let project = root.join(case);
    std::fs::create_dir_all(project.join("src")).expect("the case directory can be created");

    // Forward slashes, because backslashes in a TOML string are escapes and
    // Windows accepts either separator.
    let this_crate = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    // The empty `[workspace]` keeps the project out of this repository's
    // workspace, which would otherwise claim it and refuse to build it.
    let manifest = format!(
        "[package]\nname = \"compile-fail-{case}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [dependencies]\nagconflo-core = {{ path = \"{this_crate}\" }}\n\n[workspace]\n"
    );
    std::fs::write(project.join("Cargo.toml"), manifest).expect("the manifest can be written");
    std::fs::write(
        project.join("src").join("main.rs"),
        format!("fn main() {{\n    {body}\n}}\n"),
    )
    .expect("the snippet can be written");

    let output = Command::new(env!("CARGO"))
        .args(["check", "--offline", "--quiet", "--color", "never"])
        .current_dir(&project)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .unwrap_or_else(|error| panic!("cargo could not be started: {error}"));

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// The target directory this test binary was built into. The binary runs from
/// `<target>/<profile>/deps/`, which also holds when the target directory has
/// been moved with `CARGO_TARGET_DIR`.
fn target_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("the test binary knows its own path");
    exe.ancestors()
        .find(|dir| dir.ends_with("deps"))
        .and_then(|deps| deps.parent()?.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| panic!("{} is not inside <target>/<profile>/deps", exe.display()))
}
