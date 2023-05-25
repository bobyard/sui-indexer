DO
$$
BEGIN
CREATE TYPE token_status AS ENUM ('exist', 'delete');
END
$$;


-- Your SQL goes here
CREATE TABLE tokens (
   "chain_id" BIGINT NOT NULL,
   "token_id" varchar(255) PRIMARY KEY,
   "collection_id" varchar(255)  NOT NULL,
   "creator_address" varchar(255)  NOT NULL,
   "collection_type" varchar(255)  NOT NULL,
   "collection_name" varchar(255)  NOT NULL,
   "token_name" varchar(255)  NOT NULL,
   "attributes" text,
   "version" BIGINT NOT NULL,
   "payee_address" varchar NOT NULL,
   "royalty_points_numerator" BIGINT NOT NULL,
   "royalty_points_denominator" BIGINT NOT NULL,
   "owner_address" varchar,
   "metadata_uri" varchar NOT NULL,
   "metadata_json" varchar,
   "image" varchar,
   "tx" varchar,
   "status" token_status,
   "created_at" timestamp DEFAULT now(),
   "updated_at" timestamp DEFAULT now()
);

CREATE INDEX token_id_index ON tokens (token_id);


