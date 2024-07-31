use neutron_sdk::proto_types::slinky::oracle::v1::{GetAllCurrencyPairsRequest, GetAllCurrencyPairsResponse, GetPriceRequest, GetPriceResponse, GetPricesRequest, GetPricesResponse, MsgAddCurrencyPairs, MsgAddCurrencyPairsResponse, MsgRemoveCurrencyPairs, MsgRemoveCurrencyPairsResponse};
use test_tube_ntrn::{fn_execute, fn_query};
use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Oracle<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Oracle<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Oracle<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub add_currency_pairs: MsgAddCurrencyPairs["/slinky.oracle.v1.MsgAddCurrencyPairs"] => MsgAddCurrencyPairsResponse
    }

    fn_execute! {
        pub remove_currency_pairs: MsgRemoveCurrencyPairs["/slinky.oracle.v1.MsgRemoveCurrencyPairs"] => MsgRemoveCurrencyPairsResponse
    }

    fn_query! {
        pub get_all_currency_pairs ["/slinky.oracle.v1.Query/GetAllCurrencyPairs"]: GetAllCurrencyPairsRequest => GetAllCurrencyPairsResponse
    }

    fn_query! {
        pub get_price ["/slinky.oracle.v1.Query/GetPrice"]: GetPriceRequest => GetPriceResponse
    }

    fn_query! {
        pub get_prices ["/slinky.oracle.v1.Query/GetPrices"]: GetPricesRequest => GetPricesResponse
    }
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::Coin;
    use crate::{NeutronTestApp, Oracle};
    use neutron_sdk::proto_types::slinky::oracle::v1 as OracleTypes;
    use neutron_sdk::proto_types::slinky::oracle::v1::GetAllCurrencyPairsRequest;
    use neutron_sdk::proto_types::slinky::types::v1::CurrencyPair;
    use test_tube_ntrn::Module;

    #[test]
    #[allow(deprecated)]
    fn oracle_integration() {
        let app = NeutronTestApp::new();
        let signer = app
            .init_account(&[
                Coin::new(1_000_000_000_000_000_000_000_000u128, "untrn"),
                Coin::new(1_000_000_000_000u128, "usdc"),
            ])
            .unwrap();
        let _receiver = app
            .init_account(&[Coin::new(1_000_000_000_000u128, "untrn")])
            .unwrap();
        let oracle = Oracle::new(&app);

        let scale_factor = 1_000_000_000_000_000_000u128;

        let new_currency_pairs = vec![
            CurrencyPair{
                base: "USD".to_string(),
                quote: "USDT".to_string(),
            }
        ];
        let _res = oracle.add_currency_pairs(
            OracleTypes::MsgAddCurrencyPairs{
                authority: "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z".to_string(),
                currency_pairs: new_currency_pairs.clone(),
            },
            &signer,
        ).unwrap();

        let currency_pairs = oracle.get_all_currency_pairs(&GetAllCurrencyPairsRequest{}).unwrap();
        assert_eq!(currency_pairs.currency_pairs, new_currency_pairs)
    }
}
