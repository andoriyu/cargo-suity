# cargo-suity

[![Build Status](https://dev.azure.com/andoriyu/cargo-suity/_apis/build/status/andoriyu.cargo-suity?branchName=master)](https://dev.azure.com/andoriyu/cargo-suity/_build/latest?definitionId=3&branchName=master)
[![codecov](https://codecov.io/gh/andoriyu/cargo-suity/branch/master/graph/badge.svg)](https://codecov.io/gh/andoriyu/cargo-suity)

This tool helps you automate testing of you rust application on CI. Currently it can run defined workflows and 
report results in JUnit format. It's using unstable `rust-test` feature and may break... Under the good suity executes
 `cargo` as sub-process and parses its output.

[![asciicast](https://asciinema.org/a/IXmGVIpJzg3lzyBCpYWe3bwwq.svg)](https://asciinema.org/a/IXmGVIpJzg3lzyBCpYWe3bwwq)

## Installation

```sh
 $ cargo install -f cargo-suity
```

## Usage
```sh
 $ cargo suity
```

Current version is missing any sort of argument variables. It just runs all of workflows it could find.
 Which probably covers most of the use cases... In order to view JUnit file you probably need support of your CI
  (click on azure pipelines badge to see what I'm talking about) or some kind of [viewer](http://lukejpreston.github.io/junit_viewer/).
 
#### Exit codes
    - 0 all tests across all workflows passes
    - 101 - ran into error (permission denied, out of disk space, etc)
    - N number of failed tests


## Configuration (`suity.toml`)

File is optional. If not specified then default configuration is used. Here is an example configuration: 
```toml
[global]
features = []
format   = "JUnit"
output   = "./test-results"

[workflow.default]
doc         = false
unit        = true
integration = ["*"]

[workflow.integration-cfg-serde]
doc         = false
unit        = false
integration = ["not_really_a_test"]
```

 - `global` is used to override default values in all workflows.
 - `workflow.<name>` is used to define workflow.
### configuration toggles
| key          	| description                        	| Possible values                                                             	| default                        	|
|--------------	|------------------------------------	|-----------------------------------------------------------------------------	|--------------------------------	|
| name         	| override name for  workflow         	| Any string                                                                  	| name part in `workflow.<name>` 	|
| features     	| List of features to use            	| List of any strings                                                         	| crate's default features       	|
| format       	| Test result output format          	| JUnit                                                                       	| JUnit                          	|
| output       	| Where to save test results         	| any writable path                                                           	| `./test-results`               	|
| doc          	| Test this library's documentation. 	| true / false                                                                	| true                           	|
| lib          	| Test this package's library.       	| true / false                                                                	| true                           	|
| integration 	|                                    	| an array of  integration tests files in `tests/` folder without extension.) 	| "*" (all of them)              	|

## Code quality

Honestly...code is a mess. Only `rust-test's json to JUnit` part is covered by tests. I didn't even try running
 `clippy` on it.  However, I'm using it other projects and on itself. 