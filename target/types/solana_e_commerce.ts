/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/solana_e_commerce.json`.
 */
export type SolanaECommerce = {
  "address": "5XZ74thixMBX2tQN9P3yLTugUK4YMdRLznDNa2mRdGNT",
  "metadata": {
    "name": "solanaECommerce",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addProductToKeywordIndex",
      "discriminator": [
        243,
        134,
        232,
        148,
        238,
        90,
        87,
        167
      ],
      "accounts": [
        {
          "name": "keywordRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              }
            ]
          }
        },
        {
          "name": "targetShard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              },
              {
                "kind": "const",
                "value": [
                  0,
                  0,
                  0,
                  0
                ]
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        },
        {
          "name": "productId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "addProductToPriceIndex",
      "discriminator": [
        42,
        43,
        51,
        228,
        199,
        12,
        242,
        97
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "priceIndex",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  105,
                  99,
                  101,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "arg",
                "path": "priceRangeStart"
              },
              {
                "kind": "arg",
                "path": "priceRangeEnd"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "price",
          "type": "u64"
        },
        {
          "name": "priceRangeStart",
          "type": "u64"
        },
        {
          "name": "priceRangeEnd",
          "type": "u64"
        }
      ]
    },
    {
      "name": "addProductToSalesIndex",
      "discriminator": [
        128,
        183,
        234,
        115,
        106,
        63,
        182,
        134
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "salesIndex",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  97,
                  108,
                  101,
                  115,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "arg",
                "path": "salesRangeStart"
              },
              {
                "kind": "arg",
                "path": "salesRangeEnd"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "salesRangeStart",
          "type": "u32"
        },
        {
          "name": "salesRangeEnd",
          "type": "u32"
        },
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "sales",
          "type": "u32"
        }
      ]
    },
    {
      "name": "allocateNewChunk",
      "discriminator": [
        201,
        223,
        225,
        254,
        40,
        177,
        20,
        69
      ],
      "accounts": [
        {
          "name": "globalRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  108,
                  111,
                  98,
                  97,
                  108,
                  95,
                  105,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "merchantAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "newChunk",
          "writable": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [],
      "returns": "pubkey"
    },
    {
      "name": "autoConfirmDelivery",
      "discriminator": [
        64,
        192,
        200,
        43,
        176,
        240,
        132,
        255
      ],
      "accounts": [
        {
          "name": "order",
          "writable": true
        },
        {
          "name": "orderStats",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        },
        {
          "name": "merchant",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchant.owner",
                "account": "merchant"
              }
            ]
          }
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "batchGenerateIds",
      "discriminator": [
        200,
        125,
        236,
        30,
        205,
        194,
        50,
        160
      ],
      "accounts": [
        {
          "name": "merchantAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "activeChunk",
          "writable": true
        }
      ],
      "args": [
        {
          "name": "count",
          "type": "u16"
        }
      ],
      "returns": {
        "vec": "u64"
      }
    },
    {
      "name": "closeIdChunk",
      "discriminator": [
        107,
        104,
        198,
        252,
        175,
        239,
        166,
        232
      ],
      "accounts": [
        {
          "name": "idChunk",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  95,
                  99,
                  104,
                  117,
                  110,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "merchantKey"
              },
              {
                "kind": "arg",
                "path": "chunkIndex"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "merchantKey",
          "type": "pubkey"
        },
        {
          "name": "chunkIndex",
          "type": "u32"
        },
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closeKeywordRoot",
      "discriminator": [
        141,
        242,
        59,
        53,
        78,
        180,
        2,
        117
      ],
      "accounts": [
        {
          "name": "keywordRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        },
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closeKeywordShard",
      "discriminator": [
        168,
        27,
        232,
        181,
        155,
        23,
        16,
        68
      ],
      "accounts": [
        {
          "name": "keywordShard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              },
              {
                "kind": "arg",
                "path": "shardIndex"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        },
        {
          "name": "shardIndex",
          "type": "u32"
        },
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closeMerchant",
      "discriminator": [
        138,
        96,
        102,
        11,
        220,
        136,
        154,
        11
      ],
      "accounts": [
        {
          "name": "merchantInfo",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closeMerchantIdAccount",
      "discriminator": [
        144,
        146,
        124,
        219,
        173,
        192,
        184,
        125
      ],
      "accounts": [
        {
          "name": "merchantIdAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "merchantKey"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "merchantKey",
          "type": "pubkey"
        },
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closePaymentConfig",
      "discriminator": [
        201,
        184,
        174,
        187,
        207,
        83,
        9,
        190
      ],
      "accounts": [
        {
          "name": "paymentConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "closeSystemConfig",
      "discriminator": [
        6,
        107,
        135,
        27,
        8,
        208,
        165,
        70
      ],
      "accounts": [
        {
          "name": "systemConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "confirmDelivery",
      "discriminator": [
        11,
        109,
        227,
        53,
        179,
        190,
        88,
        155
      ],
      "accounts": [
        {
          "name": "order",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114
                ]
              },
              {
                "kind": "arg",
                "path": "buyerKey"
              },
              {
                "kind": "arg",
                "path": "merchant"
              },
              {
                "kind": "arg",
                "path": "productId"
              },
              {
                "kind": "arg",
                "path": "timestamp"
              }
            ]
          }
        },
        {
          "name": "orderStats",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        },
        {
          "name": "merchantInfo",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "order.merchant",
                "account": "order"
              }
            ]
          }
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "programTokenAccount",
          "writable": true
        },
        {
          "name": "depositEscrowAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  112,
                  111,
                  115,
                  105,
                  116,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              }
            ]
          }
        },
        {
          "name": "programAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "buyer",
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "createKeywordShard",
      "discriminator": [
        251,
        187,
        156,
        45,
        87,
        245,
        118,
        199
      ],
      "accounts": [
        {
          "name": "keywordRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              }
            ]
          }
        },
        {
          "name": "prevShard",
          "writable": true
        },
        {
          "name": "newShard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              },
              {
                "kind": "arg",
                "path": "shardIndex"
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        },
        {
          "name": "shardIndex",
          "type": "u32"
        }
      ]
    },
    {
      "name": "createOrder",
      "discriminator": [
        141,
        54,
        37,
        207,
        237,
        210,
        250,
        215
      ],
      "accounts": [
        {
          "name": "userPurchaseCount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  112,
                  117,
                  114,
                  99,
                  104,
                  97,
                  115,
                  101,
                  95,
                  99,
                  111,
                  117,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "buyer"
              }
            ]
          }
        },
        {
          "name": "order",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "account",
                "path": "merchant"
              },
              {
                "kind": "arg",
                "path": "productId"
              },
              {
                "kind": "account",
                "path": "user_purchase_count.purchase_count",
                "account": "userPurchaseCount"
              }
            ]
          }
        },
        {
          "name": "orderStats",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        },
        {
          "name": "product",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "merchant",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchant.owner",
                "account": "merchant"
              }
            ]
          }
        },
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "quantity",
          "type": "u32"
        },
        {
          "name": "shippingAddress",
          "type": "string"
        },
        {
          "name": "notes",
          "type": "string"
        },
        {
          "name": "transactionSignature",
          "type": "string"
        }
      ]
    },
    {
      "name": "createProductBase",
      "discriminator": [
        153,
        35,
        251,
        66,
        109,
        198,
        26,
        148
      ],
      "accounts": [
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "globalRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  108,
                  111,
                  98,
                  97,
                  108,
                  95,
                  105,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "merchantIdAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "activeChunk",
          "writable": true
        },
        {
          "name": "paymentConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "productAccount",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "description",
          "type": "string"
        },
        {
          "name": "price",
          "type": "u64"
        },
        {
          "name": "keywords",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "inventory",
          "type": "u64"
        },
        {
          "name": "paymentToken",
          "type": "pubkey"
        },
        {
          "name": "shippingLocation",
          "type": "string"
        }
      ],
      "returns": "u64"
    },
    {
      "name": "createProductExtended",
      "discriminator": [
        14,
        37,
        124,
        155,
        109,
        26,
        201,
        26
      ],
      "accounts": [
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "productExtended",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116,
                  95,
                  101,
                  120,
                  116,
                  101,
                  110,
                  100,
                  101,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "productBase",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "imageVideoUrls",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "salesRegions",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "logisticsMethods",
          "type": {
            "vec": "string"
          }
        }
      ]
    },
    {
      "name": "deductMerchantDeposit",
      "discriminator": [
        190,
        219,
        68,
        198,
        234,
        15,
        12,
        90
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "merchant",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchantOwner"
              }
            ]
          }
        },
        {
          "name": "merchantOwner",
          "docs": [
            "商户所有者公钥（用于PDA计算）"
          ]
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "depositTokenMint"
        },
        {
          "name": "depositEscrowAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  112,
                  111,
                  115,
                  105,
                  116,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "depositTokenMint"
              }
            ]
          }
        },
        {
          "name": "adminTokenAccount",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "reason",
          "type": "string"
        }
      ]
    },
    {
      "name": "deleteProduct",
      "discriminator": [
        173,
        212,
        141,
        230,
        33,
        82,
        166,
        25
      ],
      "accounts": [
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "product",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "hardDelete",
          "type": "bool"
        },
        {
          "name": "force",
          "type": "bool"
        }
      ]
    },
    {
      "name": "forceCloseSystemConfig",
      "discriminator": [
        189,
        71,
        100,
        160,
        167,
        174,
        235,
        108
      ],
      "accounts": [
        {
          "name": "systemConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "beneficiary",
          "writable": true,
          "signer": true
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "generateProductId",
      "discriminator": [
        204,
        200,
        95,
        199,
        43,
        150,
        179,
        74
      ],
      "accounts": [
        {
          "name": "merchantAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "activeChunk",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  95,
                  99,
                  104,
                  117,
                  110,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              },
              {
                "kind": "account",
                "path": "merchant_account.last_chunk_index",
                "account": "merchantIdAccount"
              }
            ]
          }
        }
      ],
      "args": [],
      "returns": "u64"
    },
    {
      "name": "getMerchantDepositInfo",
      "discriminator": [
        233,
        103,
        193,
        9,
        136,
        7,
        193,
        141
      ],
      "accounts": [
        {
          "name": "merchant",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchantOwner"
              }
            ]
          }
        },
        {
          "name": "merchantOwner",
          "signer": true
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "depositTokenMint"
        }
      ],
      "args": [],
      "returns": {
        "defined": {
          "name": "merchantDepositInfo"
        }
      }
    },
    {
      "name": "getMerchantStats",
      "discriminator": [
        120,
        49,
        39,
        98,
        142,
        165,
        84,
        158
      ],
      "accounts": [
        {
          "name": "merchantInfo",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": [],
      "returns": {
        "defined": {
          "name": "merchantStats"
        }
      }
    },
    {
      "name": "getOrderStats",
      "discriminator": [
        174,
        2,
        36,
        228,
        213,
        73,
        204,
        188
      ],
      "accounts": [
        {
          "name": "orderStats",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "initializeKeywordIndex",
      "discriminator": [
        36,
        128,
        212,
        91,
        103,
        123,
        46,
        6
      ],
      "accounts": [
        {
          "name": "keywordRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              }
            ]
          }
        },
        {
          "name": "firstShard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              },
              {
                "kind": "const",
                "value": [
                  0,
                  0,
                  0,
                  0
                ]
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        }
      ]
    },
    {
      "name": "initializeOrderStats",
      "discriminator": [
        188,
        141,
        99,
        39,
        119,
        215,
        43,
        254
      ],
      "accounts": [
        {
          "name": "orderStats",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "initializePaymentSystem",
      "discriminator": [
        115,
        181,
        85,
        189,
        43,
        0,
        123,
        183
      ],
      "accounts": [
        {
          "name": "paymentConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "supportedTokens",
          "type": {
            "vec": {
              "defined": {
                "name": "supportedToken"
              }
            }
          }
        },
        {
          "name": "feeRate",
          "type": "u16"
        },
        {
          "name": "feeRecipient",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "initializeProgramTokenAccount",
      "discriminator": [
        195,
        68,
        47,
        163,
        248,
        214,
        47,
        175
      ],
      "accounts": [
        {
          "name": "programTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  116,
                  111,
                  107,
                  101,
                  110,
                  95,
                  97,
                  99,
                  99,
                  111,
                  117,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "paymentTokenMint"
              }
            ]
          }
        },
        {
          "name": "programAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "paymentTokenMint"
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "initializeSalesIndex",
      "discriminator": [
        225,
        105,
        245,
        176,
        194,
        41,
        219,
        31
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "salesIndex",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  97,
                  108,
                  101,
                  115,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "arg",
                "path": "salesRangeStart"
              },
              {
                "kind": "arg",
                "path": "salesRangeEnd"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "salesRangeStart",
          "type": "u32"
        },
        {
          "name": "salesRangeEnd",
          "type": "u32"
        }
      ]
    },
    {
      "name": "initializeSystem",
      "discriminator": [
        50,
        173,
        248,
        140,
        202,
        35,
        141,
        150
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "globalRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  108,
                  111,
                  98,
                  97,
                  108,
                  95,
                  105,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "systemConfig"
            }
          }
        }
      ]
    },
    {
      "name": "initializeSystemConfig",
      "discriminator": [
        43,
        153,
        196,
        116,
        233,
        36,
        208,
        246
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "config",
          "type": {
            "defined": {
              "name": "systemConfig"
            }
          }
        }
      ]
    },
    {
      "name": "isIdExists",
      "discriminator": [
        242,
        72,
        82,
        8,
        248,
        209,
        123,
        135
      ],
      "accounts": [
        {
          "name": "merchantAccount",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "merchant",
          "signer": true
        },
        {
          "name": "idChunk"
        }
      ],
      "args": [
        {
          "name": "id",
          "type": "u64"
        }
      ],
      "returns": "bool"
    },
    {
      "name": "manageDeposit",
      "discriminator": [
        229,
        142,
        115,
        148,
        168,
        136,
        50,
        56
      ],
      "accounts": [
        {
          "name": "merchantOwner",
          "writable": true,
          "signer": true
        },
        {
          "name": "merchant",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchantOwner"
              }
            ]
          }
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "merchantTokenAccount",
          "writable": true
        },
        {
          "name": "depositTokenMint"
        },
        {
          "name": "depositEscrowAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  112,
                  111,
                  115,
                  105,
                  116,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "depositTokenMint"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "purchaseProductEscrow",
      "discriminator": [
        54,
        19,
        131,
        52,
        56,
        59,
        154,
        129
      ],
      "accounts": [
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "product",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "programTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  116,
                  111,
                  107,
                  101,
                  110,
                  95,
                  97,
                  99,
                  99,
                  111,
                  117,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "paymentTokenMint"
              }
            ]
          }
        },
        {
          "name": "programAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "buyerTokenAccount",
          "writable": true
        },
        {
          "name": "paymentTokenMint"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "refundOrder",
      "discriminator": [
        164,
        168,
        47,
        144,
        154,
        1,
        241,
        255
      ],
      "accounts": [
        {
          "name": "order",
          "writable": true
        },
        {
          "name": "programTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  116,
                  111,
                  107,
                  101,
                  110,
                  95,
                  97,
                  99,
                  99,
                  111,
                  117,
                  110,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "buyerTokenAccount",
          "writable": true
        },
        {
          "name": "programAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  103,
                  114,
                  97,
                  109,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "buyer",
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "refundReason",
          "type": "string"
        }
      ]
    },
    {
      "name": "registerMerchantAtomic",
      "discriminator": [
        32,
        44,
        174,
        126,
        35,
        71,
        237,
        6
      ],
      "accounts": [
        {
          "name": "merchant",
          "signer": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "globalRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  108,
                  111,
                  98,
                  97,
                  108,
                  95,
                  105,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "merchantInfo",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "merchantIdAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "merchant"
              }
            ]
          }
        },
        {
          "name": "initialChunk",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "description",
          "type": "string"
        }
      ]
    },
    {
      "name": "removeProductFromKeywordIndex",
      "discriminator": [
        155,
        169,
        181,
        6,
        131,
        34,
        247,
        171
      ],
      "accounts": [
        {
          "name": "keywordRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              }
            ]
          }
        },
        {
          "name": "targetShard",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  107,
                  101,
                  121,
                  119,
                  111,
                  114,
                  100,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "keyword"
              },
              {
                "kind": "const",
                "value": [
                  0,
                  0,
                  0,
                  0
                ]
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "keyword",
          "type": "string"
        },
        {
          "name": "productId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "removeProductFromPriceIndex",
      "discriminator": [
        110,
        178,
        178,
        193,
        139,
        225,
        59,
        52
      ],
      "accounts": [
        {
          "name": "priceNode",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  105,
                  99,
                  101,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "account",
                "path": "price_node.price_range_start",
                "account": "priceIndexNode"
              },
              {
                "kind": "account",
                "path": "price_node.price_range_end",
                "account": "priceIndexNode"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "removeProductFromSalesIndex",
      "discriminator": [
        49,
        37,
        83,
        165,
        184,
        118,
        207,
        143
      ],
      "accounts": [
        {
          "name": "salesNode",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  97,
                  108,
                  101,
                  115,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "account",
                "path": "sales_node.sales_range_start",
                "account": "salesIndexNode"
              },
              {
                "kind": "account",
                "path": "sales_node.sales_range_end",
                "account": "salesIndexNode"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "shipOrder",
      "discriminator": [
        2,
        191,
        151,
        45,
        16,
        248,
        97,
        142
      ],
      "accounts": [
        {
          "name": "order",
          "writable": true
        },
        {
          "name": "orderStats",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  115
                ]
              }
            ]
          }
        },
        {
          "name": "merchant",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchant.owner",
                "account": "merchant"
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "trackingNumber",
          "type": "string"
        }
      ]
    },
    {
      "name": "splitPriceNode",
      "discriminator": [
        234,
        133,
        114,
        69,
        187,
        147,
        66,
        41
      ],
      "accounts": [
        {
          "name": "priceNode",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  105,
                  99,
                  101,
                  95,
                  105,
                  110,
                  100,
                  101,
                  120
                ]
              },
              {
                "kind": "arg",
                "path": "priceRangeStart"
              },
              {
                "kind": "arg",
                "path": "priceRangeEnd"
              }
            ]
          }
        },
        {
          "name": "newPriceNode",
          "writable": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "priceRangeStart",
          "type": "u64"
        },
        {
          "name": "priceRangeEnd",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateDepositRequirement",
      "discriminator": [
        125,
        163,
        42,
        239,
        221,
        28,
        211,
        74
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "newRequirement",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateFeeRate",
      "discriminator": [
        195,
        241,
        226,
        216,
        102,
        1,
        5,
        122
      ],
      "accounts": [
        {
          "name": "paymentConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "feeRate",
          "type": "u16"
        }
      ]
    },
    {
      "name": "updateMerchantInfo",
      "discriminator": [
        254,
        42,
        5,
        10,
        82,
        188,
        116,
        32
      ],
      "accounts": [
        {
          "name": "merchantInfo",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "owner"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "name",
          "type": {
            "option": "string"
          }
        },
        {
          "name": "description",
          "type": {
            "option": "string"
          }
        }
      ]
    },
    {
      "name": "updateProduct",
      "discriminator": [
        139,
        180,
        241,
        126,
        123,
        240,
        13,
        224
      ],
      "accounts": [
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "product",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "productExtended",
          "writable": true,
          "optional": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116,
                  95,
                  101,
                  120,
                  116,
                  101,
                  110,
                  100,
                  101,
                  100
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        },
        {
          "name": "paymentConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "name",
          "type": {
            "option": "string"
          }
        },
        {
          "name": "description",
          "type": {
            "option": "string"
          }
        },
        {
          "name": "price",
          "type": {
            "option": "u64"
          }
        },
        {
          "name": "keywords",
          "type": {
            "option": {
              "vec": "string"
            }
          }
        },
        {
          "name": "inventory",
          "type": {
            "option": "u64"
          }
        },
        {
          "name": "paymentToken",
          "type": {
            "option": "pubkey"
          }
        },
        {
          "name": "imageVideoUrls",
          "type": {
            "option": {
              "vec": "string"
            }
          }
        },
        {
          "name": "shippingLocation",
          "type": {
            "option": "string"
          }
        },
        {
          "name": "salesRegions",
          "type": {
            "option": {
              "vec": "string"
            }
          }
        },
        {
          "name": "logisticsMethods",
          "type": {
            "option": {
              "vec": "string"
            }
          }
        }
      ]
    },
    {
      "name": "updateProductPrice",
      "discriminator": [
        224,
        194,
        162,
        206,
        125,
        101,
        44,
        234
      ],
      "accounts": [
        {
          "name": "merchant",
          "writable": true,
          "signer": true
        },
        {
          "name": "product",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "newPrice",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateProductSalesIndex",
      "discriminator": [
        73,
        101,
        231,
        61,
        87,
        30,
        208,
        221
      ],
      "accounts": [
        {
          "name": "oldSalesNode",
          "writable": true
        },
        {
          "name": "newSalesNode",
          "writable": true
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "oldSales",
          "type": "u32"
        },
        {
          "name": "newSales",
          "type": "u32"
        }
      ]
    },
    {
      "name": "updateSalesCount",
      "discriminator": [
        174,
        185,
        96,
        64,
        196,
        81,
        222,
        245
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "product",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "productId"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "productId",
          "type": "u64"
        },
        {
          "name": "salesIncrement",
          "type": "u32"
        }
      ]
    },
    {
      "name": "updateSupportedTokens",
      "discriminator": [
        224,
        58,
        155,
        90,
        86,
        65,
        78,
        199
      ],
      "accounts": [
        {
          "name": "paymentConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  121,
                  109,
                  101,
                  110,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "supportedTokens",
          "type": {
            "vec": {
              "defined": {
                "name": "supportedToken"
              }
            }
          }
        }
      ]
    },
    {
      "name": "withdrawMerchantDeposit",
      "discriminator": [
        20,
        19,
        35,
        124,
        181,
        39,
        149,
        187
      ],
      "accounts": [
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "merchant",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  101,
                  114,
                  99,
                  104,
                  97,
                  110,
                  116,
                  95,
                  105,
                  110,
                  102,
                  111
                ]
              },
              {
                "kind": "account",
                "path": "merchantOwner"
              }
            ]
          }
        },
        {
          "name": "merchantOwner",
          "docs": [
            "商户所有者（签名者）"
          ],
          "signer": true
        },
        {
          "name": "systemConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  121,
                  115,
                  116,
                  101,
                  109,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "recipientTokenAccount",
          "writable": true
        },
        {
          "name": "depositTokenMint"
        },
        {
          "name": "depositEscrowAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  112,
                  111,
                  115,
                  105,
                  116,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "depositTokenMint"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "globalIdRoot",
      "discriminator": [
        160,
        52,
        193,
        100,
        232,
        227,
        104,
        89
      ]
    },
    {
      "name": "idChunk",
      "discriminator": [
        181,
        3,
        189,
        241,
        130,
        58,
        50,
        91
      ]
    },
    {
      "name": "keywordRoot",
      "discriminator": [
        37,
        2,
        46,
        224,
        85,
        99,
        149,
        246
      ]
    },
    {
      "name": "keywordShard",
      "discriminator": [
        65,
        240,
        150,
        126,
        194,
        124,
        129,
        127
      ]
    },
    {
      "name": "merchant",
      "discriminator": [
        71,
        235,
        30,
        40,
        231,
        21,
        32,
        64
      ]
    },
    {
      "name": "merchantIdAccount",
      "discriminator": [
        101,
        205,
        154,
        159,
        88,
        102,
        213,
        82
      ]
    },
    {
      "name": "order",
      "discriminator": [
        134,
        173,
        223,
        185,
        77,
        86,
        28,
        51
      ]
    },
    {
      "name": "orderStats",
      "discriminator": [
        158,
        4,
        69,
        167,
        243,
        156,
        250,
        225
      ]
    },
    {
      "name": "paymentConfig",
      "discriminator": [
        252,
        166,
        185,
        239,
        186,
        79,
        212,
        152
      ]
    },
    {
      "name": "priceIndexNode",
      "discriminator": [
        140,
        233,
        236,
        104,
        100,
        161,
        6,
        120
      ]
    },
    {
      "name": "productBase",
      "discriminator": [
        136,
        90,
        110,
        91,
        159,
        123,
        158,
        70
      ]
    },
    {
      "name": "productExtended",
      "discriminator": [
        88,
        116,
        103,
        117,
        171,
        111,
        103,
        79
      ]
    },
    {
      "name": "salesIndexNode",
      "discriminator": [
        185,
        183,
        45,
        92,
        72,
        101,
        157,
        210
      ]
    },
    {
      "name": "systemConfig",
      "discriminator": [
        218,
        150,
        16,
        126,
        102,
        185,
        75,
        1
      ]
    },
    {
      "name": "userPurchaseCount",
      "discriminator": [
        218,
        163,
        123,
        229,
        41,
        108,
        161,
        253
      ]
    }
  ],
  "events": [
    {
      "name": "merchantRegisteredAtomic",
      "discriminator": [
        166,
        239,
        156,
        104,
        215,
        191,
        159,
        14
      ]
    },
    {
      "name": "productEvent",
      "discriminator": [
        254,
        182,
        14,
        1,
        15,
        26,
        228,
        34
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "missingKeywordAccount",
      "msg": ""
    },
    {
      "code": 6001,
      "name": "tooManyKeywords",
      "msg": ""
    },
    {
      "code": 6002,
      "name": "shardFull",
      "msg": ""
    },
    {
      "code": 6003,
      "name": "idGenerationFailed",
      "msg": "id"
    },
    {
      "code": 6004,
      "name": "rentCalculationFailed",
      "msg": ""
    },
    {
      "code": 6005,
      "name": "merchantNotRegistered",
      "msg": ""
    },
    {
      "code": 6006,
      "name": "idAlreadyInUse",
      "msg": "id"
    },
    {
      "code": 6007,
      "name": "idNotFound",
      "msg": "id"
    },
    {
      "code": 6008,
      "name": "idRangeOverflow",
      "msg": "id"
    },
    {
      "code": 6009,
      "name": "noAvailableId",
      "msg": "id"
    },
    {
      "code": 6010,
      "name": "invalidId",
      "msg": "id"
    },
    {
      "code": 6011,
      "name": "integerOverflow",
      "msg": ""
    },
    {
      "code": 6012,
      "name": "productNotFound",
      "msg": ""
    },
    {
      "code": 6013,
      "name": "invalidProduct",
      "msg": ""
    },
    {
      "code": 6014,
      "name": "invalidProductAccount",
      "msg": ""
    },
    {
      "code": 6015,
      "name": "invalidPrice",
      "msg": ""
    },
    {
      "code": 6016,
      "name": "invalidAmount",
      "msg": "0"
    },
    {
      "code": 6017,
      "name": "invalidProductName",
      "msg": ""
    },
    {
      "code": 6018,
      "name": "invalidProductNameLength",
      "msg": ""
    },
    {
      "code": 6019,
      "name": "invalidProductDescription",
      "msg": ""
    },
    {
      "code": 6020,
      "name": "invalidProductDescriptionLength",
      "msg": ""
    },
    {
      "code": 6021,
      "name": "tooManyImageUrls",
      "msg": "url"
    },
    {
      "code": 6022,
      "name": "tooManySalesRegions",
      "msg": ""
    },
    {
      "code": 6023,
      "name": "tooManyLogisticsMethods",
      "msg": ""
    },
    {
      "code": 6024,
      "name": "invalidMerchant",
      "msg": ""
    },
    {
      "code": 6025,
      "name": "invalidMerchantNameLength",
      "msg": ""
    },
    {
      "code": 6026,
      "name": "invalidMerchantDescriptionLength",
      "msg": ""
    },
    {
      "code": 6027,
      "name": "unauthorizedMerchant",
      "msg": ""
    },
    {
      "code": 6028,
      "name": "invalidKeyword",
      "msg": ""
    },
    {
      "code": 6029,
      "name": "invalidKeywordLength",
      "msg": ""
    },
    {
      "code": 6030,
      "name": "invalidKeywordCount",
      "msg": ""
    },
    {
      "code": 6031,
      "name": "duplicateKeyword",
      "msg": ""
    },
    {
      "code": 6032,
      "name": "shardIsFull",
      "msg": ""
    },
    {
      "code": 6033,
      "name": "invalidShardIndex",
      "msg": ""
    },
    {
      "code": 6034,
      "name": "priceIndexNodeNotFound",
      "msg": ""
    },
    {
      "code": 6035,
      "name": "salesIndexNodeNotFound",
      "msg": ""
    },
    {
      "code": 6036,
      "name": "invalidPriceRange",
      "msg": ""
    },
    {
      "code": 6037,
      "name": "invalidSalesRange",
      "msg": ""
    },
    {
      "code": 6038,
      "name": "bloomFilterUpdateFailed",
      "msg": ""
    },
    {
      "code": 6039,
      "name": "keywordIndexNotEmpty",
      "msg": ""
    },
    {
      "code": 6040,
      "name": "keywordShardNotEmpty",
      "msg": ""
    },
    {
      "code": 6041,
      "name": "merchantHasActiveProducts",
      "msg": ""
    },
    {
      "code": 6042,
      "name": "idChunkNotEmpty",
      "msg": "id"
    },
    {
      "code": 6043,
      "name": "merchantIdAccountNotEmpty",
      "msg": "id"
    },
    {
      "code": 6044,
      "name": "unsupportedToken",
      "msg": ""
    },
    {
      "code": 6045,
      "name": "insufficientTokenBalance",
      "msg": ""
    },
    {
      "code": 6046,
      "name": "insufficientSolBalance",
      "msg": "sol"
    },
    {
      "code": 6047,
      "name": "invalidTokenAmount",
      "msg": ""
    },
    {
      "code": 6048,
      "name": "tokenTransferFailed",
      "msg": ""
    },
    {
      "code": 6049,
      "name": "feeCalculationError",
      "msg": ""
    },
    {
      "code": 6050,
      "name": "paymentConfigNotFound",
      "msg": ""
    },
    {
      "code": 6051,
      "name": "tokenNotActive",
      "msg": ""
    },
    {
      "code": 6052,
      "name": "belowMinimumAmount",
      "msg": ""
    },
    {
      "code": 6053,
      "name": "productCreationFailed",
      "msg": ""
    },
    {
      "code": 6054,
      "name": "atomicOperationFailed",
      "msg": ""
    },
    {
      "code": 6055,
      "name": "invalidFeeRate",
      "msg": ""
    },
    {
      "code": 6056,
      "name": "tooManyTokens",
      "msg": ""
    },
    {
      "code": 6057,
      "name": "invalidTokenSymbol",
      "msg": ""
    },
    {
      "code": 6058,
      "name": "invalidTokenDecimals",
      "msg": ""
    },
    {
      "code": 6059,
      "name": "invalidOrderStatus",
      "msg": ""
    },
    {
      "code": 6060,
      "name": "invalidPaymentMethod",
      "msg": ""
    },
    {
      "code": 6061,
      "name": "orderNotFound",
      "msg": ""
    },
    {
      "code": 6062,
      "name": "invalidOrderQuantity",
      "msg": ""
    },
    {
      "code": 6063,
      "name": "invalidOrderPrice",
      "msg": ""
    },
    {
      "code": 6064,
      "name": "invalidOrderTotalAmount",
      "msg": ""
    },
    {
      "code": 6065,
      "name": "invalidOrderTokenPrice",
      "msg": ""
    },
    {
      "code": 6066,
      "name": "invalidOrderTokenTotalAmount",
      "msg": ""
    },
    {
      "code": 6067,
      "name": "invalidShippingAddressLength",
      "msg": ""
    },
    {
      "code": 6068,
      "name": "invalidOrderNotesLength",
      "msg": ""
    },
    {
      "code": 6069,
      "name": "invalidTransactionSignature",
      "msg": ""
    },
    {
      "code": 6070,
      "name": "invalidOrderStatusTransition",
      "msg": ""
    },
    {
      "code": 6071,
      "name": "orderCannotBeModified",
      "msg": ""
    },
    {
      "code": 6072,
      "name": "orderCannotBeRefunded",
      "msg": ""
    },
    {
      "code": 6073,
      "name": "orderAlreadyExists",
      "msg": ""
    },
    {
      "code": 6074,
      "name": "unauthorized",
      "msg": ""
    },
    {
      "code": 6075,
      "name": "invalidTimestamp",
      "msg": ""
    },
    {
      "code": 6076,
      "name": "invalidAccountOwner",
      "msg": ""
    },
    {
      "code": 6077,
      "name": "invalidAccountData",
      "msg": ""
    },
    {
      "code": 6078,
      "name": "invalidAccountSize",
      "msg": ""
    },
    {
      "code": 6079,
      "name": "invalidPda",
      "msg": "pda"
    },
    {
      "code": 6080,
      "name": "invalidAccountSeeds",
      "msg": ""
    },
    {
      "code": 6081,
      "name": "invalidAccountBump",
      "msg": "bump"
    },
    {
      "code": 6082,
      "name": "insufficientFunds",
      "msg": ""
    },
    {
      "code": 6083,
      "name": "invalidActiveChunk",
      "msg": ""
    },
    {
      "code": 6084,
      "name": "accountDiscriminatorMismatch",
      "msg": ""
    },
    {
      "code": 6085,
      "name": "insufficientAccounts",
      "msg": ""
    },
    {
      "code": 6086,
      "name": "insufficientDeposit",
      "msg": ""
    },
    {
      "code": 6087,
      "name": "insufficientLockedDeposit",
      "msg": ""
    },
    {
      "code": 6088,
      "name": "invalidDepositToken",
      "msg": ""
    },
    {
      "code": 6089,
      "name": "invalidDepositAmount",
      "msg": ""
    },
    {
      "code": 6090,
      "name": "merchantDepositInsufficient",
      "msg": "商户保证金不足，无法进行交易"
    },
    {
      "code": 6091,
      "name": "depositAlreadyLocked",
      "msg": ""
    },
    {
      "code": 6092,
      "name": "depositNotLocked",
      "msg": ""
    },
    {
      "code": 6093,
      "name": "arithmeticOverflow",
      "msg": ""
    },
    {
      "code": 6094,
      "name": "arithmeticUnderflow",
      "msg": ""
    },
    {
      "code": 6095,
      "name": "trackingNumberRequired",
      "msg": ""
    },
    {
      "code": 6096,
      "name": "invalidTrackingNumber",
      "msg": ""
    }
  ],
  "types": [
    {
      "name": "globalIdRoot",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "lastMerchantId",
            "type": "u32"
          },
          {
            "name": "lastGlobalId",
            "type": "u64"
          },
          {
            "name": "chunkSize",
            "type": "u32"
          },
          {
            "name": "merchants",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "maxProductsPerShard",
            "type": "u16"
          },
          {
            "name": "maxKeywordsPerProduct",
            "type": "u8"
          },
          {
            "name": "bloomFilterSize",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "idChunk",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "merchantId",
            "type": "u32"
          },
          {
            "name": "chunkIndex",
            "type": "u32"
          },
          {
            "name": "startId",
            "type": "u64"
          },
          {
            "name": "endId",
            "type": "u64"
          },
          {
            "name": "nextAvailable",
            "type": "u64"
          },
          {
            "name": "bitmap",
            "type": "bytes"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "keywordRoot",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "keyword",
            "type": "string"
          },
          {
            "name": "totalShards",
            "type": "u8"
          },
          {
            "name": "firstShard",
            "type": "pubkey"
          },
          {
            "name": "lastShard",
            "type": "pubkey"
          },
          {
            "name": "totalProducts",
            "type": "u32"
          },
          {
            "name": "bloomFilter",
            "type": {
              "array": [
                "u8",
                256
              ]
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "keywordShard",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "keyword",
            "type": "string"
          },
          {
            "name": "shardIndex",
            "type": "u32"
          },
          {
            "name": "prevShard",
            "type": "pubkey"
          },
          {
            "name": "nextShard",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "productIds",
            "type": {
              "vec": "u64"
            }
          },
          {
            "name": "minId",
            "type": "u64"
          },
          {
            "name": "maxId",
            "type": "u64"
          },
          {
            "name": "bloomSummary",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "merchant",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "description",
            "type": "string"
          },
          {
            "name": "productCount",
            "type": "u64"
          },
          {
            "name": "totalSales",
            "type": "u64"
          },
          {
            "name": "isActive",
            "type": "bool"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "depositAmount",
            "type": "u64"
          },
          {
            "name": "depositTokenMint",
            "type": "pubkey"
          },
          {
            "name": "depositLocked",
            "type": "u64"
          },
          {
            "name": "depositUpdatedAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "merchantDepositInfo",
      "docs": [
        "商户保证金信息结构"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "totalDeposit",
            "type": "u64"
          },
          {
            "name": "lockedDeposit",
            "type": "u64"
          },
          {
            "name": "availableDeposit",
            "type": "u64"
          },
          {
            "name": "requiredDeposit",
            "type": "u64"
          },
          {
            "name": "isSufficient",
            "type": "bool"
          },
          {
            "name": "depositTokenMint",
            "type": "pubkey"
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "merchantIdAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "merchantId",
            "type": "u32"
          },
          {
            "name": "lastChunkIndex",
            "type": "u32"
          },
          {
            "name": "lastLocalId",
            "type": "u64"
          },
          {
            "name": "activeChunk",
            "type": "pubkey"
          },
          {
            "name": "unusedChunks",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "merchantRegisteredAtomic",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "merchant",
            "type": "pubkey"
          },
          {
            "name": "merchantId",
            "type": "u32"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "initialIdRangeStart",
            "type": "u64"
          },
          {
            "name": "initialIdRangeEnd",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "merchantStats",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "productCount",
            "type": "u64"
          },
          {
            "name": "totalSales",
            "type": "u64"
          },
          {
            "name": "activeProducts",
            "type": "u64"
          },
          {
            "name": "totalKeywords",
            "type": "u64"
          },
          {
            "name": "avgProductPrice",
            "type": "u64"
          },
          {
            "name": "lastUpdated",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "order",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "merchant",
            "type": "pubkey"
          },
          {
            "name": "productId",
            "type": "u64"
          },
          {
            "name": "quantity",
            "type": "u32"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "totalAmount",
            "type": "u64"
          },
          {
            "name": "paymentToken",
            "type": "pubkey"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "orderManagementStatus"
              }
            }
          },
          {
            "name": "shippingAddress",
            "type": "string"
          },
          {
            "name": "notes",
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "confirmedAt",
            "type": {
              "option": "i64"
            }
          },
          {
            "name": "shippedAt",
            "type": {
              "option": "i64"
            }
          },
          {
            "name": "deliveredAt",
            "type": {
              "option": "i64"
            }
          },
          {
            "name": "refundedAt",
            "type": {
              "option": "i64"
            }
          },
          {
            "name": "refundRequestedAt",
            "type": {
              "option": "i64"
            }
          },
          {
            "name": "refundReason",
            "type": "string"
          },
          {
            "name": "trackingNumber",
            "type": "string"
          },
          {
            "name": "transactionSignature",
            "type": "string"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "orderManagementStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "pending"
          },
          {
            "name": "shipped"
          },
          {
            "name": "delivered"
          },
          {
            "name": "refunded"
          }
        ]
      }
    },
    {
      "name": "orderStats",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "totalOrders",
            "type": "u64"
          },
          {
            "name": "pendingOrders",
            "type": "u64"
          },
          {
            "name": "shippedOrders",
            "type": "u64"
          },
          {
            "name": "deliveredOrders",
            "type": "u64"
          },
          {
            "name": "refundedOrders",
            "type": "u64"
          },
          {
            "name": "totalRevenue",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "paymentConfig",
      "docs": [
        "系统级支付配置账户"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "supportedTokens",
            "type": {
              "vec": {
                "defined": {
                  "name": "supportedToken"
                }
              }
            }
          },
          {
            "name": "feeRate",
            "type": "u16"
          },
          {
            "name": "feeRecipient",
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "priceIndexNode",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "priceRangeStart",
            "type": "u64"
          },
          {
            "name": "priceRangeEnd",
            "type": "u64"
          },
          {
            "name": "productIds",
            "type": {
              "vec": "u64"
            }
          },
          {
            "name": "leftChild",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "rightChild",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "parent",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "height",
            "type": "u8"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "productBase",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": "u64"
          },
          {
            "name": "merchant",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "description",
            "type": "string"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "keywords",
            "type": "string"
          },
          {
            "name": "inventory",
            "type": "u64"
          },
          {
            "name": "sales",
            "type": "u32"
          },
          {
            "name": "isActive",
            "type": "bool"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "paymentToken",
            "type": "pubkey"
          },
          {
            "name": "shippingLocation",
            "type": "string"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "productEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "productId",
            "type": "u64"
          },
          {
            "name": "merchant",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "description",
            "type": "string"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "keywords",
            "type": {
              "vec": "string"
            }
          },
          {
            "name": "salesCount",
            "type": "u32"
          },
          {
            "name": "isActive",
            "type": "bool"
          },
          {
            "name": "timestamp",
            "type": "i64"
          },
          {
            "name": "eventType",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "productExtended",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "productId",
            "type": "u64"
          },
          {
            "name": "imageVideoUrls",
            "type": "string"
          },
          {
            "name": "salesRegions",
            "type": "string"
          },
          {
            "name": "logisticsMethods",
            "type": "string"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "productSales",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "productId",
            "type": "u64"
          },
          {
            "name": "merchant",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "sales",
            "type": "u32"
          },
          {
            "name": "lastUpdate",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "salesIndexNode",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "salesRangeStart",
            "type": "u32"
          },
          {
            "name": "salesRangeEnd",
            "type": "u32"
          },
          {
            "name": "productIds",
            "type": {
              "vec": "u64"
            }
          },
          {
            "name": "topItems",
            "type": {
              "vec": {
                "defined": {
                  "name": "productSales"
                }
              }
            }
          },
          {
            "name": "leftChild",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "rightChild",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "parent",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "height",
            "type": "u8"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "supportedToken",
      "docs": [
        "支持的代币信息"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "isActive",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "systemConfig",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "maxProductsPerShard",
            "type": "u16"
          },
          {
            "name": "maxKeywordsPerProduct",
            "type": "u8"
          },
          {
            "name": "chunkSize",
            "type": "u32"
          },
          {
            "name": "bloomFilterSize",
            "type": "u16"
          },
          {
            "name": "merchantDepositRequired",
            "type": "u64"
          },
          {
            "name": "depositTokenMint",
            "type": "pubkey"
          },
          {
            "name": "platformFeeRate",
            "type": "u16"
          },
          {
            "name": "platformFeeRecipient",
            "type": "pubkey"
          },
          {
            "name": "autoConfirmDays",
            "type": "u32"
          },
          {
            "name": "externalProgramId",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "userPurchaseCount",
      "docs": [
        "用户购买计数账户"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "purchaseCount",
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    }
  ]
};
