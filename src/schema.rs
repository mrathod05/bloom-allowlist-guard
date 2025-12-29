// @generated automatically by Diesel CLI.

diesel::table! {
    bloom_allowlist (id) {
        id -> Int4,
        wallet_address -> Text,
        created_at -> Nullable<Timestamp>,
    }
}
