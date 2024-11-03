use margined_neutron_std::types::cosmos::adminmodule::adminmodule::{
    MsgAddAdmin, MsgAddAdminResponse, MsgDeleteAdmin, MsgDeleteAdminResponse, MsgSubmitProposal,
    MsgSubmitProposalResponse, QueryAdminsRequest, QueryAdminsResponse,
};
use test_tube_ntrn::{fn_execute, fn_query};

use test_tube_ntrn::module::Module;
use test_tube_ntrn::runner::Runner;

pub struct Admin<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Admin<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Admin<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub add_admin: MsgAddAdmin["/cosmos.adminmodule.adminmodule.MsgAddAdmin"] => MsgAddAdminResponse
    }

    fn_execute! {
        pub delete_admin: MsgDeleteAdmin["/cosmos.adminmodule.adminmodule.MsgDeleteAdmin"] => MsgDeleteAdminResponse
    }

    fn_execute! {
        pub submit_proposal: MsgSubmitProposal => MsgSubmitProposalResponse
    }

    fn_query! {
        pub query_admins ["/cosmos.adminmodule.adminmodule.Query/Admins"]: QueryAdminsRequest => QueryAdminsResponse
    }
}
