-- Your SQL goes here
CREATE TABLE orders (
    "id" SERIAL PRIMARY KEY,
    "chain_id" int8 NOT NULL,
    "coin_id" int4 NOT NULL,
    "list_id" varchar(255) NOT NULL,
    "token_id" varchar(255) NOT NULL,
    "offer_id" varchar(255),
    "seller_address" varchar(255) NOT NULL,
    "buyer_address" varchar(255) NOT NULL,
    "value" int8 NOT NULL,
    "expire_time" timestamp NOT NULL,
    "sell_time" timestamp NOT NULL,
    "created_at" timestamp DEFAULT now(),
    "updated_at" timestamp DEFAULT now()
);