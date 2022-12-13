use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId, PromiseOrValue};

#[ext_contract(ext_ft_contract)]
pub trait ExtFtContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: AccountId) -> PromiseOrValue<U128>;
}
