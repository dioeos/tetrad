#![allow(unused)]

mod crud_fns;

pub(in crate::model) use crud_fns::{get, prep_fields_for_create};

use sea_query::{Alias, DynIden, Iden, IntoIden, IntoTableRef, TableRef, Value};

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
        Alias::new(Self::TABLE).into_table_ref()
    }

    fn has_timestamps() -> bool {
        true
    }

    fn has_owner_id() -> bool {
        false
    }
}

pub(in crate::model) struct Field {
    column: DynIden,
    value: Value,
}
//represents one column and a corresponding value
impl Field {
    pub fn new<C, V>(column: C, value: V) -> Self
    where
        C: IntoIden,
        V: Into<Value>,
    {
        Self {
            column: column.into_iden(),
            value: value.into(),
        }
    }
}

#[derive(Default)]
pub(in crate::model) struct Fields {
    entries: Vec<Field>,
}

impl Fields {
    pub fn push(&mut self, field: Field) {
        self.entries.push(field);
    }

    pub fn push_value<C, V>(&mut self, column: C, value: V)
    where
        C: IntoIden,
        V: Into<Value>,
    {
        self.push(Field::new(column, value));
    }

    //@NOTE: Use `insert_some` to omit absent values while mapping (mimics not_none_sea_fields)
    pub fn insert_some<C, V>(&mut self, column: C, value: Option<V>)
    where
        C: IntoIden,
        V: Into<Value>,
    {
        if let Some(value) = value {
            self.push_value(column, value)
        }
    }

    pub fn insert_columns_and_values(self) -> (Vec<DynIden>, Vec<Value>) {
        self.entries
            .into_iter()
            .map(|field| (field.column, field.value))
            .unzip()
    }
}

pub trait IntoFields {
    fn into_fields(self) -> Fields;
}

pub trait SelectFields {
    fn select_columns() -> Vec<DynIden>;
}
