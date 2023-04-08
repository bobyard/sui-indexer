
DO
$$
BEGIN
CREATE TYPE activity_type AS ENUM ('created', 'minted', 'transferred', 'listed', 'canceled', 'sold');
END
$$;

-- Table Definition
CREATE TABLE activities (
   "id" BIGSERIAL PRIMARY KEY,
   "chain_id" int8 NOT NULL,
   "version" int8 NOT NULL,
   "event_account_address" text NOT NULL,
   "event_creation_number" int8 NOT NULL,
   "event_sequence_number" int8 NOT NULL,
   "collection_data_id_hash" text NOT NULL,
   "token_data_id_hash" text NOT NULL,
   "property_version" int8 NOT NULL,
   "creator_address" text NOT NULL,
   "collection_name" text NOT NULL,
   "name" text NOT NULL,
   "transfer_type" activity_type NOT NULL,
   "from_address" text,
   "to_address" text,
   "token_amount" int8 NOT NULL,
   "coin_type" text,
   "coin_amount" int8,
   "transaction_timestamp" timestamp NOT NULL,
   "created_at" timestamp DEFAULT now(),
   "updated_at" timestamp DEFAULT now()
);