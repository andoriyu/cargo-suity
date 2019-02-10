//#[macro_use]
//extern crate structopt;
//use structopt::StructOpt;


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
    lib::runspec::Runspec::default().execute("default").expect("Failed to run default configuration");
}
