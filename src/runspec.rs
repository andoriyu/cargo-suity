use std::path::PathBuf;
use crate::junit::TestSuite;
use std::process::{Command};
use std::io;
use std::fs;


use crate::results;

/// Desired output format. Right not only JUnit is supported and it's the default format.
#[derive(Debug, Copy, Clone,Deserialize)]
pub enum OutputFormat {
    JUnit
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::JUnit
    }
}

#[derive(Debug,Deserialize)]
pub struct Runspec {
    /// List of features to pass to cargo.
    pub features: Vec<String>,
    /// Report format.
    pub format: OutputFormat,
    /// Output directory. Default `./test-results/`
    pub output: PathBuf,
    /// Run Doc-Tests or not. Default true.
    pub doc: bool,
    /// Run Unit-Tests or not. Default true.
    pub lib: bool,
    /// List of integration tests to run. Default all of them.
    pub integration: Vec<String>
}

impl Default for Runspec {
    fn default() -> Runspec {
        Runspec {
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
    pub fn execute(&mut self, name: &str) -> Result<(),io::Error> {
        let mut results: Vec<TestSuite> = Vec::with_capacity(5);
        let shared_args = {
            let mut args = vec![String::from("test")];
            if !self.features.is_empty() {
                args.push(String::from("--features"));
                args.push(self.features_to_string());
            }
            args
        };
        if self.lib {
            let mut args = {
                let mut new_args = shared_args.clone();
                new_args.extend(vec![
                    String::from("--lib"),
                ]);
                new_args
            };
            args.push(String::from("--"));
            add_common_args(&mut args);
            let out = Command::new("cargo").args(&args).output()?;
            let stdout: String = String::from_utf8_lossy(&out.stdout).into();
            let events = results::parse_test_results(&stdout);
            let suite = TestSuite::new(events, String::from("Lib-tests"));
            if suite.tests > 0 {
                results.push(suite);
            }
        }
        if self.doc {
            let mut args = {
                let mut new_args = shared_args.clone();
                new_args.push(String::from("--doc"));
                new_args
            };
            args.push(String::from("--"));
            add_common_args(&mut args);
            let out = Command::new("cargo").args(&args).output()?;
            let stdout: String = String::from_utf8_lossy(&out.stdout).into();
            let events = results::parse_test_results(&stdout);
            let suite = TestSuite::new(events, String::from("Doc-tests"));
            if suite.tests > 0 {
                results.push(suite);
            }
        }

        if !self.integration.is_empty() {
            if self.integration == vec!["*"] {
                let tests = get_integration_tests();
                dbg!(&tests);
                for test in tests {
                    if let Some(path) = map_to_binary(&test) {
                        let mut args = Vec::with_capacity(3);
                        add_common_args(&mut args);
                        dbg!(&path);
                        let out = Command::new(path).args(&args).output();
                        dbg!(&out);
                        let out = out?;
                        let stdout: String = String::from_utf8_lossy(&out.stdout).into();
                        let events = results::parse_test_results(&stdout);
                        let suite = TestSuite::new(events, String::from(test));
                        if suite.tests > 0 {
                            results.push(suite);
                        }
                    }
                }
            } else {
                unimplemented!();
            }
        }
        let mut output_path = PathBuf::new();
        output_path.push(&self.output);
        output_path.push(name);
        output_path.set_extension("xml");
        let file = fs::File::create(output_path)?;
        let buf_writer = io::BufWriter::new(file);
        crate::junit::write_as_xml(results, buf_writer)?;
        Ok(())
    }
}

fn get_integration_tests() -> Vec<String> {
    if let Ok(entries) = fs::read_dir("tests/") {
        entries
            .filter_map(Result::ok)
            .filter(filters::file_with_content)
            .map(filters::to_path)
            .filter(|path| filters::extension_is(path, "rs"))
            .filter_map(|path| path.clone().file_stem().map(|osstr| String::from(osstr.to_string_lossy())))
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
    if let Ok(entries) = fs::read_dir("target/debug") {
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


mod filters {
    use std::cmp::Ordering;
    use std::fs;
    use std::path;
    use std::ffi::OsStr;
    use is_executable::IsExecutable;

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