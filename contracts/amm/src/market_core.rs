use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, ext_contract, near_bindgen, require, AccountId, Balance, Gas, PromiseOrValue,
    PromiseResult,
};
use num_integer::Roots;
use std::cmp;

use crate::events::{LiquidityAdded, LiquidityRemoved, TokensSwaped};
use crate::external::ext_ft_contract;
use crate::market_inspect::MarketInspect;
use crate::market_writer::MarketWriter;
use crate::util::{get_yocto, to_dec, to_yocto};

use crate::*;

pub const DENOM: u128 = 1_000_000_000_000_000_000_000_000;

pub trait MarketMakerCore {
    fn add_liquidity(&mut self) -> U128;
    fn remove_liquidity(&mut self, shares: U128);
    fn swap(&mut self, tokenIn: AccountId) -> U128;
}

#[near_bindgen]
impl MarketMakerCore for Contract {
    fn add_liquidity(&mut self) -> U128 {
        let sender_id = env::predecessor_account_id();
        let amount_0: Balance = self
            .get_received_liquidity_amount(sender_id.clone(), self.token_0.clone())
            .into();
        let amount_1: Balance = self
            .get_received_liquidity_amount(sender_id.clone(), self.token_1.clone())
            .into();

        self.set_received_liquidity_amount(sender_id.clone(), self.get_token_0());
        self.set_received_liquidity_amount(sender_id.clone(), self.get_token_1());

        if self.reserve_0 > 0_u128 || self.reserve_1 > 0_u128 {
            require!(
                self.reserve_0.checked_mul(amount_1.into())
                    == self.reserve_1.checked_mul(amount_0.into()),
                "x / y != dx / dy"
            );
        }

        let amount_0_dec = to_dec(amount_0);

        let amount_1_dec = to_dec(amount_1);

        let shares: Balance = if self.total_supply == 0_u128 {
            to_yocto(
                (amount_0_dec
                    .checked_mul(amount_1_dec)
                    .unwrap_or_else(|| env::panic_str("failed_add_liquidity__0")))
                .sqrt(),
            )
            .into()
        } else {
            let _amount_0 = amount_0_dec
                .checked_mul(to_dec(self.total_supply))
                .unwrap_or_else(|| env::panic_str("failed_add_liquidity__1"))
                .checked_div(to_dec(self.reserve_0))
                .unwrap_or_else(|| env::panic_str("failed_add_liquidity__2"));

            let _amount_1 = amount_1_dec
                .checked_mul(to_dec(self.total_supply))
                .unwrap_or_else(|| env::panic_str("failed_add_liquidity__3"))
                .checked_div(to_dec(self.reserve_1))
                .unwrap_or_else(|| env::panic_str("failed_add_liquidity__4"));

            to_yocto(cmp::min(_amount_0, _amount_1)).into()
        };

        require!(shares > 0, "shares_0");

        self.mint(sender_id.clone(), shares.clone());

        self.update_reserve_0();
        self.update_reserve_1();

        LiquidityAdded {
            account_id: &sender_id,
            shares: &shares.into(),
            amount_0: &amount_0.into(),
            amount_1: &amount_1.into(),
        }
        .emit();

        U128(shares)
    }

    fn remove_liquidity(&mut self, _shares: U128) {
        let sender_id = env::predecessor_account_id();

        require!(
            _shares <= self.get_balance_of(sender_id.clone()),
            "no_balance"
        );

        let shares: Balance = _shares.into();

        self.burn(sender_id.clone(), shares.clone());

        let shares_dec = to_dec(shares);

        let amount_0: Balance = to_yocto(
            ((shares_dec.checked_mul(to_dec(self.reserve_0)))
                .unwrap_or_else(|| env::panic_str("failed_remove_liquidity__0")))
                / to_dec(self.total_supply),
        )
        .into();

        let amount_1: Balance = to_yocto(
            ((shares_dec.checked_mul(to_dec(self.reserve_1)))
                .unwrap_or_else(|| env::panic_str("failed_remove_liquidity__1")))
                / to_dec(self.total_supply),
        )
        .into();

        require!(amount_0 > 0 && amount_1 > 0, "amount_0 = 0 or amount_1 = 0");

        ext_ft_contract::ext(self.get_token_0())
            .with_attached_deposit(1)
            .ft_transfer(
                sender_id.clone(),
                amount_0.into(),
                Some("transfer amount_0".to_string()),
            );

        ext_ft_contract::ext(self.get_token_1())
            .with_attached_deposit(1)
            .ft_transfer(
                sender_id.clone(),
                amount_1.into(),
                Some("transfer amount_1".to_string()),
            );

        LiquidityRemoved {
            account_id: &sender_id,
            shares: &shares.into(),
            amount_0: &amount_0.into(),
            amount_1: &amount_1.into(),
        }
        .emit();
    }

    fn swap(&mut self, token_id: AccountId) -> U128 {
        require!(
            token_id == self.get_token_0() || token_id == self.token_1,
            "INVALID_TOKEN"
        );

        let sender_id = env::predecessor_account_id();

        require!(
            self.get_received_swap_amount(sender_id.clone(), token_id.clone()) > U128(0),
            "AMOUNT_0"
        );

        let amount_in = self.get_received_swap_amount(sender_id.clone(), token_id.clone());
        self.set_received_swap_amount(sender_id.clone(), token_id.clone());

        let is_token_0 = token_id == self.get_token_0();

        let (token_in, token_out, reserve_in, reserve_out) = if is_token_0 {
            (
                self.get_token_0(),
                self.get_token_1(),
                self.reserve_0,
                self.reserve_1,
            )
        } else {
            (
                self.get_token_1(),
                self.get_token_0(),
                self.reserve_0,
                self.reserve_1,
            )
        };

        let _amount_in_dec = to_dec(amount_in.into());
        let _reserve_in_dec = to_dec(reserve_in);
        let _reserve_out_dec = to_dec(reserve_out);

        let amount_in_with_fees = (_amount_in_dec.checked_mul(97))
            .unwrap_or_else(|| env::panic_str("failed_amount_in_with_fees__0"))
            .checked_div(100)
            .unwrap_or_else(|| env::panic_str("failed_amount_in_with_fees_1"));

        let _amount_out = to_yocto(
            (_reserve_out_dec
                .checked_mul(amount_in_with_fees)
                .unwrap_or_else(|| env::panic_str("failed_amount_out__0")))
            .checked_div(
                _reserve_in_dec
                    .checked_add(amount_in_with_fees)
                    .unwrap_or_else(|| env::panic_str("failed_amount_out__1")),
            )
            .unwrap_or_else(|| env::panic_str("failed_amount_out__2")),
        );

        ext_ft_contract::ext(token_out)
            .with_attached_deposit(1)
            .ft_transfer(
                sender_id.clone(),
                _amount_out.clone(),
                Some("TRANSFER_SWAPPED_TOKEN".to_string()),
            );

        self.update_reserve_0();
        self.update_reserve_1();

        TokensSwaped {
            account_id: &sender_id,
            token_in: &token_in.into(),
            amount_out: &_amount_out.into(),
        }
        .emit();

        _amount_out
    }
}
