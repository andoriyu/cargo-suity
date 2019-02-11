//#[macro_use]
//extern crate structopt;
//use structopt::StructOpt;
use colored::*;
use std::{io,fs,path, process};
use cargo_suity as lib;

/*
/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "cargo suity", about = "Test runner and reporter for cargo.")]
struct Suity {
    /// Names of workflows to run. Workflows are defined in suity.toml. If unspecified - run default one. If set to "*" - runs all.
    pub workflows: Vec<String>,
}
*/
fn main() {
    //let opt = Suity::from_args();
    let mut runspec = lib::runspec::Runspec::default();

    match  get_writer(runspec.get_output_file_path()) {
        Ok(buf_writer) => {
            let result = runspec.execute(buf_writer).expect("Failed to run default configuration");
            print_results(&runspec, &result);
            let total_number_of_failed: u64 = result.iter().map(|s| s.failures).sum();
            if total_number_of_failed > 0 {
                process::exit(1);
            }
        },
        Err(e) => eprintln!("Failed to create output file because: {:?}", e)
    }
}

fn get_writer(path: path::PathBuf) -> Result<io::BufWriter<fs::File>, lib::errors::SuityError> {
    if let Some(parent) = path.parent() {
        if ! parent.exists() {
            fs::create_dir(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    Ok(io::BufWriter::new(file))
}

fn print_results(runspec: &lib::runspec::Runspec, result: &Vec<lib::junit::TestSuite>) {
    let total_number_of_tests: u64 = result.iter().map(|s| s.tests).sum();
    let total_number_of_failed: u64 = result.iter().map(|s| s.failures).sum();
    let total_number_of_failed_str = if total_number_of_failed == 0 {
        "0".green().bold()
    } else {
        total_number_of_failed.to_string().red().bold()
    };
    eprintln!("> Workflow:                                  {}", &runspec.name);
    eprintln!("> Total number of tests in workflow:         {}", &total_number_of_tests);
    eprintln!("> Total number of failed tests in workflow:  {}", &total_number_of_failed_str);
    eprintln!();
    for suite in result {
        let pass_or_fail = if suite.failures == 0 {
            "PASS".green().bold()
        } else {
            "FAIL".red().bold()
        };
        eprintln!(" {} {}", pass_or_fail, &suite.name);

        for case in &suite.test_cases {
            let failure = &case.failure;
            let check_or_cross = if failure.is_some() {
                "☓".red().bold()
            } else {
                "✓".green().bold()
            };
            eprintln!("    {} {}", check_or_cross, case.name);
            if let Some(ref failure) = failure {
                for line in failure.message.lines() {
                    eprintln!("        {}", line);
                }
            }
        }
        eprintln!();
    }
}
