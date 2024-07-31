use neutron_sdk::proto_types::slinky::marketmap::v1::{LastUpdatedRequest, LastUpdatedResponse, MarketMapRequest, MarketMapResponse, MarketRequest, MarketResponse, MsgCreateMarkets, MsgCreateMarketsResponse, MsgUpdateMarkets, MsgUpdateMarketsResponse, ParamsRequest, ParamsResponse, MsgRemoveMarketAuthorities, MsgRemoveMarketAuthoritiesResponse};
use test_tube_ntrn::{fn_execute, fn_query};
use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Marketmap<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Marketmap<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Marketmap<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub create_markets: MsgCreateMarkets["/slinky.marketmap.v1.MsgCreateMarkets"] => MsgCreateMarketsResponse
    }

    fn_execute! {
        pub update_markets: MsgUpdateMarkets["/slinky.marketmap.v1.MsgUpdateMarkets"] => MsgUpdateMarketsResponse
    }

    fn_execute! {
        pub remove_market_authorities: MsgRemoveMarketAuthorities["/slinky.marketmap.v1.MsgRemoveMarketAuthorities"] => MsgRemoveMarketAuthoritiesResponse
    }

    fn_query! {
        pub market_map ["/slinky.marketmap.v1.Query/MarketMap"]: MarketMapRequest => MarketMapResponse
    }

    fn_query! {
        pub market ["/slinky.marketmap.v1.Query/Market"]: MarketRequest => MarketResponse
    }

    fn_query! {
        pub last_updated ["/slinky.marketmap.v1.Query/LastUpdated"]: LastUpdatedRequest => LastUpdatedResponse
    }

    fn_query! {
        pub params ["/slinky.marketmap.v1.Query/Params"]: ParamsRequest => ParamsResponse
    }
}

// TODO: tests