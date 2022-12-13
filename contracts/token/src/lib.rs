use std::default;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, StorageUsage};

pub mod events;
pub mod internal;
pub mod metadata;
pub mod storage;
pub mod token_core;

use crate::events::*;
use crate::metadata::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
    /// Metadata for the contract itself
    pub metadata: LazyOption<FungibleTokenMetadata>,
    /// The bytes for the largest possible account ID that can be registered on the contract
    pub bytes_for_longest_account_id: StorageUsage,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    Accounts,
    Metadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn init_default(owner_id: AccountId, total_supply: U128) -> Self {
        Self::init(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                name: "MATRIX".to_string(),
                symbol: "MTR".to_string(),
                decimal: 24,
            },
        )
    }

    #[init]
    pub fn init(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        let mut __self = Self {
            total_supply: total_supply.0,
            accounts: LookupMap::new(StorageKey::Accounts.try_to_vec().unwrap()),
            metadata: LazyOption::new(StorageKey::Metadata.try_to_vec().unwrap(), Some(&metadata)),
            bytes_for_longest_account_id: 0,
        };

        __self.measure_bytes_for_longest_account_id();
        __self.internal_register_account(&owner_id);
        __self.internal_deposit(&owner_id, total_supply.into());

        FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial token supply is minted"),
        }
        .emit();

        __self
    }
}
