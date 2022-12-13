use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, ext_contract, near_bindgen, AccountId, Balance, Gas, PromiseOrValue,
    PromiseResult,
};

use crate::*;

pub trait MarketInspect {
    fn get_reserve_0(&self) -> U128;
    fn get_reserve_1(&self) -> U128;
    fn get_token_0(&self) -> AccountId;
    fn get_token_1(&self) -> AccountId;
    fn get_total_supply(&self) -> U128;
    fn get_balance_of(&self, account_id: AccountId) -> U128;
    fn get_received_liquidity_amount(&self, account_id: AccountId, token_id: AccountId) -> U128;
    fn get_received_swap_amount(&self, account_id: AccountId, token_id: AccountId) -> U128;
    fn get_current_account_id(&self) -> AccountId;
}

#[near_bindgen]
impl MarketInspect for Contract {
    fn get_reserve_0(&self) -> U128 {
        self.reserve_0.into()
    }
    fn get_reserve_1(&self) -> U128 {
        self.reserve_1.into()
    }
    fn get_token_0(&self) -> AccountId {
        self.token_0.clone()
    }
    fn get_token_1(&self) -> AccountId {
        self.token_1.clone()
    }
    fn get_total_supply(&self) -> U128 {
        self.total_supply.into()
    }
    fn get_balance_of(&self, account_id: AccountId) -> U128 {
        self.balance.get(&account_id).unwrap_or(0).into()
    }
    fn get_received_liquidity_amount(&self, account_id: AccountId, token_id: AccountId) -> U128 {
        if let Some(liquidity) = self.received_liquidity_amount.get(&account_id) {
            let received = liquidity.get(&token_id).unwrap_or_default();
            return received.amount.into();
        }

        U128(0)
    }

    fn get_received_swap_amount(&self, account_id: AccountId, token_id: AccountId) -> U128 {
        if let Some(liquidity) = self.received_swap_amount.get(&account_id) {
            let received = liquidity.get(&token_id).unwrap_or_default();
            return received.amount.into();
        }

        U128(0)
    }

    fn get_current_account_id(&self) -> AccountId {
        env::current_account_id()
    }
}
