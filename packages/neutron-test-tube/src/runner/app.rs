use cosmwasm_std::Coin;

use prost::Message;
use test_tube_ntrn::account::SigningAccount;

use test_tube_ntrn::runner::result::{RunnerExecuteResult, RunnerResult};
use test_tube_ntrn::runner::Runner;
use test_tube_ntrn::BaseApp;

const FEE_DENOM: &str = "untrn";
const NEUTRON_ADDRESS_PREFIX: &str = "neutron";
const CHAIN_ID: &str = "neutron-666";
const DEFAULT_GAS_ADJUSTMENT: f64 = 1.2;

#[derive(Debug, PartialEq)]
pub struct NeutronTestApp {
    inner: BaseApp,
}

impl Default for NeutronTestApp {
    fn default() -> Self {
        NeutronTestApp::new()
    }
}

impl NeutronTestApp {
    pub fn new() -> Self {
        Self {
            inner: BaseApp::new(
                FEE_DENOM,
                CHAIN_ID,
                NEUTRON_ADDRESS_PREFIX,
                DEFAULT_GAS_ADJUSTMENT,
            ),
        }
    }

    /// Get the current block time in nanoseconds
    pub fn get_block_time_nanos(&self) -> i64 {
        self.inner.get_block_time_nanos()
    }

    /// Get the current block time in seconds
    pub fn get_block_time_seconds(&self) -> i64 {
        self.inner.get_block_time_nanos() / 1_000_000_000i64
    }

    /// Get the current block height
    pub fn get_block_height(&self) -> i64 {
        self.inner.get_block_height()
    }

    /// Get the first validator address
    pub fn get_first_validator_address(&self) -> RunnerResult<String> {
        self.inner.get_first_validator_address()
    }

    /// Get the first validator private key
    pub fn get_first_validator_private_key(&self) -> RunnerResult<String> {
        self.inner.get_first_validator_private_key()
    }

    /// Get the first validator signing account
    pub fn get_first_validator_signing_account(
        &self,
        denom: String,
        gas_adjustment: f64,
    ) -> RunnerResult<SigningAccount> {
        self.inner
            .get_first_validator_signing_account(denom, gas_adjustment)
    }

    /// Increase the time of the blockchain by the given number of seconds.
    pub fn increase_time(&self, seconds: u64) {
        self.inner.increase_time(seconds)
    }

    /// Initialize account with initial balance of any coins.
    /// This function mints new coins and send to newly created account
    pub fn init_account(&self, coins: &[Coin]) -> RunnerResult<SigningAccount> {
        self.inner.init_account(coins)
    }
    /// Convinience function to create multiple accounts with the same
    /// Initial coins balance
    pub fn init_accounts(&self, coins: &[Coin], count: u64) -> RunnerResult<Vec<SigningAccount>> {
        self.inner.init_accounts(coins, count)
    }

    /// Simulate transaction execution and return gas info
    pub fn simulate_tx<I>(
        &self,
        msgs: I,
        signer: &SigningAccount,
    ) -> RunnerResult<cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo>
    where
        I: IntoIterator<Item = cosmrs::Any>,
    {
        self.inner.simulate_tx(msgs, signer)
    }

    // /// Set parameter set for a given subspace.
    // pub fn set_param_set(&self, subspace: &str, pset: impl Into<Any>) -> RunnerResult<()> {
    //     self.inner.set_param_set(subspace, pset)
    // }

    /// Get parameter set for a given subspace.
    pub fn get_param_set<P: Message + Default>(
        &self,
        subspace: &str,
        type_url: &str,
    ) -> RunnerResult<P> {
        self.inner.get_param_set(subspace, type_url)
    }
}

impl<'a> Runner<'a> for NeutronTestApp {
    fn execute_multiple<M, R>(
        &self,
        msgs: &[(M, &str)],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        M: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.execute_multiple(msgs, signer)
    }

    fn query<Q, R>(&self, path: &str, q: &Q) -> RunnerResult<R>
    where
        Q: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.query(path, q)
    }

    fn execute_multiple_raw<R>(
        &self,
        msgs: Vec<cosmrs::Any>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        R: prost::Message + Default,
    {
        self.inner.execute_multiple_raw(msgs, signer)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Coin, Uint128};
    use neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::{
        MsgCreateDenom, MsgCreateDenomResponse, QueryParamsRequest, QueryParamsResponse,
    };

    use crate::module::Wasm;
    use crate::runner::app::NeutronTestApp;

    use test_tube_ntrn::account::Account;
    use test_tube_ntrn::module::Module;
    use test_tube_ntrn::runner::*;
    use test_tube_ntrn::ExecuteResponse;

    #[test]
    fn test_init_account() {
        let app = NeutronTestApp::default();

        // Just check it doesn't panic
        app.init_account(&coins(100_000_000_000, "untrn")).unwrap();
    }

    #[test]
    fn test_init_accounts() {
        let app = NeutronTestApp::default();

        let accounts = app
            .init_accounts(&coins(100_000_000_000, "untrn"), 3)
            .unwrap();

        assert!(accounts.get(0).is_some());
        assert!(accounts.get(1).is_some());
        assert!(accounts.get(2).is_some());
        assert!(accounts.get(3).is_none());
    }

    #[test]
    fn test_get_and_set_block_timestamp() {
        let app = NeutronTestApp::default();

        let block_time_nanos = app.get_block_time_nanos();
        let block_time_seconds = app.get_block_time_seconds();

        app.increase_time(10u64);

        assert_eq!(
            app.get_block_time_nanos(),
            block_time_nanos + 10_000_000_000
        );
        assert_eq!(app.get_block_time_seconds(), block_time_seconds + 10);
    }

    #[test]
    fn test_get_block_height() {
        let app = NeutronTestApp::default();

        assert_eq!(app.get_block_height(), 1i64);

        app.increase_time(10u64);

        assert_eq!(app.get_block_height(), 2i64);
    }

    #[test]
    fn test_execute() {
        let app = NeutronTestApp::default();

        assert_eq!(app.get_block_height(), 1i64);

        let acc = app
            .init_account(&coins(100_000_000_000_000_000_000u128, "untrn")) // 100 inj
            .unwrap();
        let addr = acc.address();

        let msg = MsgCreateDenom {
            sender: acc.address(),
            subdenom: "newdenom".to_string(),
        };

        let res: ExecuteResponse<MsgCreateDenomResponse> = app
            .execute(msg, "/osmosis.tokenfactory.v1beta1.MsgCreateDenom", &acc)
            .unwrap();

        let create_denom_attrs = &res.data.new_token_denom;
        assert_eq!(
            create_denom_attrs,
            &format!("factory/{}/{}", &addr, "newdenom")
        );

        // execute on more time to excercise account sequence
        let msg = MsgCreateDenom {
            sender: acc.address(),
            subdenom: "newerdenom".to_string(),
        };

        let res: ExecuteResponse<MsgCreateDenomResponse> = app
            .execute(msg, "/osmosis.tokenfactory.v1beta1.MsgCreateDenom", &acc)
            .unwrap();

        let create_denom_attrs = &res.data.new_token_denom;
        assert_eq!(
            create_denom_attrs,
            &format!("factory/{}/{}", &addr, "newerdenom")
        );

        // execute on more time to excercise account sequence
        let msg = MsgCreateDenom {
            sender: acc.address(),
            subdenom: "multidenom_1".to_string(),
        };

        let msg_2 = MsgCreateDenom {
            sender: acc.address(),
            subdenom: "multidenom_2".to_string(),
        };

        assert_eq!(app.get_block_height(), 4i64);

        let _res: ExecuteResponse<MsgCreateDenomResponse> = app
            .execute_multiple(
                &[
                    (msg, "/osmosis.tokenfactory.v1beta1.MsgCreateDenom"),
                    (msg_2, "/osmosis.tokenfactory.v1beta1.MsgCreateDenom"),
                ],
                &acc,
            )
            .unwrap();

        assert_eq!(app.get_block_height(), 5i64);
    }

    #[test]
    fn test_query() {
        let app = NeutronTestApp::default();

        let denom_creation_fee = app
            .query::<QueryParamsRequest, QueryParamsResponse>(
                "/osmosis.tokenfactory.v1beta1.Query/Params",
                &QueryParamsRequest {},
            )
            .unwrap()
            .params
            .unwrap()
            .denom_creation_fee;

        // fee is no longer set
        assert_eq!(denom_creation_fee, [])
    }

    #[test]
    fn test_wasm_execute_and_query() {
        use cw1_whitelist::msg::*;

        let app = NeutronTestApp::default();
        let accs = app
            .init_accounts(
                &[
                    Coin::new(Uint128::new(1_000_000_000_000), "uatom"),
                    Coin::new(Uint128::new(1_000_000_000_000), "untrn"),
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
        assert_eq!(code_id, 1);

        // initialize admins and check if the state is correct
        let init_admins = vec![admin.address()];
        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    admins: init_admins.clone(),
                    mutable: true,
                },
                Some(&admin.address()),
                Some("Test label"),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;
        let admin_list = wasm
            .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
            .unwrap();
        assert_eq!(admin_list.admins, init_admins);
        assert!(admin_list.mutable);

        // update admin and check again
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
    }
}
