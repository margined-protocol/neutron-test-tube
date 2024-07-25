# neutron-test-tube

CosmWasm x Neutron integration testing library that, unlike `cw-multi-test`, it allows you to test your cosmwasm contract against real chain's logic instead of mocks.

Please refer to [`CHANGELOG`](./CHANGELOG.md) for features and update information.

NOTE: If you need to test IBC specific functionality like IBC transactions / queries, this test framework does not support it.
Please refer to https://github.com/neutron-org/neutron-integration-tests on how to do that.

## Table of Contents

- [Getting Started](#getting-started)
- [Debugging](#debugging)
- [Using Module Wrapper](#using-module-wrapper)
- [Versioning](#versioning)

## Getting Started

To demonstrate how `neutron-test-tube` works, let use simple example contract: [cw-whitelist](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw1-whitelist) from `cw-plus`.

Here is how to setup the test:

```rust
use cosmwasm_std::Coin;
use neutron_test_tube::NeutronTestApp;

// create new neutron appchain instance.
let app = NeutronTestApp::new();

// create new account with initial funds
let accs = app
    .init_accounts(
        &[
            Coin::new(1_000_000_000_000, "usdt"),
            Coin::new(1_000_000_000_000, "untrn"),
        ],
        2,
    )
    .unwrap();

let admin = &accs[0];
let new_admin = &accs[1];
```

Now we have the appchain instance and accounts that have some initial balances and can interact with the appchain.
This does not run Docker instance or spawning external process, it just loads the appchain's code as a library to create an in memory instance.

Note that `init_accounts` is a convenience function that creates multiple accounts with the same initial balance.
If you want to create just one account, you can use `init_account` instead.

```rust
use cosmwasm_std::Coin;
use neutron_test_tube::NeutronTestApp;

let app = NeutronTestApp::new();

let account = app.init_account(&[
    Coin::new(1_000_000_000_000, "usdt"),
    Coin::new(1_000_000_000_000, "untrn"),
]);
```

Now if we want to test a cosmwasm contract, we need to

- build the wasm file
- store code
- instantiate

Then we can start interacting with our contract. Let's do just that.

```rust
use cosmwasm_std::Coin;
use cw1_whitelist::msg::{InstantiateMsg}; // for instantiating cw1_whitelist contract
use neutron_test_tube::{Account, Module, NeutronTestApp, Wasm};

let app = NeutronTestApp::new();
let accs = app
    .init_accounts(
        &[
            Coin::new(1_000_000_000_000, "usdt"),
            Coin::new(1_000_000_000_000, "untrn"),
        ],
        2,
    )
    .unwrap();
let admin = &accs[0];
let new_admin = &accs[1];

// ============= NEW CODE ================

// `Wasm` is the module we use to interact with cosmwasm related logic on the appchain
// it implements `Module` trait which you will see more later.
let wasm = Wasm::new(&app);

// Load compiled wasm bytecode
let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
let code_id = wasm
    .store_code(&wasm_byte_code, None, admin)
    .unwrap()
    .data
    .code_id;
```

Not that in this example, it loads wasm bytecode from [cw-plus release](https://github.com/CosmWasm/cw-plus/releases) for simple demonstration purposes.
You might want to run `cargo wasm` and find your wasm file in `target/wasm32-unknown-unknown/release/<contract_name>.wasm`.

```rust
use cosmwasm_std::Coin;
use cw1_whitelist::msg::{InstantiateMsg, QueryMsg, AdminListResponse};
use neutron_test_tube::{Account, Module, NeutronTestApp, Wasm};

let app = NeutronTestApp::new();
let accs = app
    .init_accounts(
        &[
            Coin::new(1_000_000_000_000, "usdt"),
            Coin::new(1_000_000_000_000, "untrn"),
        ],
        2,
    )
    .unwrap();
let admin = &accs[0];
let new_admin = &accs[1];

let wasm = Wasm::new(&app);


let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
let code_id = wasm
    .store_code(&wasm_byte_code, None, admin)
    .unwrap()
    .data
    .code_id;

// ============= NEW CODE ================

// instantiate contract with initial admin and make admin list mutable
let init_admins = vec![admin.address()];
let contract_addr = wasm
    .instantiate(
        code_id,
        &InstantiateMsg {
            admins: init_admins.clone(),
            mutable: true,
        },
        None, // contract admin used for migration, not the same as cw1_whitelist admin
        Some("Test label"), // contract label
        &[], // funds
        admin, // signer
    )
    .unwrap()
    .data
    .address;

// query contract state to check if contract instantiation works properly
let admin_list = wasm
    .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
    .unwrap();

assert_eq!(admin_list.admins, init_admins);
assert!(admin_list.mutable);
```

Now let's execute the contract and verify that the contract's state is updated properly.

```rust
use cosmwasm_std::Coin;
use cw1_whitelist::msg::{InstantiateMsg, QueryMsg, ExecuteMsg, AdminListResponse};
use neutron_test_tube::{Account, Module, NeutronTestApp, Wasm};

let app = NeutronTestApp::new();
let accs = app
    .init_accounts(
        &[
            Coin::new(1_000_000_000_000, "usdt"),
            Coin::new(1_000_000_000_000, "untrn"),
        ],
        2,
    )
    .unwrap();
let admin = &accs[0];
let new_admin = &accs[1];

let wasm = Wasm::new(&app);


let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
let code_id = wasm
    .store_code(&wasm_byte_code, None, admin)
    .unwrap()
    .data
    .code_id;

// instantiate contract with initial admin and make admin list mutable
let init_admins = vec![admin.address()];
let contract_addr = wasm
    .instantiate(
        code_id,
        &InstantiateMsg {
            admins: init_admins.clone(),
            mutable: true,
        },
        None, // contract admin used for migration, not the same as cw1_whitelist admin
        Some("Test label"), // contract label
        &[], // funds
        admin, // signer
    )
    .unwrap()
    .data
    .address;

let admin_list = wasm
    .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
    .unwrap();

assert_eq!(admin_list.admins, init_admins);
assert!(admin_list.mutable);

// ============= NEW CODE ================

// update admin list and rechec the state
let new_admins = vec![new_admin.address()];
wasm.execute::<ExecuteMsg>(
    &contract_addr,
    &ExecuteMsg::UpdateAdmins {
        admins: new_admins.clone(),
    },
    &[],
    admin,
)
.unwrap();

let admin_list = wasm
    .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
    .unwrap();

assert_eq!(admin_list.admins, new_admins);
assert!(admin_list.mutable);
```

## Debugging

In your contract code, if you want to debug, you can use [`deps.api.debug(..)`](https://docs.rs/cosmwasm-std/latest/cosmwasm_std/trait.Api.html#tymethod.debug) which will print the debug message to stdout. `wasmd` disabled this by default but `NeutronTestApp` allows stdout emission so that you can debug your smart contract while running tests.

## Using Module Wrapper

In some cases, you might want to interact directly with appchain logic to setup the environment or query appchain's state.
Module wrappers provides convenient functions to interact with the appchain's module.

Let's try to interact with `Exchange` module:

```rust
use cosmwasm_std::Coin;
use margined_neutron_std::shim::Any;
use margined_neutron_std::types::{
    cosmos::bank::v1beta1::{MsgSend, QueryBalanceRequest, SendAuthorization},
    cosmos::base::v1beta1::Coin as BaseCoin,
    neutron::dex as DexTypes,
};
use prost::Message;

use neutron_test_tube::{Account, Bank, Dex, NeutronTestApp};
use test_tube_ntrn::Module;

let app = NeutronTestApp::new();
let signer = app
    .init_account(&[
        Coin::new(1_000_000_000_000_000_000_000_000u128, "untrn"),
        Coin::new(1_000_000_000_000u128, "usdc"),
    ])
    .unwrap();
let receiver = app
    .init_account(&[Coin::new(1_000_000_000_000u128, "untrn")])
    .unwrap();
let dex = Dex::new(&app);
let bank = Bank::new(&app);

let scale_factor = 1_000_000_000_000_000_000u128;

let res = dex
    .place_limit_order(
        DexTypes::MsgPlaceLimitOrder {
            creator: signer.address().clone(),
            receiver: signer.address().clone(),
            token_in: "untrn".to_string(),
            token_out: "usdc".to_string(),
            tick_index_in_to_out: 0,
            amount_in: (10_000_000_000_000_000_00u128).to_string(),
            order_type: 0,
            expiration_time: None,
            max_amount_out: "".to_string(),
            limit_sell_price: (10u128 * scale_factor).to_string(),
        },
        &signer,
    )
    .unwrap();

let res = dex
    .tick_liquidity_all(&DexTypes::QueryAllTickLiquidityRequest {
        pair_id: "untrn<>usdc".to_string(),
        token_in: "untrn".to_string(),
        pagination: None,
    })
    .unwrap();

app.increase_time(1u64);
```

Additional examples can be found in the [modules](./src/module/) directory.

## Versioning

The version of `neutron-test-tube` follows that of Neutron mainnet releases.
