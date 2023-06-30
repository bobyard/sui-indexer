-- Your SQL goes here
DO
$$
BEGIN
CREATE TYPE list_type AS ENUM ('listed', 'expired', 'canceled', 'sold');
END
$$;

DO
$$
BEGIN
CREATE TYPE market_type AS ENUM ('bob_yard', 'kiosk', 'origin_byte_kiosk');
END
$$;


CREATE TABLE lists (
     "id" SERIAL PRIMARY KEY,
     "chain_id" int8 NOT NULL,
     "coin_id" int4 NOT NULL,
     "list_id" varchar(255) NOT NULL,
     "list_time" timestamp NOT NULL,
     "token_id" varchar(255) NOT NULL,
     "seller_address" varchar(255) NOT NULL,
     "seller_value" int8 NOT NULL,
     "expire_time" timestamp,
     "list_type" list_type NOT NULL,
     "market_type" market_type NOT NULL,
     "created_at" timestamp DEFAULT now(),
     "updated_at" timestamp DEFAULT now()
);