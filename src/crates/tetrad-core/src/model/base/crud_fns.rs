use time::Timestamp;

use super::{DbBmc, Fields, TimestampIden};

pub fn prep_fields_for_create<BMC>(fields: &mut Fields)
where
    BMC: DbBmc,
{
    if BMC::has_timestamps() {
        add_timestamps_for_create(fields);
    }
}

fn add_timestamps_for_create(fields: &mut Fields) {
    let now_ms = Timestamp::now().as_milliseconds();
    fields.push_value(TimestampIden::CreatedAtMs, now_ms);
    fields.push_value(TimestampIden::UpdatedAtMs, now_ms);
}
