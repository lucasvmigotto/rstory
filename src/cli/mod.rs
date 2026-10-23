//! Command-line interface: clap commands, text and JSON output, exit codes.

pub mod args;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_the_contract() {
        let table = [
            (ExitCode::Ok, 0),
            (ExitCode::Unexpected, 1),
            (ExitCode::Usage, 2),
            (ExitCode::DaemonDown, 3),
            (ExitCode::AlreadyRunning, 4),
            (ExitCode::Environment, 5),
            (ExitCode::LimitReached, 6),
            (ExitCode::Inactive, 7),
            (ExitCode::Interrupted, 130),
        ];
        for (code, n) in table {
            assert_eq!(code.code(), n);
        }
    }

    #[test]
    fn protocol_errors_map_to_the_catalog_exit_codes() {
        use crate::ipc::error::ErrorCode as E;
        assert_eq!(ExitCode::from_proto(E::NotFound), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::Ambiguous), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::BadRequest), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::PinLimit), ExitCode::LimitReached);
        assert_eq!(ExitCode::from_proto(E::Inactive), ExitCode::Inactive);
        assert_eq!(ExitCode::from_proto(E::Internal), ExitCode::Unexpected);
        assert_eq!(ExitCode::from_proto(E::BackendError), ExitCode::Unexpected);
        assert_eq!(ExitCode::from_proto(E::NoTerminal), ExitCode::Unexpected);
    }

    #[test]
    fn a_ref_is_an_index_or_a_short_id() {
        assert_eq!("0".parse::<EntryRef>(), Ok(EntryRef::Index(0)));
        assert_eq!("12".parse::<EntryRef>(), Ok(EntryRef::Index(12)));
        assert_eq!(
            "01jb".parse::<EntryRef>(),
            Ok(EntryRef::ShortId("01JB".into()))
        );
        assert_eq!(
            "ab12".parse::<EntryRef>(),
            Ok(EntryRef::ShortId("AB12".into()))
        );
    }

    #[test]
    fn bad_refs_are_rejected() {
        for bad in ["", "abc", "-1", "ab 12", "ab!2", "99999999999"] {
            assert!(bad.parse::<EntryRef>().is_err(), "{bad:?}");
        }
    }
}
