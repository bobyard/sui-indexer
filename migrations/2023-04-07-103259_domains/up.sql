-- Your SQL goes here
CREATE TABLE domains (
    "chain_id" int8 NOT NULL,
    "token_id" varchar(255) PRIMARY KEY,
    "collection_id" varchar(255)  NOT NULL,
    "domain" varchar(255) NOT NULL,
    "domain_type" varchar(255) NOT NULL,
    "sub_domain" varchar(255) NOT NULL,
    "suffix" varchar(255) NOT NULL,
    "description" varchar NOT NULL,
    "version" int8 NOT NULL,
    "metadata_uri" varchar NOT NULL,
    "metadata_json" varchar,
    "image" varchar,
    "expired_time" timestamp,
    "regest_time" timestamp,
    "owner_address" varchar,
    "created_at" timestamp DEFAULT now(),
    "updated_at" timestamp DEFAULT now()
);