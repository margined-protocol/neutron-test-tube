use cosmrs::tx::MessageExt;
use margined_neutron_std::shim::Any;
use margined_neutron_std::types::cosmos::{
    adminmodule::adminmodule::MsgSubmitProposal,
    gov::{
        v1::{
            MsgSubmitProposalResponse, MsgVote, MsgVoteResponse, QueryParamsRequest,
            QueryParamsResponse, QueryProposalRequest, QueryProposalResponse,
            QueryProposalsRequest, QueryProposalsResponse, VoteOption,
        },
        v1beta1,
    },
};
use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;
use test_tube_ntrn::{
    fn_execute, fn_query, Account, RunnerError, RunnerExecuteResult, SigningAccount,
};

use crate::NeutronTestApp;

pub struct Gov<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Gov<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Gov<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub submit_proposal: MsgSubmitProposal => MsgSubmitProposalResponse
    }

    fn_execute! {
        pub submit_proposal_v1beta1: v1beta1::MsgSubmitProposal => v1beta1::MsgSubmitProposalResponse

    }

    fn_execute! {
        pub vote: MsgVote => MsgVoteResponse
    }

    fn_query! {
        pub query_proposal ["/cosmos.gov.v1beta1.Query/Proposal"]: QueryProposalRequest => QueryProposalResponse
    }

    fn_query! {
        pub query_proposals ["/cosmos.gov.v1beta1.Query/Proposals"]: QueryProposalsRequest => QueryProposalsResponse
    }

    fn_query! {
        pub query_params ["/cosmos.gov.v1beta1.Query/Params"]: QueryParamsRequest => QueryParamsResponse
    }

    pub fn submit_executable_proposal<M: prost::Message>(
        &self,
        msg_type_url: String,
        msg: M,
        proposer: String,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgSubmitProposalResponse> {
        self.submit_proposal(
            MsgSubmitProposal {
                messages: vec![Any {
                    type_url: msg_type_url,
                    value: msg
                        .to_bytes()
                        .map_err(|e| RunnerError::EncodeError(e.into()))?,
                }],
                proposer,
            },
            signer,
        )
    }
}

/// Extension for Gov module
/// It has ability to access to `NeutronTestApp` which is more specific than `Runner`
pub struct GovWithAppAccess<'a> {
    gov: Gov<'a, NeutronTestApp>,
    app: &'a NeutronTestApp,
}

impl<'a> GovWithAppAccess<'a> {
    pub fn new(app: &'a NeutronTestApp) -> Self {
        Self {
            gov: Gov::new(app),
            app,
        }
    }

    pub fn to_gov(&self) -> &Gov<'a, NeutronTestApp> {
        &self.gov
    }

    pub fn propose_and_execute<M: prost::Message>(
        &self,
        msg_type_url: String,
        msg: M,
        proposer: String,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgSubmitProposalResponse> {
        // submit proposal
        let submit_proposal_res = self.gov.submit_proposal(
            MsgSubmitProposal {
                messages: vec![Any {
                    type_url: msg_type_url,
                    value: msg
                        .to_bytes()
                        .map_err(|e| RunnerError::EncodeError(e.into()))?,
                }],
                proposer,
            },
            signer,
        )?;

        let proposal_id = submit_proposal_res.data.proposal_id;

        // get validator to vote yes for proposal
        let val = self
            .app
            .get_first_validator_signing_account("untrn".to_string(), 1.3)?;

        self.gov
            .vote(
                MsgVote {
                    proposal_id,
                    voter: val.address(),
                    option: VoteOption::Yes.into(),
                    metadata: "".to_string(),
                },
                &val,
            )
            .unwrap();

        // increase time to pass voting period
        // self.app.increase_time(voting_period.seconds as u64 + 1);

        Ok(submit_proposal_res)
    }
}
