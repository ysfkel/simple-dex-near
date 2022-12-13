use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, AccountId, Balance, Promise};

use crate::*;
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

pub trait StorageManagement {
    // Returns the StorageBalance structure showing updated balances.
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[allow(unused_variables)]
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        // Get the amount of $NEAR to deposit
        let amount: Balance = env::attached_deposit();
        // If an account was specified, use that. Otherwise, use the predecessor account.
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);

        // If the account is already registered, refund the deposit.
        if self.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        // Register the account and refund any excess $NEAR
        } else {
            // Get the minimum required storage and ensure the deposit is at least that amount
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic_str("The attached deposit is less than the minimum storage balance");
            }

            // Register the account
            self.internal_register_account(&account_id);
            // Perform a refund
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }

        // Return the storage balance of the account
        StorageBalance {
            total: self.storage_balance_bounds().min,
            available: 0.into(),
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        // Calculate the required storage balance by taking the bytes for the longest account ID and multiplying by the current byte cost
        let required_storage_balance =
            Balance::from(self.bytes_for_longest_account_id) * env::storage_byte_cost();

        // Storage balance bounds will have min == max == required_storage_balance
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        // Get the storage balance of the account. Available will always be 0 since you can't overpay for storage.
        if self.accounts.contains_key(&account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}
