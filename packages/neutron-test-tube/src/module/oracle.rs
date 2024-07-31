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
        pub add_currency_pairs: MsgAddCurrencyPairs["/slinky.oracle.v1.Msg/AddCurrencyPairs"] => MsgAddCurrencyPairsResponse
    }

    fn_execute! {
        pub remove_currency_pairs: MsgRemoveCurrencyPairs["/slinky.oracle.v1.Msg/RemoveCurrencyPairs"] => MsgRemoveCurrencyPairsResponse
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

// TODO: tests