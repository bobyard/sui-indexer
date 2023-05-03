// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "activity_type"))]
    pub struct ActivityType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "list_type"))]
    pub struct ListType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "offer_type"))]
    pub struct OfferType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_type"))]
    pub struct OrderType;
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
    check_point (chain_id) {
        chain_id -> Int8,
        version -> Int8,
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
        display_name -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        discord -> Nullable<Varchar>,
        twitter -> Nullable<Varchar>,
        icon -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
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
    domains (token_id) {
        chain_id -> Int8,
        token_id -> Varchar,
        collection_id -> Varchar,
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
    use diesel::sql_types::*;
    use super::sql_types::ListType;

    lists (id) {
        id -> Int4,
        chain_id -> Int8,
        coin_id -> Int4,
        list_id -> Varchar,
        list_time -> Timestamp,
        token_id -> Varchar,
        seller_address -> Varchar,
        seller_value -> Int8,
        expire_time -> Timestamp,
        list_type -> ListType,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OfferType;

    offers (id) {
        id -> Int4,
        chain_id -> Int8,
        coin_id -> Int4,
        offer_id -> Varchar,
        list_id -> Varchar,
        buyer_address -> Varchar,
        offer_value -> Int8,
        offer_type -> OfferType,
        expire_time -> Timestamp,
        offer_time -> Timestamp,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrderType;

    orders (id) {
        id -> Int4,
        chain_id -> Int8,
        coin_id -> Int4,
        list_id -> Varchar,
        token_id -> Varchar,
        offer_id -> Nullable<Varchar>,
        seller_address -> Varchar,
        buyer_address -> Varchar,
        value -> Int8,
        order_type -> OrderType,
        expire_time -> Timestamp,
        sell_time -> Timestamp,
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
        collection_type -> Varchar,
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

diesel::allow_tables_to_appear_in_same_query!(
    activities,
    check_point,
    collections,
    domains,
    lists,
    offers,
    orders,
    tokens,
);
