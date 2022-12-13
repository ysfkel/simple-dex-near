use near_sdk::{require, Balance, PromiseResult};

use crate::*;

const ADD_LIQUIDITY: &str = "ADD_LIQUIDITY";
const SWAP_TOKEN: &str = "SWAP_TOKEN";

trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128;
    fn process_received_liquidity(
        &mut self,
        transaction_sender_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) -> U128;
    fn process_swap(&mut self, token_in: AccountId, sender_id: AccountId, _amount: U128) -> U128;
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        let transaction_sender_id = env::predecessor_account_id();

        require!(msg == ADD_LIQUIDITY || msg == SWAP_TOKEN, "INVALID_ACTION");

        require!(
            transaction_sender_id == self.token_0 || transaction_sender_id == self.token_1,
            "INVALID_TOKEN"
        );

        match msg.as_str() {
            ADD_LIQUIDITY => {
                self.process_received_liquidity(transaction_sender_id, sender_id, amount)
            }
            SWAP_TOKEN => self.process_swap(transaction_sender_id, sender_id, amount),
            _ => env::panic_str("INVALID_ACTION"),
        };
        U128(0)
    }

    fn process_received_liquidity(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        _amount: U128,
    ) -> U128 {
        let _amount: Balance = _amount.into();

        let liquidity_item = match self.received_liquidity_amount.get(&sender_id) {
            Some(_liquidity_item) => {
                let mut liquidity_item = _liquidity_item;

                let mut received = liquidity_item.get(&token_id).unwrap_or_default();

                received.amount = U128(
                    _amount
                        .checked_add(received.amount.into())
                        .unwrap_or_else(|| env::panic_str("PANICK_WHILE_UPDATING_SWAP_AMOUNT")),
                );
                received.received = true;
                liquidity_item.insert(&token_id, &received);
                liquidity_item
            }
            None => {
                let prefix = get_storage_prefix(sender_id.clone());

                let mut liquidity_item: UnorderedMap<AccountId, LiquidityReceived> =
                    UnorderedMap::new(prefix);

                let received = LiquidityReceived {
                    received: true,
                    amount: _amount.into(),
                };
                liquidity_item.insert(&token_id, &received);
                liquidity_item
            }
        };

        self.received_liquidity_amount
            .insert(&sender_id, &liquidity_item);

        U128(0)
    }

    fn process_swap(&mut self, token_id: AccountId, sender_id: AccountId, _amount: U128) -> U128 {
        let _amount: Balance = _amount.into();

        let swap_item = match self.received_swap_amount.get(&sender_id) {
            Some(_swap_item) => {
                let mut swap_item = _swap_item;
                let mut received = swap_item.get(&token_id).unwrap_or_default();
                received.amount = U128(
                    _amount
                        .checked_add(received.amount.into())
                        .unwrap_or_else(|| env::panic_str("PANICK_WHILE_UPDATING_SWAP_AMOUNT")),
                );
                received.received = true;
                swap_item.insert(&token_id, &received);
                swap_item
            }
            None => {
                let prefix = get_storage_prefix(sender_id.clone());
                let mut swap_item: UnorderedMap<AccountId, LiquidityReceived> =
                    UnorderedMap::new(prefix);
                let received = LiquidityReceived {
                    received: true,
                    amount: _amount.into(),
                };
                swap_item.insert(&token_id, &received);
                swap_item
            }
        };

        self.received_swap_amount.insert(&sender_id, &swap_item);

        U128(0)
    }
}

pub fn get_storage_prefix(account_id: AccountId) -> Vec<u8> {
    let prefix: Vec<u8> = [
        b"s".as_slice(),
        &near_sdk::env::sha256_array(account_id.as_bytes()),
    ]
    .concat();
    prefix
}
