use crate::errors::SuityError;
use crate::junit::TestSuite;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::results;

pub enum RunspecResult {
    Ok,
    Errors(u64),
}

impl RunspecResult {
    pub fn as_exit_code(&self) -> i32 {
        match self {
            RunspecResult::Ok => 0,
            RunspecResult::Errors(n) => *n as i32,
        }
    }
}
/// Desired output format. Right not only JUnit is supported and it's the default format.
#[derive(Debug, Copy, Clone, Deserialize)]
pub enum OutputFormat {
    JUnit,
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::JUnit
    }
}

#[derive(Debug, Deserialize)]
pub struct Runspec {
    /// How to name this spec.
    #[serde(default = "default::name")]
    pub name: String,
    /// List of features to pass to cargo.
    #[serde(default = "default::features")]
    pub features: Vec<String>,
    /// Report format.
    #[serde(default = "default::format")]
    pub format: OutputFormat,
    /// Output directory. Default `./test-results/`
    #[serde(default = "default::output")]
    pub output: PathBuf,
    /// Run Doc-Tests or not. Default true.
    #[serde(default = "default::doc")]
    pub doc: bool,
    /// Run Unit-Tests or not. Default true.
    #[serde(default = "default::lib")]
    pub lib: bool,
    /// List of integration tests to run. Default all of them.
    #[serde(default = "default::integration")]
    pub integration: Vec<String>,
}

impl Default for Runspec {
    fn default() -> Runspec {
        Runspec {
            name: String::from("default"),
            features: Vec::new(),
            format: OutputFormat::default(),
            output: PathBuf::from("test-results/"),
            doc: true,
            lib: true,
            integration: vec![String::from("*")],
        }
    }
}

impl Runspec {
    fn features_to_string(&self) -> String {
        itertools::join(self.features.iter(), " ")
    }

    pub fn execute<W: io::Write>(&mut self, output: W) -> Result<Vec<TestSuite>, SuityError> {
        let mut results: Vec<TestSuite> = Vec::with_capacity(5);
        let shared_args = self.get_shared_args();

        let mut args = shared_args.clone();
        args.push(String::from("--no-run"));
        let status = Command::new("cargo").args(args).status()?;
        if !status.success() {
            return Err(SuityError::FailedToCompile {
                workflow: self.name.clone(),
            });
        }
        if self.lib {
            let mut args = shared_args.clone();
            args.push(String::from("--lib"));
            args.push(String::from("--"));
            add_common_args(&mut args);

            let test_suite_name = format!("[{}] Lib-tests", self.name).to_string();

            if let Some(suite) = Runspec::run_cargo(&mut args, test_suite_name)? {
                results.push(suite);
            }
        }
        if self.doc {
            let mut args = shared_args.clone();
            args.push(String::from("--doc"));
            args.push(String::from("--"));
            add_common_args(&mut args);
            let test_suite_name = format!("[{}] Doc-tests", self.name).to_string();

            if let Some(suite) = Runspec::run_cargo(&mut args, test_suite_name)? {
                results.push(suite);
            }
        }

        if !self.integration.is_empty() {
            let tests = {
                if self.integration == vec!["*"] {
                    get_integration_tests()
                } else {
                    self.integration.clone()
                }
            };
            for name in tests {
                if name != "*" {
                    if let Some(suite) = self.run_integration_test(&name)? {
                        results.push(suite);
                    }
                }
            }
        }
        crate::junit::write_as_xml(&results, output)?;
        Ok(results)
    }

    fn run_integration_test(&mut self, test: &String) -> Result<Option<TestSuite>, SuityError> {
        let test_suite_name = format!("[{}] {}", self.name, &test).to_string();
        if let Some(path) = map_to_binary(&test) {
            let mut args = Vec::with_capacity(3);
            add_common_args(&mut args);
            let out = Command::new(path).args(&args).output()?;
            Runspec::parse_test_output(test_suite_name, &out)
        } else {
            Err(SuityError::TestBinaryNotFound {
                name: test.clone(),
                workflow: self.name.clone(),
            })
        }
    }

    fn run_cargo(
        args: &Vec<String>,
        test_suite_name: String,
    ) -> Result<Option<TestSuite>, SuityError> {
        let out = Command::new("cargo").args(args).output()?;
        Runspec::parse_test_output(test_suite_name, &out)
    }

    fn parse_test_output(
        test_suite_name: String,
        out: &Output,
    ) -> Result<Option<TestSuite>, SuityError> {
        let stdout: String = String::from_utf8_lossy(&out.stdout).into();
        let events = results::parse_test_results(&stdout);
        let suite = TestSuite::new(events, test_suite_name)?;
        if suite.tests > 0 {
            Ok(Some(suite))
        } else {
            Ok(None)
        }
    }

    fn get_shared_args(&mut self) -> Vec<String> {
        let mut args: Vec<String> = vec![String::from("test")];
        if !self.features.is_empty() {
            args.push(String::from("--features"));
            args.push(self.features_to_string());
        }
        args
    }

    pub fn get_output_file_path(&self) -> PathBuf {
        let mut output_path = PathBuf::new();
        output_path.push(&self.output);
        output_path.push(&self.name);
        output_path.set_extension("xml");
        output_path
    }
}

fn get_integration_tests() -> Vec<String> {
    if let Ok(entries) = fs::read_dir("tests/") {
        entries
            .filter_map(Result::ok)
            .filter(filters::file_with_content)
            .map(filters::to_path)
            .filter(|path| filters::extension_is(path, "rs"))
            .filter_map(|path| {
                path.clone()
                    .file_stem()
                    .map(|osstr| String::from(osstr.to_string_lossy()))
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn add_common_args(args: &mut Vec<String>) {
    args.push(String::from("-Z"));
    args.push(String::from("unstable-options"));
    args.push(String::from("--format=json"));
}

fn map_to_binary(name: &String) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir("target/debug/deps") {
        let mut executables: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .filter(filters::file_with_content)
            .map(filters::to_path)
            .filter(filters::is_executable)
            .filter(|f| filters::filename_starts_with(&f, &name))
            .collect();
        executables.sort_by(filters::sort_by_modify_date);
        executables.last().cloned()
    } else {
        None
    }
}

mod default {
    pub fn name() -> String {
        super::Runspec::default().name.clone()
    }
    pub fn features() -> Vec<String> {
        super::Runspec::default().features.clone()
    }
    pub fn format() -> super::OutputFormat {
        super::Runspec::default().format.clone()
    }
    pub fn output() -> std::path::PathBuf {
        super::Runspec::default().output.clone()
    }
    pub fn doc() -> bool {
        super::Runspec::default().doc.clone()
    }
    pub fn lib() -> bool {
        super::Runspec::default().lib.clone()
    }
    pub fn integration() -> Vec<String> {
        super::Runspec::default().integration.clone()
    }
}

mod filters {
    use is_executable::IsExecutable;
    use std::cmp::Ordering;
    use std::ffi::OsStr;
    use std::fs;
    use std::path;

    pub fn file_with_content(f: &fs::DirEntry) -> bool {
        if let Ok(meta) = f.metadata() {
            meta.is_file() && meta.len() > 1
        } else {
            false
        }
    }

    pub fn to_path(f: fs::DirEntry) -> path::PathBuf {
        f.path()
    }

    pub fn is_executable(f: &path::PathBuf) -> bool {
        f.is_executable()
    }

    pub fn extension_is(f: &path::PathBuf, suffix: &str) -> bool {
        let rust_ext = OsStr::new(suffix);
        f.extension() == Some(&rust_ext)
    }

    pub fn filename_starts_with(f: &path::PathBuf, prefix: &str) -> bool {
        if let Some(filename) = f.file_name() {
            let filename = filename.to_str().unwrap_or("");
            filename.starts_with(prefix)
        } else {
            false
        }
    }
    pub fn sort_by_modify_date(a: &path::PathBuf, b: &path::PathBuf) -> Ordering {
        let a_meta = a.metadata().map(|a| a.modified().unwrap()).unwrap();
        let b_meta = b.metadata().map(|b| b.modified().unwrap()).unwrap();
        a_meta.cmp(&b_meta)
    }
}
