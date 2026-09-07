use serde::Serialize;
use time::Timestamp;

#[derive(Debug, Serialize)]
pub(in crate::instance) struct InstanceDto {
    id: String,
    name: String,
    setup_completed_at_ms: Timestamp
}
