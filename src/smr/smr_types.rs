use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::types::{Hash, ViewChangeReason};
use crate::wal::SMRBase;
use crate::DurationConfig;

/// SMR steps. The default step is commit step because SMR needs rich status to start a new block.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    /// Prepose step, in this step:
    /// Firstly, each node calculate the new proposer, then:
    /// Leader:
    ///     proposer a block,
    /// Replica:
    ///     wait for a proposal and check it.
    /// Then goto prevote step.
    #[display("Prepose step")]
    Propose,

    /// Prevote step, in this step:
    /// Leader:
    ///     1. wait for others signed prevote votes,
    ///     2. aggregate them to an aggregated vote,
    ///     3. broadcast the aggregated vote to others.
    /// Replica:
    ///     1. transmit prevote vote,
    ///     2. wait for aggregated vote,
    ///     3. check the aggregated vote.
    /// Then goto precommit step.
    #[display("Prevote step")]
    Prevote,

    /// Precommit step, in this step:
    /// Leader:
    ///     1. wait for others signed precommit votes,
    ///     2. aggregate them to an aggregated vote,
    ///     3. broadcast the aggregated vote to others.
    /// Replica:
    ///     1. transmit precommit vote,
    ///     2. wait for aggregated vote,
    ///     3. check the aggregated vote.
    /// If there is no consensus in the precommit step, goto propose step and start a new round
    /// cycle. Otherwise, goto commit step.
    #[display("Precommit step")]
    Precommit,

    /// Brake step, in this step:
    /// wait for other nodes.
    #[display("Brake step")]
    Brake,

    /// Commit step, in this step each node commit the block and wait for the rich status. After
    /// receiving the it, all nodes will goto propose step and start a new block consensus.
    #[display("Commit step")]
    #[default]
    Commit,
}


impl From<Step> for u8 {
    fn from(step: Step) -> u8 {
        match step {
            Step::Propose => 0,
            Step::Prevote => 1,
            Step::Precommit => 2,
            Step::Brake => 3,
            Step::Commit => 4,
        }
    }
}

impl From<u8> for Step {
    fn from(s: u8) -> Self {
        match s {
            0 => Step::Propose,
            1 => Step::Prevote,
            2 => Step::Precommit,
            3 => Step::Brake,
            4 => Step::Commit,
            _ => panic!("Invalid step!"),
        }
    }
}

///
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FromWhere {
    ///
    PrevoteQC(u64),
    ///
    PrecommitQC(u64),
    ///
    ChokeQC(u64),
}

impl FromWhere {
    pub fn get_round(&self) -> u64 {
        match self {
            FromWhere::PrevoteQC(round) => *round,
            FromWhere::PrecommitQC(round) => *round,
            FromWhere::ChokeQC(round) => *round,
        }
    }

    pub fn to_reason(&self, old_round: u64) -> ViewChangeReason {
        match self {
            FromWhere::PrevoteQC(round) => {
                ViewChangeReason::UpdateFromHigherPrevoteQC(old_round, *round)
            }
            FromWhere::PrecommitQC(round) => {
                ViewChangeReason::UpdateFromHigherPrecommitQC(old_round, *round)
            }
            FromWhere::ChokeQC(round) => {
                ViewChangeReason::UpdateFromHigherChokeQC(old_round, *round)
            }
        }
    }
}

/// SMR event that state and timer monitor this.
/// **NOTICE**: The `height` field is just for the timer. Timer will take this to signal the timer
/// height. State will ignore this field on handling event.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum SMREvent {
    /// New round event,
    /// for state: update round,
    /// for timer: set a propose step timer. If `round == 0`, set an extra total height timer.
    #[display(
        "New round {} event, lock round {:?}, lock proposal {:?}",
        round,
        lock_round,
        lock_proposal
    )]
    NewRoundInfo {
        height: u64,
        round: u64,
        lock_round: Option<u64>,
        lock_proposal: Option<Hash>,
        from_where: FromWhere,
        new_interval: Option<u64>,
        new_config: Option<DurationConfig>,
    },

    /// Prevote event,
    /// for state: transmit a prevote vote,
    /// for timer: set a prevote step timer.
    #[display(
        "Prevote event height {}, round {}, block hash {:?}, lock round {:?}",
        height,
        round,
        "hex_encode(block_hash)",
        lock_round
    )]
    PrevoteVote {
        height: u64,
        round: u64,
        block_hash: Hash,
        lock_round: Option<u64>,
    },

    /// Precommit event,
    /// for state: transmit a precommit vote,
    /// for timer: set a precommit step timer.
    #[display(
        "Precommit event height {}, round {}, block hash {:?}, lock round {:?}",
        height,
        round,
        "hex_encode(block_hash)",
        lock_round
    )]
    PrecommitVote {
        height: u64,
        round: u64,
        block_hash: Hash,
        lock_round: Option<u64>,
    },
    /// Commit event,
    /// for state: do commit,
    /// for timer: do nothing.
    #[display("Commit event hash {:?}", "hex_encode(_0)")]
    Commit(Hash),

    /// Brake event,
    /// for state: broadcast Choke message,
    /// for timer: set a retry timeout timer.
    #[display(
        "Brake event height {}, round {}, lock round {:?}",
        height,
        round,
        lock_round
    )]
    Brake {
        height: u64,
        round: u64,
        lock_round: Option<u64>,
    },

    /// Stop event,
    /// for state: stop process,
    /// for timer: stop process.
    #[display("Stop event")]
    Stop,
}

/// SMR trigger types.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum TriggerType {
    /// Proposal trigger.
    #[display("Proposal")]
    Proposal,
    /// Prevote quorum certificate trigger.
    #[display("PrevoteQC")]
    PrevoteQC,
    /// Precommit quorum certificate trigger.
    #[display("PrecommitQC")]
    PrecommitQC,
    /// New Height trigger.
    #[display("New height")]
    NewHeight(SMRStatus),
    /// Wal infomation.
    #[display("Wal Infomation")]
    WalInfo,
    /// Brake timeout.
    #[display("Brake Timeout")]
    BrakeTimeout,
    /// Continue new round trigger.
    #[display("Continue Round")]
    ContinueRound,
    /// Stop process.
    #[display("Stop Process")]
    Stop,
}

/// SMR trigger sources.
#[derive(Serialize, Deserialize, Clone, Debug, Display, PartialEq, Eq)]
pub enum TriggerSource {
    /// SMR triggered by state.
    #[display("State")]
    State = 0,
    /// SMR triggered by timer.
    #[display("Timer")]
    Timer = 1,
}

impl From<TriggerType> for u8 {
    fn from(t: TriggerType) -> u8 {
        match t {
            TriggerType::Proposal => 0u8,
            TriggerType::PrevoteQC => 1u8,
            TriggerType::PrecommitQC => 2u8,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for TriggerType {
    /// It should not occur that call `from(3u8)`.
    fn from(s: u8) -> Self {
        match s {
            0 => TriggerType::Proposal,
            1 => TriggerType::PrevoteQC,
            2 => TriggerType::PrecommitQC,
            3 => unreachable!(),
            _ => panic!("Invalid trigger type!"),
        }
    }
}

/// A SMR trigger to touch off SMR process. For different trigger type,
/// the field `hash` and `round` have different restrictions and meaning.
/// While trigger type is `Proposal`:
///     * `hash`: Proposal block hash,
///     * `round`: Optional lock round.
/// While trigger type is `PrevoteQC` or `PrecommitQC`:
///     * `hash`: QC block hash,
///     * `round`: QC round, this must be `Some`.
/// While trigger type is `NewHeight`:
///     * `hash`: A empty hash,
///     * `round`: This must be `None`.
/// For each sources, while filling the `SMRTrigger`, the `height` field take the current height
/// directly.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
#[display("{:?} trigger from {:?}, height {}", trigger_type, source, height)]
pub struct SMRTrigger {
    /// SMR trigger type.
    pub trigger_type: TriggerType,
    /// SMR trigger source.
    pub source: TriggerSource,
    /// SMR trigger hash, the meaning shown above.
    pub hash: Hash,
    /// SMR trigger round, the meaning shown above.
    pub lock_round: Option<u64>,
    ///
    pub round: u64,
    /// **NOTICE**: This field is only for timer to signed timer's height. Therefore, the SMR can
    /// filter out the outdated timers.
    pub height: u64,
    ///
    pub wal_info: Option<SMRBase>,
}

/// An inner lock struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    /// Lock round.
    pub round: u64,
    /// Lock hash.
    pub hash: Hash,
}

/// SMR new status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SMRStatus {
    /// New height.
    pub height: u64,
    /// New height interval.
    pub new_interval: Option<u64>,
    /// New timeout configuration.
    pub new_config: Option<DurationConfig>,
}

#[cfg(test)]
impl SMRStatus {
    pub fn new(height: u64) -> Self {
        SMRStatus {
            height,
            new_interval: None,
            new_config: None,
        }
    }
}
