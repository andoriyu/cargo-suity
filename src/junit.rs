/// Support for export in JUnit format.

use crate::results::{Event, EventKind};
use crate::errors::SuityError;
use std::io::{Write,self};
use xml_writer::XmlWriter;

/// Indicates that the test failed. A failure is a test which the code has explicitly failed by
/// using the mechanisms for that purpose. e.g., via an assertEq.
/// Contains as a text node relevant data for the failure, e.g., a stack trace.
#[derive(Debug, Eq, PartialEq)]
pub struct Failure {
    /// Relevant data for the failure
    pub message: String,
}

/// Contains result of a test case
#[derive(Debug, Eq, PartialEq)]
pub struct TestCase {
    /// The full name of test
    pub name: String,
    /// Indicates that test failed
    pub failure: Option<Failure>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TestSuite {
    /// Name of the test suite
    pub name: String,
    /// How many tests erred out. In rust we can't can't detect it :(
    pub errors: u64,
    /// How many tests failed.
    pub failures: u64,
    /// Total amount of tests
    pub tests: u64,
    pub test_cases: Vec<TestCase>,
}



impl TestSuite {
    /// Create TestSuite from event stream.
    /// NOTE: Only works if event stream is related to a single testsuite. You have to run unit, doc
    /// and integration tests separately!
    pub fn new(events: Vec<Event>, name: String) -> Result<TestSuite,SuityError> {

        let mut suite = TestSuite {
            name,
            errors: 0,
            failures:0,
            tests: 0,
            test_cases: Vec::new()
        };

        let mut counter = 0;


        for event in events {
            match event {
                Event::Suite(s) => {
                    match s.event {
                        EventKind::Ignored => { /* no-op */ },
                        EventKind::Started => {
                            suite.tests = s.test_count.unwrap();
                            counter += 1;
                            if counter > 1 {
                                return Err(SuityError::MultipleTestRuns);
                            }
                        },
                        EventKind::Failed | EventKind::Ok => {
                            suite.failures = s.failed.unwrap();
                        }
                    }
                },
                Event::Test(t) => {
                    match t.event {
                        EventKind::Started => { /* no-op */ },
                        EventKind::Ignored => { /* no-op */ },
                        EventKind::Ok => {
                            suite.test_cases.push(
                                TestCase {
                                    name: t.name,
                                    failure: None
                                }
                            )
                        }
                        EventKind::Failed => {
                            suite.test_cases.push(
                                TestCase {
                                    name: t.name,
                                    failure: Some(Failure{
                                        message: t.stdout.unwrap()
                                    })
                                }
                            )
                        }
                    }

                }
            }
        }
        Ok(suite)
    }
}


pub fn write_as_xml<W: Write>(suites: &Vec<TestSuite>, writer: W) -> Result<(),io::Error> {
    let mut xml = XmlWriter::new(writer);
    xml.dtd("utf-8")?;
    xml.begin_elem("testsuites")?;
    for suite in suites {
        xml.begin_elem("testsuite")?;
        xml.attr_esc("name", &suite.name)?;
        xml.attr("errors", suite.errors.to_string().as_str())?;
        xml.attr("failures", suite.failures.to_string().as_str())?;
        xml.attr("tests", suite.tests.to_string().as_str())?;
        for testcase in &suite.test_cases {
            xml.begin_elem("testcase")?;
            xml.attr("name", &testcase.name)?;
            if let Some(ref failure) = &testcase.failure {
                xml.begin_elem("failure")?;
                xml.attr_esc("message", &failure.message)?;
                xml.end_elem()?;
            }
            xml.end_elem()?;
        }
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()
}

#[cfg(test)]
mod tests {

    use crate::results::parse_test_results;
    use super::{TestSuite, TestCase, Failure, write_as_xml};

    #[test]
    fn test_simple_output() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;

        let events = parse_test_results(stdout);

        let name = String::from("Doc Tests");
        let test_name = String::from("parsers::test::test_zpools_on_single_zpool");
        let expected_test_case = TestCase {
            name: test_name.clone(),
            failure: None,
        };
        let expected = TestSuite {
            name: name.clone(),
            errors: 0,
            failures: 0,
            tests: 1,
            test_cases: vec![expected_test_case]
        };
        let suite = TestSuite::new(events, name).unwrap();

        assert_eq!(expected, suite);
    }

    #[test]
    fn test_failed_output() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 2 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "test", "event": "started", "name": "failed" }
{ "type": "test", "name": "failed", "event": "failed", "stdout": "idk dawg" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 1, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;

        let events = parse_test_results(stdout);

        let name = String::from("Doc Tests");
        let expected_test_case = TestCase {
            name: String::from("parsers::test::test_zpools_on_single_zpool"),
            failure: None,
        };
        let expected_test_case2 = TestCase {
            name: String::from("failed"),
            failure: Some(Failure {
                message: String::from("idk dawg")
            }),
        };
        let expected = TestSuite {
            name: name.clone(),
            errors: 0,
            failures: 1,
            tests: 2,
            test_cases: vec![expected_test_case, expected_test_case2]
        };
        let suite = TestSuite::new(events, name).unwrap();

        assert_eq!(expected, suite);
    }


    #[test]
    fn test_generate_xml_no_error_single_testsuite() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 2 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "test", "event": "started", "name": "failed" }
{ "type": "test", "name": "failed", "event": "failed", "stdout": "idk dawg" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 1, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;

        let name = String::from("Doc Tests");
        let events = parse_test_results(stdout);
        let suite = TestSuite::new(events, name).unwrap();

        let suites = vec![suite];

        let mut output = Vec::with_capacity(128);

        write_as_xml(&suites, &mut output).unwrap();
    }
    #[test]
    fn test_multiple_outputs() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }
{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;
        let events = parse_test_results(stdout);
        let suite = TestSuite::new(events, String::from("should fail"));
        assert!(suite.is_err());
    }
}
