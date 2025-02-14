// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block_stardust::semantic as bee;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum ConflictReason {
    None = 0,
    InputUtxoAlreadySpent = 1,
    InputUtxoAlreadySpentInThisMilestone = 2,
    InputUtxoNotFound = 3,
    CreatedConsumedAmountMismatch = 4,
    InvalidSignature = 5,
    TimelockNotExpired = 6,
    InvalidNativeTokens = 7,
    StorageDepositReturnUnfulfilled = 8,
    InvalidUnlock = 9,
    InputsCommitmentsMismatch = 10,
    UnverifiedSender = 11,
    InvalidChainStateTransition = 12,
    SemanticValidationFailed = 255,
}

impl From<bee::ConflictReason> for ConflictReason {
    fn from(value: bee::ConflictReason) -> Self {
        match value {
            bee::ConflictReason::None => Self::None,
            bee::ConflictReason::InputUtxoAlreadySpent => Self::InputUtxoAlreadySpent,
            bee::ConflictReason::InputUtxoAlreadySpentInThisMilestone => Self::InputUtxoAlreadySpentInThisMilestone,
            bee::ConflictReason::InputUtxoNotFound => Self::InputUtxoNotFound,
            bee::ConflictReason::CreatedConsumedAmountMismatch => Self::CreatedConsumedAmountMismatch,
            bee::ConflictReason::InvalidSignature => Self::InvalidSignature,
            bee::ConflictReason::TimelockNotExpired => Self::TimelockNotExpired,
            bee::ConflictReason::InvalidNativeTokens => Self::InvalidNativeTokens,
            bee::ConflictReason::StorageDepositReturnUnfulfilled => Self::StorageDepositReturnUnfulfilled,
            bee::ConflictReason::InvalidUnlock => Self::InvalidUnlock,
            bee::ConflictReason::InputsCommitmentsMismatch => Self::InputsCommitmentsMismatch,
            bee::ConflictReason::UnverifiedSender => Self::UnverifiedSender,
            bee::ConflictReason::InvalidChainStateTransition => Self::InvalidChainStateTransition,
            bee::ConflictReason::SemanticValidationFailed => Self::SemanticValidationFailed,
        }
    }
}

impl From<ConflictReason> for bee::ConflictReason {
    fn from(value: ConflictReason) -> Self {
        match value {
            ConflictReason::None => bee::ConflictReason::None,
            ConflictReason::InputUtxoAlreadySpent => bee::ConflictReason::InputUtxoAlreadySpent,
            ConflictReason::InputUtxoAlreadySpentInThisMilestone => {
                bee::ConflictReason::InputUtxoAlreadySpentInThisMilestone
            }
            ConflictReason::InputUtxoNotFound => bee::ConflictReason::InputUtxoNotFound,
            ConflictReason::CreatedConsumedAmountMismatch => bee::ConflictReason::CreatedConsumedAmountMismatch,
            ConflictReason::InvalidSignature => bee::ConflictReason::InvalidSignature,
            ConflictReason::TimelockNotExpired => bee::ConflictReason::TimelockNotExpired,
            ConflictReason::InvalidNativeTokens => bee::ConflictReason::InvalidNativeTokens,
            ConflictReason::StorageDepositReturnUnfulfilled => bee::ConflictReason::StorageDepositReturnUnfulfilled,
            ConflictReason::InvalidUnlock => bee::ConflictReason::InvalidUnlock,
            ConflictReason::InputsCommitmentsMismatch => bee::ConflictReason::InputsCommitmentsMismatch,
            ConflictReason::UnverifiedSender => bee::ConflictReason::UnverifiedSender,
            ConflictReason::InvalidChainStateTransition => bee::ConflictReason::InvalidChainStateTransition,
            ConflictReason::SemanticValidationFailed => bee::ConflictReason::SemanticValidationFailed,
        }
    }
}
