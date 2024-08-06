use neutron_sdk::proto_types::slinky::oracle::v1::{
    GetPriceRequest, GetPriceResponse, GetPricesRequest, GetPricesResponse,
};
use test_tube_ntrn::fn_query;
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
    fn_query! {
        pub get_price ["/slinky.oracle.v1.Query/GetPrice"]: GetPriceRequest => GetPriceResponse
    }

    fn_query! {
        pub get_prices ["/slinky.oracle.v1.Query/GetPrices"]: GetPricesRequest => GetPricesResponse
    }
}

#[cfg(test)]
mod tests {
    use crate::{NeutronTestApp, Oracle};
    use cosmwasm_std::Coin;
    use neutron_sdk::proto_types::slinky::oracle::v1::GetPriceRequest;
    use neutron_sdk::proto_types::slinky::types::v1::CurrencyPair;
    use test_tube_ntrn::Module;

    #[test]
    #[allow(deprecated)]
    fn oracle_integration() {
        let app = NeutronTestApp::new();

        let _receiver = app
            .init_account(&[Coin::new(1_000_000_000_000u128, "untrn")])
            .unwrap();
        // let marketmap = Marketmap::new(&app);
        let oracle = Oracle::new(&app);

        // we can only set prices manually
        app.set_price_for_currency_pair(
            "USDT",
            "NTRN",
            150,
            app.get_block_time_seconds(),
            app.get_block_height(),
        );

        let price = oracle
            .get_price(&GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "USDT".to_string(),
                    quote: "NTRN".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(price.price.unwrap().price, "150");
    }
}
