use crate::view::{
    accumulator_proof_view::AccumulatorProofView, event_with_proof_view::EventWithProofView,
    state_with_proof_view::StateWithProofView, transaction_info_view::TransactionInfoView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionInfoWithProofView {
    pub transaction_info: TransactionInfoView,
    pub proof: AccumulatorProofView,
    pub event_proof: Option<EventWithProofView>,
    pub state_proof: Option<StateWithProofView>,
}

// impl From<TransactionInfoWithProof> for TransactionInfoWithProofView {
//     fn from(origin: TransactionInfoWithProof) -> Self {
//         Self {
//             transaction_info: origin.transaction_info.into(),
//             proof: origin.proof.into(),
//             event_proof: origin.event_proof.map(Into::into),
//             state_proof: origin.state_proof.map(Into::into),
//         }
//     }
// }
//
// impl TryFrom<TransactionInfoWithProofView> for TransactionInfoWithProof {
//     type Error = anyhow::Error;
//
//     fn try_from(view: TransactionInfoWithProofView) -> Result<Self, Self::Error> {
//         Ok(Self {
//             transaction_info: view.transaction_info.try_into()?,
//             proof: view.proof.into(),
//             event_proof: view.event_proof.map(TryInto::try_into).transpose()?,
//             state_proof: view.state_proof.map(Into::into),
//         })
//     }
// }
