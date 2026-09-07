mod crud_fns;

use modql::SIden;
use sea_query::{Iden, IntoTableRef, TableRef};

#[derive(Iden)]
pub(in crate::model) enum TimestampIden {
    CreatedAtMs,
    UpdatedAtMs,
}

#[derive(Iden)]
pub enum CommonIden {
	Id,
}

pub(in crate::model) trait DbBmc {
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        SIden(Self::TABLE).into_table_ref()
    }

    fn has_timestamps() -> bool {
        true
    }

    fn has_owner_id() -> bool {
        false
    }
}
