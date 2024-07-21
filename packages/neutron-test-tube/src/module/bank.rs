use neutron_std::types::cosmos::bank::v1beta1::{
    MsgSend, MsgSendResponse, QueryAllBalancesRequest, QueryAllBalancesResponse,
    QueryBalanceRequest, QueryBalanceResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use test_tube_ntrn::{fn_execute, fn_query};

use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Bank<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Bank<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Bank<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub send: MsgSend["/cosmos.bank.v1beta1.MsgSend"] => MsgSendResponse
    }

    fn_query! {
        pub query_balance ["/cosmos.bank.v1beta1.Query/Balance"]: QueryBalanceRequest => QueryBalanceResponse
    }

    fn_query! {
        pub query_all_balances ["/cosmos.bank.v1beta1.Query/AllBalances"]: QueryAllBalancesRequest => QueryAllBalancesResponse
    }

    fn_query! {
        pub query_total_supply ["/cosmos.bank.v1beta1.Query/TotalSupply"]: QueryTotalSupplyRequest => QueryTotalSupplyResponse
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Coin;
    use neutron_std::types::cosmos::bank::v1beta1::{MsgSend, QueryBalanceRequest};
    use neutron_std::types::cosmos::base::v1beta1::Coin as BaseCoin;

    use crate::{Account, Bank, NeutronTestApp};
    use test_tube_ntrn::Module;

    #[test]
    fn bank_integration() {
        let app = NeutronTestApp::new();
        let signer = app
            .init_account(&[Coin::new(100_000_000_000u128, "untrn")])
            .unwrap();
        let receiver = app.init_account(&[Coin::new(1u128, "untrn")]).unwrap();
        let bank = Bank::new(&app);

        let response = bank
            .query_balance(&QueryBalanceRequest {
                address: receiver.address(),
                denom: "untrn".to_string(),
            })
            .unwrap();
        assert_eq!(
            response.balance.unwrap(),
            BaseCoin {
                amount: 1u128.to_string(),
                denom: "untrn".to_string(),
            }
        );

        bank.send(
            MsgSend {
                from_address: signer.address(),
                to_address: receiver.address(),
                amount: vec![BaseCoin {
                    amount: 9u128.to_string(),
                    denom: "untrn".to_string(),
                }],
            },
            &signer,
        )
        .unwrap();
    }
}
