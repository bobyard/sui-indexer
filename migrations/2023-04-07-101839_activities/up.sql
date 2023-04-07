-- Your SQL goes here
-- Table Definition
CREATE TABLE activities (
   "id" SERIAL PRIMARY KEY,
   "chain_id" int8 NOT NULL,
   "version" int8 NOT NULL,
   "event_account_address" text NOT NULL,
   "event_creation_number" int8 NOT NULL,
   "event_sequence_number" int8 NOT NULL,
   "collection_data_id_hash" text NOT NULL,
   "token_data_id_hash" text NOT NULL,
   "property_version" numeric NOT NULL,
   "creator_address" text NOT NULL,
   "collection_name" text NOT NULL,
   "name" text NOT NULL,
   "transfer_type" text NOT NULL,
   "from_address" text,
   "to_address" text,
   "token_amount" numeric NOT NULL,
   "coin_type" text,
   "coin_amount" numeric,
   "transaction_timestamp" timestamp NOT NULL,
   "created_at" timestamp DEFAULT now(),
   "updated_at" timestamp DEFAULT now()
);