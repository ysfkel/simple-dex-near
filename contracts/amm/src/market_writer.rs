use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, ext_contract, near_bindgen, AccountId, Balance, Gas, PromiseOrValue,
    PromiseResult,
};

use crate::constants::{GAS_FOR_FT_TRANSFER_CALL, GAS_FOR_RESOLVE_TRANSFER};
use crate::external::ext_ft_contract;
use crate::market_inspect::MarketInspect;
use crate::*;

pub(crate) trait MarketWriter {
    fn update_reserve_0(&mut self);
    fn update_reserve_1(&mut self);
    fn resolve_reserve_0(&mut self) -> U128;
    fn resolve_reserve_1(&mut self) -> U128;
    fn mint(&mut self, sender_id: AccountId, shares: Balance);
    fn burn(&mut self, sender_id: AccountId, shares: Balance);
    fn set_received_liquidity_amount(&mut self, account_id: AccountId, token: AccountId);
    fn set_received_swap_amount(&mut self, sender_id: AccountId, token_id: AccountId);
}

#[near_bindgen]
impl MarketWriter for Contract {
    fn update_reserve_0(&mut self) {
        let current_account_id = self.get_current_account_id();
        ext_ft_contract::ext(self.token_0.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
            .ft_balance_of(current_account_id.clone())
            .then(
                Self::ext(current_account_id)
                    .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
                    .resolve_reserve_0(),
            );
    }

    fn update_reserve_1(&mut self) {
        let current_account_id = self.get_current_account_id();
        ext_ft_contract::ext(self.token_1.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
            .ft_balance_of(current_account_id.clone())
            .then(
                Self::ext(current_account_id)
                    .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
                    .resolve_reserve_1(),
            );
    }

    #[private]
    fn mint(&mut self, sender_id: AccountId, shares: Balance) {
        let mut _balance = self.balance.get(&sender_id).unwrap_or(0);
        let new_balance = _balance
            .checked_add(shares)
            .unwrap_or_else(|| env::panic_str("FAILED_MINT_SHARES"));
        self.balance.insert(&sender_id, &new_balance);
        self.total_supply = self.total_supply + shares;
    }

    fn burn(&mut self, sender_id: AccountId, shares: Balance) {
        let mut _balance = self.balance.get(&sender_id).unwrap_or(0);
        let new_balance = _balance
            .checked_sub(shares)
            .unwrap_or_else(|| env::panic_str("FAILED_BURN_SHARES"));
        self.balance.insert(&sender_id, &new_balance);
        self.total_supply = self.total_supply - shares;
    }

    #[private]
    fn resolve_reserve_0(&mut self) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result");

        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }

            PromiseResult::Successful(result) => {
                if let Ok(balance) = near_sdk::serde_json::from_slice::<U128>(&result) {
                    self.reserve_0 = balance.into();
                    balance
                } else {
                    env::panic_str("reserve_0_update_failed")
                }
            }

            PromiseResult::Failed => U128(0),
        }
    }

    #[private]
    fn resolve_reserve_1(&mut self) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "Expected 1 promise result");

        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }

            PromiseResult::Successful(result) => {
                if let Ok(balance) = near_sdk::serde_json::from_slice::<U128>(&result) {
                    self.reserve_1 = balance.into();
                    balance
                } else {
                    env::panic_str("reserve_1_update_failed")
                }
            }

            PromiseResult::Failed => U128(0),
        }
    }

    #[private]
    fn set_received_liquidity_amount(&mut self, sender_id: AccountId, token_id: AccountId) {
        if let Some(_liquidity_item) = self.received_liquidity_amount.get(&sender_id) {
            let mut liquidity_item = _liquidity_item;
            let mut received = liquidity_item.get(&token_id).unwrap_or_default();
            received.amount = U128(0);
            liquidity_item.insert(&token_id, &received);
            self.received_liquidity_amount
                .insert(&sender_id, &liquidity_item);
        }
    }

    #[private]
    fn set_received_swap_amount(&mut self, sender_id: AccountId, token_id: AccountId) {
        if let Some(_swap_item) = self.received_swap_amount.get(&sender_id) {
            let mut swap_item = _swap_item;
            let mut received = swap_item.get(&token_id).unwrap_or_default();
            received.amount = U128(0);
            swap_item.insert(&token_id, &received);
            self.received_swap_amount.insert(&sender_id, &swap_item);
        }
    }
}
