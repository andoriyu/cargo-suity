/// Type of event generated by test runner
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum ItemKind {
    /// One specific instance of test
    Test,
    /// Group of tests
    Suite,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Started,
    Ok,
    Failed,
    Ignored,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Suite {
    pub event: EventKind,
    pub test_count: Option<u64>,
    pub passed: Option<u64>,
    pub failed: Option<u64>,
    pub allowed_fail: Option<u64>,
    pub ignored: Option<u64>,
    pub measured: Option<u64>,
    pub filtered_out: Option<u64>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Test {
    pub event: EventKind,
    pub name: String,
    pub stdout: Option<String>,
}
#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Suite(Suite),
    Test(Test),
}

impl Event {
    #[cfg(test)]
    pub(crate) fn new_suite(event: EventKind) -> Event {
        Event::Suite(Suite {
            event,
            test_count: None,
            passed: None,
            failed: None,
            allowed_fail: None,
            ignored: None,
            measured: None,
            filtered_out: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn set_test_count(mut self, test_count: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.test_count = Some(test_count),
            _ => panic!("trying to set test_count on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_passed(mut self, passed: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.passed = Some(passed),
            _ => panic!("trying to set passed on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_failed(mut self, failed: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.failed = Some(failed),
            _ => panic!("trying to set failed on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_allowed_fail(mut self, allowed_fail: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.allowed_fail = Some(allowed_fail),
            _ => panic!("trying to set allowed_fail on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_ignored(mut self, ignored: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.ignored = Some(ignored),
            _ => panic!("trying to set ignored on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_measured(mut self, measured: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.measured = Some(measured),
            _ => panic!("trying to set measured on Event::Test"),
        };
        self
    }
    #[cfg(test)]
    pub(crate) fn set_filtered_out(mut self, filtered_out: u64) -> Event {
        match self {
            Event::Suite(ref mut s) => s.filtered_out = Some(filtered_out),
            _ => panic!("trying to set filtered_out on Event::Test"),
        };
        self
    }

    #[cfg(test)]
    pub(crate) fn new_test(event: EventKind, name: String) -> Event {
        Event::Test(Test {
            event,
            name,
            stdout: None,
        })
    }
    #[cfg(test)]
    pub(crate) fn set_stdout(mut self, stdout: String) -> Event {
        match self {
            Event::Test(ref mut t) => t.stdout = Some(stdout),
            _ => panic!("trying to set stdout on Event::Suite"),
        };
        self
    }
}

pub fn parse_test_results(stdout: &str) -> Vec<Event> {
    stdout
        .lines()
        .map(serde_json::from_str)
        .map(Result::unwrap)
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::{parse_test_results, Event, EventKind};

    #[test]
    fn suite_started() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 1 }"#;

        let expected = Event::new_suite(EventKind::Started).set_test_count(1);
        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }

    #[test]
    fn suite_ended_ok() {
        let stdout = r#"{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;

        let expected = Event::new_suite(EventKind::Ok)
            .set_passed(1)
            .set_failed(0)
            .set_allowed_fail(0)
            .set_ignored(0)
            .set_measured(0)
            .set_filtered_out(40);

        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }
    #[test]
    fn suite_ended_bad() {
        let stdout = r#"{ "type": "suite", "event": "failed", "passed": 15, "failed": 2, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }"#;

        let expected = Event::new_suite(EventKind::Failed)
            .set_passed(15)
            .set_failed(2)
            .set_allowed_fail(0)
            .set_ignored(0)
            .set_measured(0)
            .set_filtered_out(0);

        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }

    #[test]
    fn test_stared() {
        let stdout = r#"{ "type": "test", "event": "started", "name": "test_zpool_scrub" }"#;

        let expected = Event::new_test(EventKind::Started, String::from("test_zpool_scrub"));

        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }
    #[test]
    fn test_completed_ok() {
        let stdout = r#"{ "type": "test", "name": "test_zpool_scrub", "event": "ok" }"#;

        let expected = Event::new_test(EventKind::Ok, String::from("test_zpool_scrub"));

        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }
    #[test]
    fn test_completed_bad() {
        let stdout = r#"{ "type": "test", "name": "test_status", "event": "failed", "stdout": "thread 'test_status' panicked at 'assertion failed: `(left == right)`\n  left: `CreateZpoolRequest { name: \"tests-13180141141555479701\", props: None, altroot: None, mount: None, create_mode: Gentle, vdevs: [SingleDisk(\"/vdevs/vdev0\")], caches: [], zil: None }`,\n right: `CreateZpoolRequest { name: \"tank\", props: None, altroot: None, mount: None, create_mode: Gentle, vdevs: [SingleDisk(\"/vdevs/vdev0\")], caches: [], zil: None }`', tests/test_zpool.rs:401:9\nthread 'test_status' panicked at 'called `Result::unwrap()` on an `Err` value: Any', src/libcore/result.rs:1009:5\n" }"#;

        let err = b"thread 'test_status' panicked at 'assertion failed: `(left == right)`\n  left: `CreateZpoolRequest { name: \"tests-13180141141555479701\", props: None, altroot: None, mount: None, create_mode: Gentle, vdevs: [SingleDisk(\"/vdevs/vdev0\")], caches: [], zil: None }`,\n right: `CreateZpoolRequest { name: \"tank\", props: None, altroot: None, mount: None, create_mode: Gentle, vdevs: [SingleDisk(\"/vdevs/vdev0\")], caches: [], zil: None }`', tests/test_zpool.rs:401:9\nthread 'test_status' panicked at 'called `Result::unwrap()` on an `Err` value: Any', src/libcore/result.rs:1009:5\n";
        let expected = Event::new_test(EventKind::Failed, String::from("test_status"))
            .set_stdout(String::from_utf8_lossy(err.as_ref()).into());

        let event: Event = serde_json::from_str(stdout).expect("Failed to parse stdout!");
        assert_eq!(expected, event);
    }

    #[test]
    fn test_simple_output() {
        let stdout = r#"{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "parsers::test::test_zpools_on_single_zpool" }
{ "type": "test", "name": "parsers::test::test_zpools_on_single_zpool", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 40 }"#;

        let mut expected = Vec::with_capacity(4);
        expected.push(Event::new_suite(EventKind::Started).set_test_count(1));
        expected.push(Event::new_test(
            EventKind::Started,
            String::from("parsers::test::test_zpools_on_single_zpool"),
        ));
        expected.push(Event::new_test(
            EventKind::Ok,
            String::from("parsers::test::test_zpools_on_single_zpool"),
        ));
        expected.push(
            Event::new_suite(EventKind::Ok)
                .set_passed(1)
                .set_failed(0)
                .set_allowed_fail(0)
                .set_ignored(0)
                .set_measured(0)
                .set_filtered_out(40),
        );

        let actual = parse_test_results(stdout);
        assert_eq!(expected, actual);
    }
}
