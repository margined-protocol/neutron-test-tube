use margined_neutron_std::types::cosmos::gov::v1::{
    MsgSubmitProposal, MsgSubmitProposalResponse, MsgVote, MsgVoteResponse, QueryProposalRequest,
    QueryProposalResponse,
};
use margined_neutron_std::types::cosmos::gov::v1beta1;
use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;
use test_tube_ntrn::{fn_execute, fn_query};

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
}
