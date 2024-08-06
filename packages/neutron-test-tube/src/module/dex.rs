use margined_neutron_std::types::neutron::dex as DexTypes;
use test_tube_ntrn::{fn_execute, fn_query};

use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Dex<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Dex<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Dex<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub cancel_filled_limit_order: DexTypes::MsgCancelLimitOrder["/neutron.dex.MsgCancelLimitOrder"] => DexTypes::MsgCancelLimitOrderResponse
    }

    fn_execute! {
        pub deposit: DexTypes::MsgDeposit["/neutron.dex.MsgDeposit"] => DexTypes::MsgDepositResponse
    }

    fn_execute! {
        pub multi_hop_swap: DexTypes::MsgMultiHopSwap["/neutron.dex.MsgDeposit"] => DexTypes::MsgMultiHopSwapResponse
    }

    fn_execute! {
        pub place_limit_order: DexTypes::MsgPlaceLimitOrder["/neutron.dex.MsgPlaceLimitOrder"] => DexTypes::MsgPlaceLimitOrderResponse
    }

    fn_execute! {
        pub withdrawal: DexTypes::MsgWithdrawal["/neutron.dex.MsgWithdrawal"] => DexTypes::MsgWithdrawalResponse
    }

    fn_execute! {
        pub withdraw_filled_limit_order: DexTypes::MsgWithdrawFilledLimitOrder["/neutron.dex.MsgWithdrawFilledLimitOrder"] => DexTypes::MsgWithdrawFilledLimitOrderResponse
    }

    fn_query! {
        pub params ["/neutron.dex.Query/Params"]: DexTypes::QueryParamsRequest => DexTypes::QueryParamsResponse
    }

    fn_query! {
        pub limit_order_tranche_user ["/neutron.dex.Query/LimitOrderTrancheUser"]: DexTypes::QueryGetLimitOrderTrancheUserRequest => DexTypes::QueryGetLimitOrderTrancheUserResponse
    }

    fn_query! {
        pub limit_order_tranche_user_all ["/neutron.dex.Query/LimitOrderTrancheUserAll"]: DexTypes::QueryAllLimitOrderTrancheUserRequest => DexTypes::QueryAllLimitOrderTrancheUserResponse
    }

    fn_query! {
        pub limit_order_tranche_user_all_by_address ["/neutron.dex.Query/LimitOrderTrancheUserAllByAddress"]: DexTypes::QueryAllLimitOrderTrancheUserByAddressRequest => DexTypes::QueryAllLimitOrderTrancheUserByAddressResponse
    }

    fn_query! {
        pub limit_order_tranche ["/neutron.dex.Query/LimitOrderTranche"]: DexTypes::QueryGetLimitOrderTrancheRequest => DexTypes::QueryGetLimitOrderTrancheResponse
    }

    fn_query! {
        pub limit_order_tranche_all ["/neutron.dex.Query/LimitOrderTrancheAll"]: DexTypes::QueryAllLimitOrderTrancheRequest => DexTypes::QueryAllLimitOrderTrancheResponse
    }

    fn_query! {
        pub user_deposits_all ["/neutron.dex.Query/UserDepositsAll"]: DexTypes::QueryAllUserDepositsRequest => DexTypes::QueryAllUserDepositsResponse
    }

    fn_query! {
        pub tick_liquidity_all ["/neutron.dex.Query/TickLiquidityAll"]: DexTypes::QueryAllTickLiquidityRequest => DexTypes::QueryAllTickLiquidityResponse
    }

    fn_query! {
        pub inactive_limit_order_tranche ["/neutron.dex.Query/GetInactiveLimitOrder"]: DexTypes::QueryGetInactiveLimitOrderTrancheRequest => DexTypes::QueryGetInactiveLimitOrderTrancheResponse
    }

    fn_query! {
        pub inactive_limit_order_tranche_all ["/neutron.dex.Query/AllInactiveLimitOrderTranche"]: DexTypes::QueryAllInactiveLimitOrderTrancheRequest => DexTypes::QueryAllInactiveLimitOrderTrancheResponse
    }

    fn_query! {
        pub pool_reserves_all ["/neutron.dex.Query/AllPoolReserves"]: DexTypes::QueryAllPoolReservesRequest => DexTypes::QueryAllPoolReservesResponse
    }

    fn_query! {
        pub pool_reserves ["/neutron.dex.Query/GetPoolReserves"]: DexTypes::QueryGetPoolReservesRequest => DexTypes::QueryGetPoolReservesResponse
    }

    fn_query! {
        pub estimate_multi_hop_swap ["/neutron.dex.Query/EstimateMultiHopSwap"]: DexTypes::QueryEstimateMultiHopSwapRequest => DexTypes::QueryEstimateMultiHopSwapResponse
    }

    fn_query! {
        pub estimate_place_limit_order ["/neutron.dex.Query/EstimatePlaceLimitOrder"]: DexTypes::QueryEstimatePlaceLimitOrderRequest => DexTypes::QueryEstimatePlaceLimitOrderResponse
    }

    fn_query! {
        pub pool ["/neutron.dex.Query/Pool"]: DexTypes::QueryPoolRequest => DexTypes::QueryPoolResponse
    }

    fn_query! {
        pub pool_by_id ["/neutron.dex.Query/PoolById"]: DexTypes::QueryPoolByIdRequest => DexTypes::QueryPoolResponse
    }

    fn_query! {
        pub pool_metadata ["/neutron.dex.Query/GetPoolMetadata"]: DexTypes::QueryGetPoolMetadataRequest => DexTypes::QueryGetPoolMetadataResponse
    }

    fn_query! {
        pub pool_metadata_all ["/neutron.dex.Query/AllPoolMetadata"]: DexTypes::QueryAllPoolMetadataRequest => DexTypes::QueryAllPoolMetadataResponse
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Coin;
    use margined_neutron_std::types::neutron::dex as DexTypes;

    use crate::{Account, Dex, NeutronTestApp};
    use test_tube_ntrn::Module;

    #[test]
    #[allow(deprecated)]
    fn dex_integration() {
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
        let dex = Dex::new(&app);

        let scale_factor = 1_000_000_000_000_000_000u128;

        let _res = dex
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

        let _res = dex
            .tick_liquidity_all(&DexTypes::QueryAllTickLiquidityRequest {
                pair_id: "untrn<>usdc".to_string(),
                token_in: "untrn".to_string(),
                pagination: None,
            })
            .unwrap();
    }
}
