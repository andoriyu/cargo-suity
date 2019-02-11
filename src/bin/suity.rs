//#[macro_use]
//extern crate structopt;
//use structopt::StructOpt;
use colored::*;
use std::{io,fs,path, process};
use cargo_suity as lib;
use lib::runspec::{RunspecResult};
use lib::errors::SuityError;
use lib::configuration;
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
    match run_whole_thing() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Ran into error: {}", e);
            process::exit(101)
        }
    }

}

fn run_whole_thing() -> Result<i32, SuityError> {
    let configuration = get_configuration()?;
    let mut exit_code = 0;

    for mut spec in configuration.get_runspecs() {
        exit_code += execute_runspec(&mut spec).map(|r| r.as_exit_code())?;
    }
    Ok(exit_code)

}

fn get_configuration() -> Result<configuration::Configuration, SuityError> {
    let conf_file =  path::Path::new("suity.toml");

    if conf_file.exists() {
        let contents = fs::read_to_string(conf_file)?;
        let conf: configuration::Configuration = toml::from_str(&contents).map_err(|e| SuityError::FailedToParseConfiguration(e))?;
        Ok(conf)
    } else {
        Ok(configuration::Configuration::default())
    }
}

fn execute_runspec(runspec: &mut lib::runspec::Runspec) -> Result<RunspecResult, SuityError> {
    let buf_writer= get_writer(runspec.get_output_file_path())?;
    let result = runspec.execute(buf_writer).expect("Failed to run default configuration");
    print_results(&runspec, &result);
    let total_number_of_failed: u64 = result.iter().map(|s| s.failures).sum();
    if total_number_of_failed > 0 {
        Ok(RunspecResult::Errors(total_number_of_failed))
    } else {
        Ok(RunspecResult::Ok)
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
        "0".green()
    } else {
        total_number_of_failed.to_string().red()
    };
    eprintln!("> Workflow:                                  {}", &runspec.name);
    eprintln!("> Total number of tests in workflow:         {}", &total_number_of_tests);
    eprintln!("> Total number of failed tests in workflow:  {}", &total_number_of_failed_str);
    eprintln!();
    for suite in result {
        let pass_or_fail = if suite.failures == 0 {
            "PASS".green()
        } else {
            "FAIL".red()
        };
        eprintln!(" {} {}", pass_or_fail, &suite.name);

        for case in &suite.test_cases {
            let failure = &case.failure;
            let check_or_cross = if failure.is_some() {
                "☓".red()
            } else {
                "✓".green()
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
