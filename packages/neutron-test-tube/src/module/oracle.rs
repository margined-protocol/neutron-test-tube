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
    // use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    // use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as BaseCoin;
    // use neutron_sdk::proto_types::slinky::marketmap::v1::{Market, MarketMapRequest, MsgCreateMarkets, ProviderConfig, Ticker};
    use crate::{NeutronTestApp, Oracle};
    use cosmwasm_std::Coin;
    use neutron_sdk::proto_types::slinky::oracle::v1::GetPriceRequest;
    use neutron_sdk::proto_types::slinky::types::v1::CurrencyPair;
    use test_tube_ntrn::Module;

    #[test]
    #[allow(deprecated)]
    fn oracle_integration() {
        let app = NeutronTestApp::new();
        // let signer = app
        //     .init_account(&[
        //         Coin::new(1_000_000_000_000_000_000_000_000u128, "untrn"),
        //         Coin::new(1_000_000_000_000u128, "usdc"),
        //     ])
        //     .unwrap();
        // let signer2 = app
        //     .init_account(&[
        //         Coin::new(1_000_000_000_000_000_000_000_000u128, "untrn"),
        //         Coin::new(1_000_000_000_000u128, "usdc"),
        //     ])
        //     .unwrap();
        let _receiver = app
            .init_account(&[Coin::new(1_000_000_000_000u128, "untrn")])
            .unwrap();
        // let marketmap = Marketmap::new(&app);
        let oracle = Oracle::new(&app);
        // let bank = Bank::new(&app);

        // let scale_factor = 1_000_000_000_000_000_000u128;

        // bank.send(
        //     MsgSend {
        //         from_address: signer.address(),
        //         to_address: "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z".to_string(),
        //         amount: vec![BaseCoin {
        //             amount: 20_000u128.to_string(),
        //             denom: "untrn".to_string(),
        //         }],
        //     },
        //     &signer,
        // )
        // .unwrap();

        // does not work since cannot sign the message from authority (module account)
        // let new_markets: Vec<Market> = vec![
        //     Market{
        //         ticker: Some(Ticker{
        //             currency_pair: Some(CurrencyPair{ base: "USDT".to_string(), quote: "NTRN".to_string() }),
        //             decimals: 6,
        //             min_provider_count: 1,
        //             enabled: true,
        //             metadata_json: "".to_string(),
        //         }),
        //         provider_configs: vec![ProviderConfig{
        //             name: "api".to_string(),
        //             off_chain_ticker: "ticker".to_string(),
        //             normalize_by_pair: None,
        //             invert: false,
        //             metadata_json: "".to_string(),
        //         }],
        //     }
        // ];
        // let _res = marketmap.create_markets(MsgCreateMarkets{
        //     authority: "neutron1hxskfdxpp5hqgtjj6am6nkjefhfzj359x0ar3z".to_string(),
        //     create_markets: new_markets,
        // }, &signer2).unwrap();
        // let _res = marketmap.market_map(&MarketMapRequest{}).unwrap();

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
