pub mod app;

#[cfg(test)]
mod tests {

    use crate::{Bank, Wasm};

    use super::app::NeutronTestApp;
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use cosmwasm_std::{to_json_binary, BankMsg, Coin, CosmosMsg, Empty, Event, WasmMsg, Binary};
    use cw1_whitelist::msg::{ExecuteMsg, InstantiateMsg};
    use neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::{
        MsgCreateDenom, MsgCreateDenomResponse,
    };
    use cosmos_sdk_proto::{
        cosmos::bank::v1beta1::{MsgSendResponse, QueryBalanceRequest},
        cosmwasm::wasm::v1::{MsgExecuteContractResponse, MsgInstantiateContractResponse},
    };
    use std::ffi::CString;
    use prost::Message;
    use test_tube_ntrn::account::Account;
    use test_tube_ntrn::runner::error::RunnerError::QueryError;
    use test_tube_ntrn::runner::result::RawResult;
    use test_tube_ntrn::runner::Runner;
    use test_tube_ntrn::Module;

    #[derive(::prost::Message)]
    struct AdhocRandomQueryRequest {
        #[prost(uint64, tag = "1")]
        id: u64,
    }

    #[derive(::prost::Message)]
    struct AdhocRandomQueryResponse {
        #[prost(string, tag = "1")]
        msg: String,
    }

    #[test]
    fn test_query_error_no_route() {
        let app = NeutronTestApp::default();
        let res = app.query::<AdhocRandomQueryRequest, AdhocRandomQueryResponse>(
            "/neutron.random.v1beta1.Query/AdhocRandom",
            &AdhocRandomQueryRequest { id: 1 },
        );

        let err = res.unwrap_err();
        assert_eq!(
            err,
            QueryError {
                msg: "No route found for `/neutron.random.v1beta1.Query/AdhocRandom`".to_string()
            }
        );
    }

    #[test]
    fn test_raw_result_ptr_with_0_bytes_in_content_should_not_error() {
        let base64_string = BASE64_STANDARD.encode(vec![vec![0u8], vec![0u8]].concat());
        let res = unsafe {
            RawResult::from_ptr(
                CString::new(base64_string.as_bytes().to_vec())
                    .unwrap()
                    .into_raw(),
            )
        }
        .unwrap()
        .into_result()
        .unwrap();

        assert_eq!(res, vec![0u8]);
    }

    #[test]
    fn test_execute_cosmos_msgs() {
        let app = NeutronTestApp::new();
        let signer = app
            .init_account(&[Coin::new(10000000000u128, "untrn")])
            .unwrap();

        let bank = Bank::new(&app);

        // BankMsg::Send
        let to = app.init_account(&[]).unwrap();
        let coin = Coin::new(100u128, "untrn");
        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: to.address(),
            amount: vec![coin],
        });
        app.execute_cosmos_msgs::<MsgSendResponse>(&[send_msg], &signer)
            .unwrap();
        let balance = bank
            .query_balance(&QueryBalanceRequest {
                address: to.address(),
                denom: "untrn".to_string(),
            })
            .unwrap()
            .balance;
        assert_eq!(balance.clone().unwrap().amount, "100".to_string());
        assert_eq!(balance.unwrap().denom, "untrn".to_string());

        // WasmMsg, first upload a contract
        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &signer)
            .unwrap()
            .data
            .code_id;
        assert_eq!(code_id, 1);

        // Wasm::Instantiate
        let instantiate_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id,
            msg: to_json_binary(&InstantiateMsg {
                admins: vec![signer.address()],
                mutable: true,
            })
            .unwrap(),
            funds: vec![],
            label: "test".to_string(),
            admin: None,
        });
        let init_res = app
            .execute_cosmos_msgs::<MsgInstantiateContractResponse>(&[instantiate_msg], &signer)
            .unwrap();
        let contract_address = init_res.data.address;
        assert_ne!(contract_address, "".to_string());

        // Wasm::Execute
        let execute_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_address.clone(),
            msg: to_json_binary(&ExecuteMsg::<Empty>::Freeze {}).unwrap(),
            funds: vec![],
        });
        let execute_res = app
            .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[execute_msg], &signer)
            .unwrap();
        let events = execute_res.events;

        let wasm_events: Vec<Event> = events.into_iter().filter(|x| x.ty == "wasm").collect();
        for event in wasm_events.iter() {
            assert_eq!(event.attributes[0].key, "_contract_address");
            assert_eq!(event.attributes[0].value, contract_address);
            assert_eq!(event.attributes[1].key, "action");
            assert_eq!(event.attributes[1].value, "freeze");
        }

        // Stargate
        let denom = "test".to_string();
        #[allow(deprecated)]
        let create_denom_msg: CosmosMsg = CosmosMsg::Stargate {
            type_url: "/osmosis.tokenfactory.v1beta1.MsgCreateDenom".to_string(),
            value: Binary::from(MsgCreateDenom {
                sender: signer.address(),
                subdenom: denom.clone(),
            }.encode_to_vec()),
        };
        let create_denom_res = app
            .execute_cosmos_msgs::<MsgCreateDenomResponse>(&[create_denom_msg], &signer)
            .unwrap();
        assert_eq!(
            create_denom_res.data.new_token_denom,
            format!("factory/{}/{}", signer.address(), denom)
        );
    }
}
