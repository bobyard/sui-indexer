// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "activity_type"))]
    pub struct ActivityType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ActivityType;

    activities (id) {
        id -> Int8,
        chain_id -> Int8,
        version -> Int8,
        event_account_address -> Text,
        event_creation_number -> Int8,
        event_sequence_number -> Int8,
        collection_data_id_hash -> Text,
        token_data_id_hash -> Text,
        property_version -> Int8,
        creator_address -> Text,
        collection_name -> Text,
        name -> Text,
        transfer_type -> ActivityType,
        from_address -> Nullable<Text>,
        to_address -> Nullable<Text>,
        token_amount -> Int8,
        coin_type -> Nullable<Text>,
        coin_amount -> Nullable<Int8>,
        transaction_timestamp -> Timestamp,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    collections (id) {
        id -> Int4,
        chain_id -> Int4,
        slug -> Nullable<Varchar>,
        collection_id -> Varchar,
        collection_type -> Varchar,
        creator_address -> Varchar,
        collection_name -> Varchar,
        description -> Varchar,
        supply -> Int8,
        version -> Int8,
        metadata_uri -> Text,
        metadata -> Text,
        floor_sell_id -> Nullable<Int4>,
        floor_sell_value -> Nullable<Int8>,
        floor_sell_coin_id -> Nullable<Int4>,
        best_bid_id -> Nullable<Int4>,
        best_bid_value -> Nullable<Int8>,
        best_bid_coin_id -> Nullable<Int4>,
        verify -> Bool,
        last_metadata_sync -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    domains (id) {
        id -> Int4,
        chain_id -> Int8,
        hash_id -> Nullable<Varchar>,
        domain -> Varchar,
        domain_type -> Varchar,
        sub_domain -> Varchar,
        suffix -> Varchar,
        description -> Varchar,
        version -> Int8,
        metadata_uri -> Varchar,
        metadata_json -> Nullable<Varchar>,
        image -> Nullable<Varchar>,
        expired_time -> Nullable<Timestamp>,
        regest_time -> Nullable<Timestamp>,
        owner_address -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tokens (token_id) {
        chain_id -> Int8,
        token_id -> Varchar,
        collection_id -> Varchar,
        creator_address -> Varchar,
        collection_name -> Varchar,
        token_name -> Varchar,
        attributes -> Nullable<Text>,
        version -> Int8,
        payee_address -> Varchar,
        royalty_points_numerator -> Int8,
        royalty_points_denominator -> Int8,
        owner_address -> Nullable<Varchar>,
        metadata_uri -> Varchar,
        metadata_json -> Nullable<Varchar>,
        image -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(activities, collections, domains, tokens,);
