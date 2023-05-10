-- Your SQL goes here
DO
$$
BEGIN
CREATE TYPE offer_type AS ENUM ('listed', 'expired', 'canceled', 'sold');
END
$$;

CREATE TABLE offers (
   "id" SERIAL PRIMARY KEY,
   "chain_id" int8 NOT NULL,
   "coin_id" int4 NOT NULL,
   "offer_id" varchar(255) NOT NULL,
   "list_id" varchar(255) NOT NULL,
   "buyer_address" varchar(255) NOT NULL,
   "offer_value" int8 NOT NULL,
   "offer_type" offer_type NOT NULL,
   "expire_time" timestamp NOT NULL,
   "offer_time" timestamp NOT NULL,
   "created_at" timestamp DEFAULT now(),
   "updated_at" timestamp DEFAULT now()
);