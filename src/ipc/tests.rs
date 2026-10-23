use super::codec::{CodecError, LineReader, encode};
use super::protocol::{MAX_LINE_BYTES, parse_duration};

fn reader(data: &[u8], max: usize) -> LineReader<&[u8]> {
    LineReader::with_limit(data, max)
}

#[tokio::test]
async fn lines_are_split_on_newlines_and_empty_lines_are_skipped() {
    let mut r = reader(b"one\n\ntwo\r\nthree", 64);
    assert_eq!(r.next_line().await.unwrap(), Some(b"one".to_vec()));
    assert_eq!(r.next_line().await.unwrap(), Some(b"two".to_vec()));
    assert_eq!(r.next_line().await.unwrap(), Some(b"three".to_vec()));
    assert_eq!(r.next_line().await.unwrap(), None);
}

#[tokio::test]
async fn a_line_at_the_limit_is_accepted_and_one_byte_more_is_not() {
    let mut ok = vec![b'a'; 16];
    ok.push(b'\n');
    assert_eq!(
        reader(&ok, 16).next_line().await.unwrap().map(|l| l.len()),
        Some(16)
    );
    let mut long = vec![b'a'; 17];
    long.push(b'\n');
    assert!(matches!(
        reader(&long, 16).next_line().await,
        Err(CodecError::TooLong)
    ));
}

#[tokio::test]
async fn an_unterminated_endless_line_is_cut_off_at_the_limit() {
    let data = vec![b'a'; 100_000];
    assert!(matches!(
        reader(&data, 1000).next_line().await,
        Err(CodecError::TooLong)
    ));
}

#[tokio::test]
async fn the_protocol_limit_is_two_mebibytes() {
    assert_eq!(MAX_LINE_BYTES, 2 * 1024 * 1024);
    let mut line = vec![b'x'; MAX_LINE_BYTES];
    line.push(b'\n');
    let mut r = LineReader::new(&line[..]);
    assert_eq!(
        r.next_line().await.unwrap().map(|l| l.len()),
        Some(MAX_LINE_BYTES)
    );
    let mut line = vec![b'x'; MAX_LINE_BYTES + 1];
    line.push(b'\n');
    let mut r = LineReader::new(&line[..]);
    assert!(matches!(r.next_line().await, Err(CodecError::TooLong)));
}

#[test]
fn encode_adds_one_newline() {
    assert_eq!(encode(&serde_json::json!({"a": 1})), b"{\"a\":1}\n");
}

#[test]
fn durations_follow_the_schema_pattern() {
    use std::time::Duration;
    assert_eq!(parse_duration("50s"), Some(Duration::from_secs(50)));
    assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
    assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
    for bad in [
        "",
        "s",
        "5",
        "5d",
        "-5s",
        "5.5s",
        "5 s",
        "٣s",
        "99999999999999999999s",
    ] {
        assert_eq!(parse_duration(bad), None, "{bad:?}");
    }
}
