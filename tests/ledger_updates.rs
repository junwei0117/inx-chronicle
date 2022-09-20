// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

#[cfg(feature = "rand")]
mod test_rand {
    use std::collections::{HashMap, HashSet};

    use bee_block_stardust::rand::number::rand_number_range;
    use chronicle::{
        db::{
            collections::{
                LedgerUpdateByAddressRecord, LedgerUpdateByMilestoneRecord, LedgerUpdateCollection, SortOrder,
            },
            MongoDb,
        },
        types::{
            ledger::{LedgerOutput, MilestoneIndexTimestamp},
            stardust::block::{
                output::{AddressUnlockCondition, BasicOutput, OutputId},
                BlockId, Output,
            },
        },
    };
    use futures::TryStreamExt;

    use super::common::connect_to_test_db;

    // insert_spent_ledger_updates
    // insert_unspent_ledger_updates
    // stream_ledger_updates_by_address
    // stream_ledger_updates_by_milestone

    // FIXME: there seems to be a bug with adding ledger updates as the number of inserted documents is sometimes wrong.
    #[tokio::test]
    async fn test_ledger_updates_by_address() {
        let (db, collection) = setup("test-ledger-updates-by-address").await;

        let mut outputs = HashSet::new();
        let address_unlock_condition = AddressUnlockCondition::rand();
        let address = address_unlock_condition.clone().address;

        let ledger_outputs = std::iter::repeat_with(|| (BlockId::rand(), rand_number_range(1..1000), OutputId::rand()))
            .take(2)
            .inspect(|(_, _, output_id)| {
                outputs.insert(output_id.clone());
            })
            .map(|(block_id, amount, output_id)| {
                let output = BasicOutput {
                    amount: amount.into(),
                    native_tokens: Vec::new().into_boxed_slice(),
                    address_unlock_condition: address_unlock_condition.clone(),
                    storage_deposit_return_unlock_condition: None,
                    timelock_unlock_condition: None,
                    expiration_unlock_condition: None,
                    features: Vec::new().into_boxed_slice(),
                };

                LedgerOutput {
                    block_id,
                    booked: MilestoneIndexTimestamp {
                        milestone_index: 0.into(),
                        milestone_timestamp: 12345.into(),
                    },
                    output: Output::Basic(output),
                    output_id,
                }
            })
            .chain(
                std::iter::repeat_with(|| (BlockId::rand(), Output::rand(), OutputId::rand()))
                    .take(0)
                    .map(|(block_id, output, output_id)| LedgerOutput {
                        block_id,
                        booked: MilestoneIndexTimestamp {
                            milestone_index: 0.into(),
                            milestone_timestamp: 12345.into(),
                        },
                        output,
                        output_id,
                    }),
            )
            .collect::<Vec<_>>();

        assert_eq!(ledger_outputs.len(), 2);

        collection
            .insert_unspent_ledger_updates(ledger_outputs.iter())
            .await
            .unwrap();

        assert_eq!(collection.len().await.unwrap(), 2);

        let mut s = collection
            .stream_ledger_updates_by_address(&address, 100, None, SortOrder::Newest)
            .await
            .unwrap();

        while let Some(LedgerUpdateByAddressRecord {
            output_id,
            at,
            is_spent,
        }) = s.try_next().await.unwrap()
        {
            assert!(outputs.remove(&output_id));
            assert_eq!(
                at,
                MilestoneIndexTimestamp {
                    milestone_index: 0.into(),
                    milestone_timestamp: 12345.into()
                }
            );
            assert!(!is_spent);
        }
        assert!(outputs.is_empty());

        teardown(db).await;
    }

    #[tokio::test]
    async fn test_ledger_updates_by_milestone() {
        let (db, collection) = setup("test-ledger-updates-by-milestone").await;

        let mut outputs = HashMap::new();
        let ledger_outputs = std::iter::repeat_with(|| (BlockId::rand(), Output::rand_basic(), OutputId::rand()))
            .take(100)
            .enumerate()
            .inspect(|(i, (_, _, output_id))| {
                assert!(outputs.insert(output_id.clone(), *i as u32 / 5).is_none());
            })
            .map(|(i, (block_id, output, output_id))| LedgerOutput {
                block_id,
                booked: MilestoneIndexTimestamp {
                    milestone_index: (i as u32 / 5).into(),
                    milestone_timestamp: (12345 + (i as u32 / 5)).into(),
                },
                output,
                output_id,
            })
            .collect::<Vec<_>>();

        assert_eq!(ledger_outputs.len(), 100);

        collection
            .insert_unspent_ledger_updates(ledger_outputs.iter())
            .await
            .unwrap();

        assert_eq!(collection.len().await.unwrap(), 100);

        let mut s = collection
            .stream_ledger_updates_by_milestone(0.into(), 100, None)
            .await
            .unwrap();

        while let Some(LedgerUpdateByMilestoneRecord {
            output_id, is_spent, ..
        }) = s.try_next().await.unwrap()
        {
            assert_eq!(outputs.remove(&output_id), Some(0));
            assert!(!is_spent);
        }
        assert_eq!(outputs.len(), 95);

        teardown(db).await;
    }

    async fn setup(database_name: impl ToString) -> (MongoDb, LedgerUpdateCollection) {
        let db = connect_to_test_db(database_name).await.unwrap();
        db.clear().await.unwrap();
        let collection = db.collection::<LedgerUpdateCollection>();
        collection.create_indexes().await.unwrap();
        (db, collection)
    }

    async fn teardown(db: MongoDb) {
        db.drop().await.unwrap();
    }
}
