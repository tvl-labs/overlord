use std::convert::TryFrom;

use alloy_rlp::{encode_list, Decodable, Encodable, Header};
use bytes::BufMut;

use crate::smr::smr_types::Step;
use crate::types::{
    Address, AggregatedChoke, AggregatedVote, Commit, Hash, PoLC, Proof, Proposal, Signature,
    SignedProposal, UpdateFrom, VoteType,
};
use crate::wal::{WalInfo, WalLock};
use crate::Codec;

impl Encodable for VoteType {
    fn encode(&self, out: &mut dyn BufMut) {
        let value: u8 = self.into();
        let enc: [&dyn Encodable; 1] = [&value];
        encode_list::<_, dyn Encodable>(&enc, out);
    }
}

impl<T: Codec> Encodable for SignedProposal<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        let enc: [&dyn Encodable; 2] = [&self.signature, &self.proposal];
        encode_list::<_, dyn Encodable>(&enc, out);
    }
}

impl<T: Codec> Decodable for SignedProposal<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        Ok(SignedProposal {
            signature: Signature::decode(&mut payload)?,
            proposal: Proposal::decode(&mut payload)?,
        })
    }
}

impl Decodable for VoteType {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        let value = u8::decode(&mut payload)?;
        Ok(VoteType::try_from(value).unwrap())
    }
}

impl<T: Codec> Encodable for Proposal<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        let content = bcs::to_bytes(&self.content).unwrap();

        if let Some(polc) = &self.lock {
            let enc: [&dyn Encodable; 7] = [
                &true,
                &self.height,
                &self.round,
                &content,
                &self.block_hash,
                polc,
                &self.proposer,
            ];
            encode_list::<_, dyn Encodable>(&enc, out);
        } else {
            let enc: [&dyn Encodable; 6] = [
                &false,
                &self.height,
                &self.round,
                &content,
                &self.block_hash,
                &self.proposer,
            ];
            encode_list::<_, dyn Encodable>(&enc, out);
        }
    }
}

impl<T: Codec> Decodable for Proposal<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        let has_locked = bool::decode(&mut payload)?;

        if has_locked {
            return Ok(Proposal {
                height: u64::decode(&mut payload)?,
                round: u64::decode(&mut payload)?,
                content: {
                    let buf = <Vec<u8>>::decode(&mut payload)?;
                    bcs::from_bytes(&buf)
                        .map_err(|_| alloy_rlp::Error::Custom("Decode content error."))?
                },
                block_hash: Hash::decode(&mut payload)?,
                lock: Some(PoLC::decode(&mut payload)?),
                proposer: Address::decode(&mut payload)?,
            });
        }

        Ok(Proposal {
            height: u64::decode(&mut payload)?,
            round: u64::decode(&mut payload)?,
            content: {
                let buf = <Vec<u8>>::decode(&mut payload)?;
                bcs::from_bytes(&buf)
                    .map_err(|_| alloy_rlp::Error::Custom("Decode content error."))?
            },
            block_hash: Hash::decode(&mut payload)?,
            lock: None,
            proposer: Address::decode(&mut payload)?,
        })
    }
}

impl<T: Codec> Encodable for Commit<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        let content = bcs::to_bytes(&self.content).unwrap();
        let enc: [&dyn Encodable; 3] = [&self.height, &content, &self.proof];
        encode_list::<_, dyn Encodable>(&enc, out);
    }
}

impl<T: Codec> Decodable for Commit<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        Ok(Commit {
            height: u64::decode(&mut payload)?,
            content: {
                let buf = <Vec<u8>>::decode(&mut payload)?;
                bcs::from_bytes(&buf)
                    .map_err(|_| alloy_rlp::Error::Custom("Decode content error."))?
            },
            proof: Proof::decode(&mut payload)?,
        })
    }
}

impl Encodable for UpdateFrom {
    fn encode(&self, out: &mut dyn BufMut) {
        match self {
            UpdateFrom::PrevoteQC(qc) => {
                let enc: [&dyn Encodable; 2] = [&0u8, qc];
                encode_list::<_, dyn Encodable>(&enc, out);
            }
            UpdateFrom::PrecommitQC(qc) => {
                let enc: [&dyn Encodable; 2] = [&1u8, qc];
                encode_list::<_, dyn Encodable>(&enc, out);
            }
            UpdateFrom::ChokeQC(qc) => {
                let enc: [&dyn Encodable; 2] = [&2u8, qc];
                encode_list::<_, dyn Encodable>(&enc, out);
            }
        }
    }
}

impl Decodable for UpdateFrom {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        let value = u8::decode(&mut payload)?;
        match value {
            0u8 => {
                let qc = AggregatedVote::decode(&mut payload)?;
                Ok(UpdateFrom::PrevoteQC(qc))
            }
            1u8 => {
                let qc = AggregatedVote::decode(&mut payload)?;
                Ok(UpdateFrom::PrecommitQC(qc))
            }
            2u8 => {
                let qc = AggregatedChoke::decode(&mut payload)?;
                Ok(UpdateFrom::ChokeQC(qc))
            }
            _ => Err(alloy_rlp::Error::Custom("Invalid update from.")),
        }
    }
}

impl<T: Codec> Encodable for WalLock<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        let content = bcs::to_bytes(&self.content).unwrap();
        let enc: [&dyn Encodable; 3] = [&self.lock_round, &self.lock_votes, &content];
        encode_list::<_, dyn Encodable>(&enc, out);
    }
}

impl<T: Codec> Decodable for WalLock<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        Ok(WalLock {
            lock_round: u64::decode(&mut payload)?,
            lock_votes: AggregatedVote::decode(&mut payload)?,
            content: {
                let buf = <Vec<u8>>::decode(&mut payload)?;
                bcs::from_bytes(&buf)
                    .map_err(|_| alloy_rlp::Error::Custom("Decode content error."))?
            },
        })
    }
}

impl Encodable for Step {
    fn encode(&self, out: &mut dyn BufMut) {
        let value: u8 = self.into();
        let enc: [&dyn Encodable; 1] = [&value];
        encode_list::<_, dyn Encodable>(&enc, out);
    }
}

impl Decodable for Step {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        let value = u8::decode(&mut payload)?;
        Ok(Step::from(value))
    }
}

impl<T: Codec> Encodable for WalInfo<T> {
    fn encode(&self, out: &mut dyn BufMut) {
        if let Some(lock) = &self.lock {
            let enc: [&dyn Encodable; 6] = [
                &true,
                &self.height,
                &self.round,
                &self.step,
                &lock,
                &self.from,
            ];
            encode_list::<_, dyn Encodable>(&enc, out);
        } else {
            let enc: [&dyn Encodable; 5] =
                [&false, &self.height, &self.round, &self.step, &self.from];
            encode_list::<_, dyn Encodable>(&enc, out);
        }
    }
}

impl<T: Codec> Decodable for WalInfo<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut payload = Header::decode_bytes(buf, true)?;
        let has_locked = bool::decode(&mut payload)?;

        if has_locked {
            return Ok(WalInfo {
                height: u64::decode(&mut payload)?,
                round: u64::decode(&mut payload)?,
                step: Step::decode(&mut payload)?,
                lock: Some(WalLock::decode(&mut payload)?),
                from: UpdateFrom::decode(&mut payload)?,
            });
        }

        Ok(WalInfo {
            height: u64::decode(&mut payload)?,
            round: u64::decode(&mut payload)?,
            step: Step::decode(&mut payload)?,
            from: UpdateFrom::decode(&mut payload)?,
            lock: None,
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use rand::random;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::types::{AggregatedSignature, Choke, Node, SignedChoke, SignedVote, Status, Vote};
    use crate::DurationConfig;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    struct Pill {
        height: u64,
        epoch: Vec<u64>,
    }

    impl Pill {
        fn new() -> Self {
            let height = random::<u64>();
            let epoch = (0..128).map(|_| random::<u64>()).collect::<Vec<_>>();
            Pill { height, epoch }
        }
    }

    impl<T: Codec> SignedProposal<T> {
        fn new(content: T, lock: Option<PoLC>) -> Self {
            SignedProposal {
                signature: gen_signature(),
                proposal: Proposal::new(content, lock),
            }
        }
    }

    impl<T: Codec> Proposal<T> {
        fn new(content: T, lock: Option<PoLC>) -> Self {
            let height = random::<u64>();
            let round = random::<u64>();
            let block_hash = gen_hash();
            let proposer = gen_address();
            Proposal {
                height,
                round,
                content,
                block_hash,
                lock,
                proposer,
            }
        }
    }

    impl PoLC {
        fn new() -> Self {
            PoLC {
                lock_round: random::<u64>(),
                lock_votes: AggregatedVote::new(1u8),
            }
        }
    }

    impl SignedVote {
        fn new(vote_type: u8) -> Self {
            SignedVote {
                signature: gen_signature(),
                vote: Vote::new(vote_type),
                voter: gen_address(),
            }
        }
    }

    impl AggregatedVote {
        fn new(vote_type: u8) -> Self {
            AggregatedVote {
                signature: gen_aggr_signature(),
                vote_type: VoteType::try_from(vote_type).unwrap(),
                height: random::<u64>(),
                round: random::<u64>(),
                block_hash: gen_hash(),
                leader: gen_address(),
            }
        }
    }

    impl Vote {
        fn new(vote_type: u8) -> Self {
            Vote {
                height: random::<u64>(),
                round: random::<u64>(),
                vote_type: VoteType::try_from(vote_type).unwrap(),
                block_hash: gen_hash(),
            }
        }
    }

    impl<T: Codec> Commit<T> {
        fn new(content: T) -> Self {
            let height = random::<u64>();
            let proof = Proof::new();
            Commit {
                height,
                content,
                proof,
            }
        }
    }

    impl Proof {
        fn new() -> Self {
            Proof {
                height: random::<u64>(),
                round: random::<u64>(),
                block_hash: gen_hash(),
                signature: gen_aggr_signature(),
            }
        }
    }

    impl AggregatedChoke {
        fn new() -> Self {
            AggregatedChoke {
                height: random::<u64>(),
                round: random::<u64>(),
                signature: gen_signature(),
                voters: vec![gen_address(), gen_address()],
            }
        }
    }

    impl Choke {
        fn new(from: UpdateFrom) -> Self {
            Choke {
                height: random::<u64>(),
                round: random::<u64>(),
                from,
            }
        }
    }

    impl SignedChoke {
        fn new(from: UpdateFrom) -> Self {
            SignedChoke {
                signature: gen_signature(),
                address: gen_address(),
                choke: Choke::new(from),
            }
        }
    }

    impl Status {
        fn new(time: Option<u64>, is_update_config: bool) -> Self {
            let config = if is_update_config {
                Some(DurationConfig {
                    propose_ratio: random::<u64>(),
                    prevote_ratio: random::<u64>(),
                    precommit_ratio: random::<u64>(),
                    brake_ratio: random::<u64>(),
                })
            } else {
                None
            };

            Status {
                height: random::<u64>(),
                interval: time,
                timer_config: config,
                authority_list: vec![Node::new(gen_address())],
            }
        }
    }

    impl<T: Codec> WalInfo<T> {
        fn new(content: Option<T>) -> Self {
            let lock = if let Some(tmp) = content {
                let polc = PoLC::new();
                Some(WalLock {
                    lock_round: polc.lock_round,
                    lock_votes: polc.lock_votes,
                    content: tmp,
                })
            } else {
                None
            };

            let height = random::<u64>();
            let round = random::<u64>();
            let step = Step::Precommit;
            let from = UpdateFrom::ChokeQC(AggregatedChoke::new());
            WalInfo {
                height,
                round,
                step,
                lock,
                from,
            }
        }
    }

    fn gen_hash() -> Hash {
        Hash::from((0..16).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn gen_address() -> Address {
        Address::from((0..32).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn gen_signature() -> Signature {
        Signature::from((0..64).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn gen_aggr_signature() -> AggregatedSignature {
        AggregatedSignature {
            signature: gen_signature(),
            address_bitmap: Bytes::from((0..8).map(|_| random::<u8>()).collect::<Vec<_>>()),
        }
    }

    #[test]
    fn test_pill_codec() {
        for _ in 0..100 {
            let pill = Pill::new();
            let decode: Pill = bcs::from_bytes(&bcs::to_bytes(&pill).unwrap()).unwrap();
            assert_eq!(decode, pill);
        }
    }

    #[test]
    fn test_types_rlp() {
        // Test SignedProposal
        let signed_proposal = SignedProposal::new(Pill::new(), Some(PoLC::new()));
        let res: SignedProposal<Pill> =
            Decodable::decode(&mut alloy_rlp::encode(&signed_proposal).as_ref()).unwrap();
        assert_eq!(signed_proposal, res);

        let signed_proposal = SignedProposal::new(Pill::new(), None);
        let res: SignedProposal<Pill> =
            Decodable::decode(&mut alloy_rlp::encode(&signed_proposal).as_ref()).unwrap();
        assert_eq!(signed_proposal, res);

        // Test SignedVote
        let signed_vote = SignedVote::new(2u8);
        let res: SignedVote =
            Decodable::decode(&mut alloy_rlp::encode(&signed_vote).as_ref()).unwrap();
        assert_eq!(signed_vote, res);

        let signed_vote = SignedVote::new(1u8);
        let res: SignedVote =
            Decodable::decode(&mut alloy_rlp::encode(&signed_vote).as_ref()).unwrap();
        assert_eq!(signed_vote, res);

        // Test AggregatedVote
        let aggregated_vote = AggregatedVote::new(2u8);
        let res: AggregatedVote =
            Decodable::decode(&mut alloy_rlp::encode(&aggregated_vote).as_ref()).unwrap();
        assert_eq!(aggregated_vote, res);

        let aggregated_vote = AggregatedVote::new(1u8);
        let res: AggregatedVote =
            Decodable::decode(&mut alloy_rlp::encode(&aggregated_vote).as_ref()).unwrap();
        assert_eq!(aggregated_vote, res);

        // Test Commit
        let commit = Commit::new(Pill::new());
        let res: Commit<Pill> =
            Decodable::decode(&mut alloy_rlp::encode(&commit).as_ref()).unwrap();
        assert_eq!(commit, res);

        // Test Status
        let status = Status::new(None, true);
        let res: Status = Decodable::decode(&mut alloy_rlp::encode(&status).as_ref()).unwrap();
        assert_eq!(status, res);

        // Test Status
        let status = Status::new(Some(3000), false);
        let res: Status = Decodable::decode(&mut alloy_rlp::encode(&status).as_ref()).unwrap();
        assert_eq!(status, res);

        // Test Aggregated Choke
        let aggregated_choke = AggregatedChoke::new();
        let res: AggregatedChoke =
            Decodable::decode(&mut alloy_rlp::encode(&aggregated_choke).as_ref()).unwrap();
        assert_eq!(aggregated_choke, res);

        // Test Signed Choke
        let signed_choke = SignedChoke::new(UpdateFrom::PrevoteQC(AggregatedVote::new(1u8)));
        let res: SignedChoke =
            Decodable::decode(&mut alloy_rlp::encode(&signed_choke).as_ref()).unwrap();
        assert_eq!(signed_choke, res);

        let signed_choke = SignedChoke::new(UpdateFrom::PrecommitQC(AggregatedVote::new(2u8)));
        let res: SignedChoke =
            Decodable::decode(&mut alloy_rlp::encode(&signed_choke).as_ref()).unwrap();
        assert_eq!(signed_choke, res);

        let signed_choke = SignedChoke::new(UpdateFrom::ChokeQC(AggregatedChoke::new()));
        let res: SignedChoke =
            Decodable::decode(&mut alloy_rlp::encode(&signed_choke).as_ref()).unwrap();
        assert_eq!(signed_choke, res);

        // Test Wal Info
        let pill = Pill::new();
        let wal_info = WalInfo::new(Some(pill));
        let res: WalInfo<Pill> =
            Decodable::decode(&mut alloy_rlp::encode(&wal_info).as_ref()).unwrap();
        assert_eq!(wal_info, res);

        let wal_info = WalInfo::new(None);
        let res: WalInfo<Pill> =
            Decodable::decode(&mut alloy_rlp::encode(&wal_info).as_ref()).unwrap();
        assert_eq!(wal_info, res);
    }
}
