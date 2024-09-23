use margined_neutron_std::types::slinky::{
    marketmap::v1 as MarketMapTypesV1, oracle::v1 as OracleTypesV1,
};
use test_tube_ntrn::{fn_execute, fn_query};

use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Slinky<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Slinky<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Slinky<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub create_markets: MarketMapTypesV1::MsgCreateMarkets["/slinky.marketmap.v1.MsgCreateMarkets"] => MarketMapTypesV1::MsgCreateMarketsResponse
    }

    fn_query! {
        pub get_all_currency_pairs ["/slinky.oracle.v1.Query/GetAllCurrencyPairs"]: OracleTypesV1::GetAllCurrencyPairsRequest => OracleTypesV1::GetAllCurrencyPairsResponse
    }

    fn_query! {
        pub get_price ["/slinky.oracle.v1.Query/GetPrice"]: OracleTypesV1::GetPriceRequest => OracleTypesV1::GetPriceResponse
    }

    fn_query! {
        pub get_prices ["/slinky.oracle.v1.Query/GetPrices"]: OracleTypesV1::GetPricesRequest => OracleTypesV1::GetPricesResponse
    }

    fn_query! {
        pub get_currency_pair_mapping ["/slinky.oracle.v1.Query/GetCurrencyPairMapping"]: OracleTypesV1::GetCurrencyPairMappingRequest => OracleTypesV1::GetCurrencyPairMappingResponse
    }

    fn_query! {
        pub get_params ["/slinky.marketmap.v1.Query/Params"]: MarketMapTypesV1::ParamsRequest => MarketMapTypesV1::ParamsResponse
    }
}

#[cfg(test)]
mod tests {
    use margined_neutron_std::{
        shim::Timestamp,
        types::slinky::{
            marketmap::v1::{Market, MsgCreateMarkets, ProviderConfig, Ticker},
            oracle::v1::{self as OracleTypes, GetPriceResponse, QuotePrice},
            types::v1::CurrencyPair,
        },
    };

    use crate::{Account, NeutronTestApp, Slinky};
    use test_tube_ntrn::{runner::app::SlinkyPrices, Module};

    #[test]
    fn slinky_integration() {
        let app = NeutronTestApp::new();

        let slinky = Slinky::new(&app);

        let val = app
            .get_first_validator_signing_account("untrn".to_string(), 1.3)
            .unwrap();

        let res = slinky
            .get_all_currency_pairs(&OracleTypes::GetAllCurrencyPairsRequest {})
            .unwrap();
        assert_eq!(
            res.currency_pairs,
            vec![CurrencyPair {
                base: "ATOM".to_string(),
                quote: "USDT".to_string(),
            }]
        );

        slinky
            .create_markets(
                MsgCreateMarkets {
                    authority: val.address(),
                    create_markets: vec![Market {
                        ticker: Some(Ticker {
                            currency_pair: Some(CurrencyPair {
                                base: "NTRN".to_string(),
                                quote: "USDC".to_string(),
                            }),
                            decimals: 6,
                            min_provider_count: 1,
                            enabled: true,
                            metadata_json: "".to_string(),
                        }),
                        provider_configs: vec![ProviderConfig {
                            name: "margined".to_string(),
                            off_chain_ticker: "NRTN/USD".to_string(),
                            normalize_by_pair: None,
                            invert: false,
                            metadata_json: "".to_string(),
                        }],
                    }],
                },
                &val,
            )
            .unwrap();

        let res = slinky
            .get_all_currency_pairs(&OracleTypes::GetAllCurrencyPairsRequest {})
            .unwrap();
        assert_eq!(
            res.currency_pairs,
            vec![
                CurrencyPair {
                    base: "ATOM".to_string(),
                    quote: "USDT".to_string(),
                },
                CurrencyPair {
                    base: "NTRN".to_string(),
                    quote: "USDC".to_string(),
                }
            ]
        );

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "ATOM".to_string(),
                    quote: "USDT".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(
            res,
            GetPriceResponse {
                price: Some(QuotePrice {
                    price: "4480000".to_string(),
                    block_timestamp: Some(Timestamp {
                        // this is set in genesis so no clue why it is negative
                        // but lets not worry too much
                        seconds: -62135596800,
                        nanos: 0
                    }),
                    block_height: 0
                }),
                decimals: 8,
                nonce: 0,
                id: 0,
            }
        );

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "NTRN".to_string(),
                    quote: "USDC".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(
            res,
            GetPriceResponse {
                price: Some(QuotePrice {
                    price: "0".to_string(),
                    block_timestamp: Some(Timestamp {
                        // this is set in genesis so no clue why it is negative
                        // but lets not worry too much
                        seconds: -62135596800,
                        nanos: 0
                    }),
                    block_height: 0
                }),
                decimals: 6,
                nonce: 0,
                id: 1,
            }
        );

        app.set_slinky_prices(&[SlinkyPrices {
            base: "ATOM".to_string(),
            quote: "USDT".to_string(),
            price: 513000000u128,
        }]);

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "ATOM".to_string(),
                    quote: "USDT".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(res.price.unwrap().price, "513000000".to_string());

        // Set NTRN/USDC
        app.set_slinky_prices(&[SlinkyPrices {
            base: "NTRN".to_string(),
            quote: "USDC".to_string(),
            price: 4230000u128,
        }]);

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "NTRN".to_string(),
                    quote: "USDC".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(res.price.unwrap().price, "4230000".to_string());

        // Reduce price
        app.set_slinky_prices(&[SlinkyPrices {
            base: "NTRN".to_string(),
            quote: "USDC".to_string(),
            price: 4130000u128,
        }]);

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "NTRN".to_string(),
                    quote: "USDC".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(res.price.unwrap().price, "4130000".to_string());

        // Reduce price
        app.set_slinky_prices(&[SlinkyPrices {
            base: "NTRN".to_string(),
            quote: "USDC".to_string(),
            price: 5012345u128,
        }]);

        let res = slinky
            .get_price(&OracleTypes::GetPriceRequest {
                currency_pair: Some(CurrencyPair {
                    base: "NTRN".to_string(),
                    quote: "USDC".to_string(),
                }),
            })
            .unwrap();
        assert_eq!(res.price.unwrap().price, "5012345".to_string());
    }
}
