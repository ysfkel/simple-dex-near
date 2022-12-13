use std::default;

use market_types::{LiquidityReceived, SwapReceived};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, StorageUsage};

pub mod constants;
pub mod events;
pub mod external;
pub mod ft_receiver;
pub mod market_core;
pub mod market_inspect;
pub mod market_types;
pub mod market_writer;
pub mod util;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub reserve_0: Balance,
    pub reserve_1: Balance,
    pub total_supply: Balance,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub balance: LookupMap<AccountId, Balance>,
    pub received_liquidity_amount: LookupMap<AccountId, UnorderedMap<AccountId, LiquidityReceived>>,
    pub received_swap_amount: LookupMap<AccountId, UnorderedMap<AccountId, LiquidityReceived>>,
    pub bytes_for_longest_account_id: StorageUsage,
}

#[derive(BorshSerialize)]
pub enum StorageKey {
    Shares,
    ReceivedLiquidityAmount,
    ReceivedSwap,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn init(owner_id: AccountId, _token_0: AccountId, _token_1: AccountId) -> Self {
        let mut __self = Self {
            reserve_0: 0_u128,
            reserve_1: 0_u128,
            token_0: _token_0,
            token_1: _token_1,
            total_supply: 0,
            balance: LookupMap::new(StorageKey::Shares.try_to_vec().unwrap()),
            bytes_for_longest_account_id: 0,
            received_liquidity_amount: LookupMap::new(
                StorageKey::ReceivedLiquidityAmount.try_to_vec().unwrap(),
            ),
            received_swap_amount: LookupMap::new(StorageKey::ReceivedSwap.try_to_vec().unwrap()),
        };
        __self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init() {
        let owner_id: AccountId = "ykel.testnet".parse().unwrap();
        let token_0: AccountId = "token_1.testnet".parse().unwrap();
        let token_1: AccountId = "token_2.testnet".parse().unwrap();
        let contract = Contract::init(owner_id, token_0.clone(), token_1.clone());
        assert_eq!(contract.token_0, token_0);
        assert_eq!(contract.reserve_0, 0);
        assert_eq!(contract.reserve_1, 0);
        assert_eq!(contract.total_supply, 0);
    }

    #[test]
    fn test_add_deposit() {
        let owner_id: AccountId = "ykel.testnet".parse().unwrap();
        let token_0: AccountId = "token_1.testnet".parse().unwrap();
        let token_1: AccountId = "token_2.testnet".parse().unwrap();
        let contract = Contract::init(owner_id, token_0.clone(), token_1.clone());
        assert_eq!(contract.token_0, token_0);
        assert_eq!(contract.reserve_0, 0);
        assert_eq!(contract.reserve_1, 0);
        assert_eq!(contract.total_supply, 0);
    }
}
