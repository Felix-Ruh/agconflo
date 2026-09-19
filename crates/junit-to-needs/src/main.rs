//! `junit-to-needs --crate <name>... <junit.xml>` prints the needs file for the
//! tests of the named crates in a nextest JUnit report.
//!
//! The file goes to stdout and nothing is written anywhere else, so a caller
//! comparing it with a committed copy can never overwrite that copy by mistake.
//! Exit status: 0 with the file printed; 1 when the report cannot be read or
//! imported, with nothing printed; 2 for a malformed command line.

use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "usage: junit-to-needs --crate <name> [--crate <name>...] <junit.xml>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (crates, path) = match parse(&args) {
        Ok(Some(parsed)) => parsed,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(problem) => {
            eprintln!("junit-to-needs: {problem}\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    let xml = match std::fs::read_to_string(path) {
        Ok(xml) => xml,
        Err(error) => {
            eprintln!("junit-to-needs: cannot read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let runs = match junit_to_needs::import(&xml, &crates) {
        Ok(runs) => runs,
        Err(error) => {
            eprintln!("junit-to-needs: {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = std::io::stdout()
        .lock()
        .write_all(runs.to_json().as_bytes())
    {
        eprintln!("junit-to-needs: cannot write the needs file: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// The crates and the report path, or `None` when help was asked for.
fn parse(args: &[String]) -> Result<Option<(Vec<&str>, &str)>, String> {
    let mut crates = Vec::new();
    let mut path = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--crate" => match args.next() {
                Some(name) if !name.starts_with('-') => crates.push(name.as_str()),
                _ => return Err("--crate needs a crate name after it".to_owned()),
            },
            option if option.starts_with('-') => return Err(format!("unknown option `{option}`")),
            report => {
                if path.replace(report).is_some() {
                    return Err("give exactly one report".to_owned());
                }
            }
        }
    }
    if crates.is_empty() {
        return Err("name at least one crate with --crate".to_owned());
    }
    let path = path.ok_or_else(|| "give the path of a JUnit report".to_owned())?;
    Ok(Some((crates, path)))
}
