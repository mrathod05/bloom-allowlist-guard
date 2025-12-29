use crate::schema::bloom_allowlist;
use diesel::prelude::*;

    #[derive(Queryable, Selectable)]
    #[diesel(table_name = bloom_allowlist)]
    pub struct AllowlistEntry {
        pub id: i32,
        pub wallet_address: String,
    }

    #[derive(Insertable)]
    #[diesel(table_name = bloom_allowlist)]
    pub struct NewEntry {
        pub wallet_address: String,
    }
