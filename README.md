## Event Catch

TODO

### Teranfer_policy create
```shell
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "transfer_policy",
    ),
    name: Identifier(
        "TransferPolicyCreated",
    ),
    type_params: [
        Struct(
            StructTag {
                address: 38340dc5e020898767fc6fd7c9b2743f6baa22df5448e4b8be3881dcbf59d47e,
                module: Identifier(
                    "NinjasNFT",
                ),
                name: Identifier(
                    "NinJas",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0xb0dec51abb0bde9a148d2c490574a26acb2225992ba358e1d48c2806e40794c0"),
}
```


### list to kiosk

```shell
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "kiosk",
    ),
    name: Identifier(
        "ItemListed",
    ),
    type_params: [
        Struct(
            StructTag {
                address: 38340dc5e020898767fc6fd7c9b2743f6baa22df5448e4b8be3881dcbf59d47e,
                module: Identifier(
                    "NinjasNFT",
                ),
                name: Identifier(
                    "NinJas",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0x4e1de0cc1bdb59d2c38a594716b3ccd0aad257b5627161f34f27f324188e8e0f"),
    "kiosk": String("0x41fdf46e3f0d3781cc14b3bbc2620c5e07347049789b65288f5fb1043c709f70"),
    "price": String("111111111111"),
}
```


### item delist 

```shell
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "kiosk",
    ),
    name: Identifier(
        "ItemDelisted",
    ),
    type_params: [
        Struct(
            StructTag {
                address: 38340dc5e020898767fc6fd7c9b2743f6baa22df5448e4b8be3881dcbf59d47e,
                module: Identifier(
                    "NinjasNFT",
                ),
                name: Identifier(
                    "NinJas",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0x4e1de0cc1bdb59d2c38a594716b3ccd0aad257b5627161f34f27f324188e8e0f"),
    "kiosk": String("0x41fdf46e3f0d3781cc14b3bbc2620c5e07347049789b65288f5fb1043c709f70"),
}
```




[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "collection_id": String("0x1774322d4f9c2ccd769cf61830d37aa681489acc3e13791dc69318e93d423d72"),
    "object": String("0xf5a78c4dff1008d47a22b9469b31b3286a133ecb4cdcdc5040a86c003cf8219a"),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: a167c38f0ddf9b04f2cec53d8e638f58971e6e9dd9b258a7eaab40e801366a74,
    module: Identifier(
        "collection",
    ),
    name: Identifier(
        "MintCollectionEvent",
    ),
    type_params: [],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "collection_id": String("0xfad0fd1bc88220d6b4dcb0f0690de7a87be704b9298ceb7d47af29364a30c4f8"),
    "type_name": Object {
        "name": String("c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f::GenesisNFT::Genesis"),
    },
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "display",
    ),
    name: Identifier(
        "DisplayCreated",
    ),
    type_params: [
        Struct(
            StructTag {
                address: c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f,
                module: Identifier(
                    "GenesisNFT",
                ),
                name: Identifier(
                    "Genesis",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0x328cf28cd9b0e3743246804eb9222b8f5f9537de4f93194b5697980423429368"),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "display",
    ),
    name: Identifier(
        "VersionUpdated",
    ),
    type_params: [
        Struct(
            StructTag {
                address: c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f,
                module: Identifier(
                    "GenesisNFT",
                ),
                name: Identifier(
                    "Genesis",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "fields": Object {
        "contents": Array [
            Object {
                "key": String("name"),
                "value": String("{name}"),
            },
            Object {
                "key": String("link"),
                "value": String(""),
            },
            Object {
                "key": String("image_url"),
                "value": String("ipfs://{image_url}"),
            },
            Object {
                "key": String("description"),
                "value": String("Step into a world where imagination knows no bounds with Genesis NFT Cards, a groundbreaking collection that merges art, technology, and infinite possibilities. Each card is a testament to the boundless creativity and limitless potential that lies within us all."),
            },
            Object {
                "key": String("project_url"),
                "value": String("https://sui.best"),
            },
            Object {
                "key": String("creator"),
                "value": String("sui.best"),
            },
            Object {
                "key": String("tags"),
                "value": String("[\"Ticket\",\"Collectible\"]"),
            },
        ],
    },
    "id": String("0x328cf28cd9b0e3743246804eb9222b8f5f9537de4f93194b5697980423429368"),
    "version": Number(1),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "transfer_policy",
    ),
    name: Identifier(
        "TransferPolicyCreated",
    ),
    type_params: [
        Struct(
            StructTag {
                address: c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f,
                module: Identifier(
                    "GenesisNFT",
                ),
                name: Identifier(
                    "Genesis",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0xf73056f740cb5dea22530b66911c9f1f444616f74d0bf03ce53a05bc73ecdf13"),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 0000000000000000000000000000000000000000000000000000000000000002,
    module: Identifier(
        "transfer_policy",
    ),
    name: Identifier(
        "TransferPolicyCreated",
    ),
    type_params: [
        Struct(
            StructTag {
                address: c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f,
                module: Identifier(
                    "GenesisNFT",
                ),
                name: Identifier(
                    "Genesis",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "id": String("0x37c7de2649bb31ae3574a9b63d3711d71b302cab85f64ba47e7630a8e018029c"),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: 228f170efc35d36357c8d9a81661df0e2e463adbadcf79784ac13e060be1d75f,
    module: Identifier(
        "orderbook",
    ),
    name: Identifier(
        "OrderbookCreatedEvent",
    ),
    type_params: [],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "ft_type": String("0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"),
    "nft_type": String("c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f::GenesisNFT::Genesis"),
    "orderbook": String("0xe116015a36c84bb2ee0703a29dda3255e0d0c9be460110d88670cf072fc8b170"),
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:110] &e.type_ = StructTag {
    address: a167c38f0ddf9b04f2cec53d8e638f58971e6e9dd9b258a7eaab40e801366a74,
    module: Identifier(
        "mint_event",
    ),
    name: Identifier(
        "MintEvent",
    ),
    type_params: [
        Struct(
            StructTag {
                address: c2f06036458f53ed3b8937e86e52889365628d7acc1d66deb851a9b5beef602f,
                module: Identifier(
                    "GenesisNFT",
                ),
                name: Identifier(
                    "Genesis",
                ),
                type_params: [],
            },
        ),
    ],
}
[crates/sui-indexer/src/handlers/bobyard_event_catch.rs:112] &e.parsed_json = Object {
    "collection_id": String("0xfad0fd1bc88220d6b4dcb0f0690de7a87be704b9298ceb7d47af29364a30c4f8"),
    "object": String("0x6e85e75e1c7f2f956b98b06b3c4d494b6075992681c0b6e6970c5fa7cbc235a6"),
}