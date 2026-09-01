pub(crate) fn now_ms() -> i64 {
    let milliseconds = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    i64::try_from(milliseconds).expect("current Unix timestamp must fit in i64 milliseconds")
}
