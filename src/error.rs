#[cfg(test)]
use std::cmp::{Eq, PartialEq};
use std::error::Error;

use derive_more::Display;

/// Overlord consensus error.
#[derive(Clone, Debug, Display)]
pub enum ConsensusError {
    ///
    #[display("Invalid address")]
    InvalidAddress,
    ///
    #[display("Channel error {:?}", _0)]
    ChannelErr(String),
    ///
    #[display("Trigger {} SMR error", _0)]
    TriggerSMRErr(String),
    ///
    #[display("Monitor {} event error", _0)]
    MonitorEventErr(String),
    ///
    #[display("Throw {} event error", _0)]
    ThrowEventErr(String),
    ///
    #[display("Proposal error {}", _0)]
    ProposalErr(String),
    ///
    #[display("Prevote error {}", _0)]
    PrevoteErr(String),
    ///
    #[display("Precommit error {}", _0)]
    PrecommitErr(String),
    ///
    #[display("Brake error {}", _0)]
    BrakeErr(String),
    ///
    #[display("Self round is {}, vote round is {}", local, vote)]
    RoundDiff {
        ///
        local: u64,
        ///
        vote: u64,
    },
    ///
    #[display("Self check not pass {}", _0)]
    SelfCheckErr(String),
    ///
    #[display("Correctness error {}", _0)]
    CorrectnessErr(String),
    ///
    #[display("Timer error {}", _0)]
    TimerErr(String),
    ///
    #[display("State error {}", _0)]
    StateErr(String),
    ///
    #[display("Multiple proposal in height {}, round {}", _0, _1)]
    MultiProposal(u64, u64),
    ///
    #[display("Storage error {}", _0)]
    StorageErr(String),
    ///
    #[display("Save Wal error {}, {}, {} step", height, round, step)]
    SaveWalErr {
        ///
        height: u64,
        ///
        round: u64,
        ///
        step: String,
    },
    ///
    #[display("Load Wal error {}", _0)]
    LoadWalErr(String),
    ///
    #[display("Crypto error {}", _0)]
    CryptoErr(String),
    ///
    #[display("Aggregated signature error {}", _0)]
    AggregatedSignatureErr(String),
    /// Other error.
    #[display("Other error {}", _0)]
    Other(String),
}

impl Error for ConsensusError {}

#[cfg(test)]
impl PartialEq for ConsensusError {
    fn eq(&self, other: &Self) -> bool {
        use self::ConsensusError::{
            CorrectnessErr, InvalidAddress, MonitorEventErr, Other, PrecommitErr, PrevoteErr,
            ProposalErr, RoundDiff, SelfCheckErr, ThrowEventErr, TriggerSMRErr,
        };
        match (self, other) {
            // If compare objects are the following types of error, as long as the error type need
            // the same, the details are ignored.
            (InvalidAddress, InvalidAddress)
            | (TriggerSMRErr(_), TriggerSMRErr(_))
            | (MonitorEventErr(_), MonitorEventErr(_))
            | (ThrowEventErr(_), ThrowEventErr(_))
            | (ProposalErr(_), ProposalErr(_))
            | (PrevoteErr(_), PrevoteErr(_))
            | (PrecommitErr(_), PrecommitErr(_))
            | (SelfCheckErr(_), SelfCheckErr(_)) => true,
            // If it is the following two types of errors, in the judgment, the error type need the
            // same, and the error information need the same.
            (RoundDiff { local: m, vote: n }, RoundDiff { local: p, vote: q }) => m == p && n == q,
            (Other(x), Other(y)) | (CorrectnessErr(x), CorrectnessErr(y)) => x == y,
            _ => false,
        }
    }
}

#[cfg(test)]
impl Eq for ConsensusError {}
