-- Your SQL goes here
CREATE TABLE tokens (
   "id" SERIAL PRIMARY KEY,
   "chain_id" int8 NOT NULL,
   "token_id" varchar(255) NOT NULL,
   "collection_id" varchar(255)  NOT NULL,
   "creator_address" varchar(255)  NOT NULL,
   "collection_name" varchar(255)  NOT NULL,
   "token_name" varchar(255)  NOT NULL,
   "attributes" text,
   "version" int8 NOT NULL,
   "payee_address" varchar NOT NULL,
   "royalty_points_numerator" int8 NOT NULL,
   "royalty_points_denominator" int8 NOT NULL,
   "owner_address" varchar,
   "metadata_uri" varchar NOT NULL,
   "metadata_json" varchar,
   "image" varchar,
   "created_at" timestamp DEFAULT now(),
   "updated_at" timestamp DEFAULT now()
);