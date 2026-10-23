//! Doctor tests: foundation (T001), healthy baseline and JSON (T002), failure table (T005),
//! snapshots (T011) and probes (T010).

use super::*;

mod foundation {
    use super::*;

    fn c(status: Status) -> Check {
        Check {
            id: "x",
            status,
            message: "m".into(),
            hint: None,
        }
    }

    #[test]
    fn statuses_have_json_names_and_an_order() {
        let names: Vec<_> = [Status::Ok, Status::Info, Status::Warn, Status::Fail]
            .map(Status::as_str)
            .to_vec();
        assert_eq!(names, ["ok", "info", "warn", "fail"]);
        assert!(Status::Ok < Status::Info && Status::Info < Status::Warn);
        assert!(Status::Warn < Status::Fail);
    }

    #[test]
    fn exit_scale_is_zero_for_ok_and_info_one_for_warn_two_for_fail() {
        assert_eq!(exit_code(&[]), 0);
        assert_eq!(exit_code(&[c(Status::Ok), c(Status::Info)]), 0);
        assert_eq!(exit_code(&[c(Status::Ok), c(Status::Warn)]), 1);
        assert_eq!(
            exit_code(&[c(Status::Fail), c(Status::Warn), c(Status::Ok)]),
            2
        );
    }

    #[test]
    fn counts_warnings_and_failures_only() {
        let all = [
            c(Status::Ok),
            c(Status::Info),
            c(Status::Warn),
            c(Status::Warn),
            c(Status::Fail),
        ];
        assert_eq!(counts(&all), (2, 1));
    }

    #[test]
    fn constructors_set_status_and_hint() {
        let ok = Check::ok("session", "Wayland");
        assert_eq!((ok.id, ok.status, ok.hint), ("session", Status::Ok, None));
        let w = Check::with_hint("tray", Status::Warn, "m", "h");
        assert_eq!((w.status, w.hint.as_deref()), (Status::Warn, Some("h")));
    }
}
