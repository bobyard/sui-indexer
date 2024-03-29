// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "activity_type"))]
    pub struct ActivityType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "list_type"))]
    pub struct ListType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "market_type"))]
    pub struct MarketType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "offer_type"))]
    pub struct OfferType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_type"))]
    pub struct OrderType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "token_status"))]
    pub struct TokenStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ActivityType;

    activities (id) {
        id -> Int8,
        chain_id -> Int8,
        version -> Int8,
        tx -> Nullable<Text>,
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
    collections (collection_id) {
        chain_id -> Int4,
        slug -> Nullable<Varchar>,
        collection_id -> Varchar,
        collection_type -> Varchar,
        creator_address -> Varchar,
        royaltie -> Nullable<Varchar>,
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
        tx -> Nullable<Varchar>,
        metadata -> Text,
        verify -> Bool,
        last_metadata_sync -> Int8,
        created_at -> Int8,
        updated_at -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ListType;
    use super::sql_types::MarketType;

    lists (id) {
        id -> Int4,
        chain_id -> Int8,
        coin_id -> Int4,
        list_id -> Varchar,
        list_time -> Timestamp,
        token_id -> Varchar,
        seller_address -> Varchar,
        seller_value -> Int8,
        expire_time -> Nullable<Timestamp>,
        list_type -> ListType,
        market_type -> MarketType,
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
        sell_time -> Timestamp,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TokenStatus;

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
        metadata_json -> Nullable<Text>,
        image -> Nullable<Varchar>,
        tx -> Nullable<Varchar>,
        status -> Nullable<TokenStatus>,
        created_at -> Int8,
        updated_at -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    activities,
    check_point,
    collections,
    lists,
    offers,
    orders,
    tokens,
);
