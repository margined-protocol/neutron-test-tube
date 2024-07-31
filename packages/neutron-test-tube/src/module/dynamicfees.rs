// TODO: no types in neutron-sdk

// use neutron_sdk::proto_types::neutron::::{QueryParamsRequest, QueryParamsResponse, QueryFailuresRequest, QueryFailuresResponse, MsgUpdateParams, MsgUpdateParamsResponse};
// use test_tube_ntrn::{fn_execute, fn_query};
//
// use test_tube_ntrn::module::Module;
// use test_tube_ntrn::runner::Runner;
//
// pub struct Dynamicfees<'a, R: Runner<'a>> {
//     runner: &'a R,
// }
//
// impl<'a, R: Runner<'a>> Module<'a, R> for Dynamicfees<'a, R> {
//     fn new(runner: &'a R) -> Self {
//         Self { runner }
//     }
// }
//
// impl<'a, R> Dynamicfees<'a, R>
// where
//     R: Runner<'a>,
// {
//     fn_execute! {
//         pub update_params: MsgUpdateParams["/neutron.dynamicfees.v1.Msg/UpdateParams"] => MsgUpdateParamsResponse
//     }
//
//     fn_query! {
//         pub query_params ["/neutron.dynamicfees.v1.Query/Params"]: QueryParamsRequest => QueryParamsResponse
//     }
// }
//
// // TODO: tests