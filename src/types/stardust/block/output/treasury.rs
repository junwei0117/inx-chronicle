// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Borrow;

use bee_block_stardust::output as bee;
use serde::{Deserialize, Serialize};

use super::OutputAmount;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreasuryOutput {
    pub amount: OutputAmount,
}

impl<T: Borrow<bee::TreasuryOutput>> From<T> for TreasuryOutput {
    fn from(value: T) -> Self {
        Self {
            amount: value.borrow().amount().into(),
        }
    }
}

impl TryFrom<TreasuryOutput> for bee::TreasuryOutput {
    type Error = bee_block_stardust::Error;

    fn try_from(value: TreasuryOutput) -> Result<Self, Self::Error> {
        Self::new(value.amount.0)
    }
}

#[cfg(feature = "rand")]
mod rand {
    use bee_block_stardust::rand::output::rand_treasury_output;

    use super::*;

    impl TreasuryOutput {
        /// Generates a random [`TreasuryOutput`].
        pub fn rand() -> Self {
            rand_treasury_output().into()
        }
    }
}

#[cfg(all(test, feature = "rand"))]
mod test {
    use mongodb::bson::{from_bson, to_bson};

    use super::*;

    #[test]
    fn test_treasury_output_bson() {
        let output = TreasuryOutput::rand();
        bee::TreasuryOutput::try_from(output).unwrap();
        let bson = to_bson(&output).unwrap();
        assert_eq!(output, from_bson::<TreasuryOutput>(bson).unwrap());
    }
}
