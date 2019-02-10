use std::default::Default;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::runspec::{OutputFormat,Runspec};

#[derive(Debug, Default, Clone,Deserialize)]
pub struct Workflow {
    /// List of features to pass to cargo.
    pub features: Option<Vec<String>>,
    /// Report format.
    pub format: Option<OutputFormat>,
    /// Output directory. Default `./test-results/`
    pub output: Option<PathBuf>,
    /// Run Doc-Tests or not. Default true.
    pub doc: Option<bool>,
    /// Run Unit-Tests or not. Default true.
    pub unit: Option<bool>,
    /// List of integration tests to run. Default all of them.
    pub integration: Option<Vec<String>>
}

impl Workflow {
    /// Merge default run configuration and defined workflow.
    pub fn merge(self, right: &Runspec) -> Runspec {
        Runspec {
            features: self.features.unwrap_or(right.features.clone()),
            format: self.format.unwrap_or(right.format),
            output: self.output.unwrap_or(right.output.clone()),
            doc: self.doc.unwrap_or(right.doc),
            lib: self.unit.unwrap_or(right.lib),
            integration: self.integration.unwrap_or(right.integration.clone()),
        }
    }
}

#[derive(Debug,Deserialize)]
pub struct Configuration {
    pub global: Runspec,
    pub workflow: HashMap<String, Workflow>,
}

impl Configuration {
    /// Return list of fully actionable run configurations aka Runspec.
    pub fn get_runspecs(&self) -> Vec<(String,Runspec)> {
        if self.workflow.is_empty() {
            return vec![self.get_default()];
        }
        unimplemented!()
        //self.workflow.values().cloned().map(|workflow| workflow.merge(&self.global)).collect()
    }

    /// Give specific runspec
    pub fn get_runspec(&self, name: &String) -> Option<(String, Runspec)> {
        self.workflow.get(name).map(|s| (name.clone(), s.clone().merge(&self.global)))
    }

    pub fn get_default(&self) -> (String, Runspec) {
        let name = String::from("default");
        self.get_runspec(&name)
            .unwrap_or((name, Workflow::default().merge(&self.global)))
    }
}

