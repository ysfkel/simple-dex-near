use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityReceived {
    pub received: bool,
    pub amount: U128,
}

impl Default for LiquidityReceived {
    fn default() -> Self {
        Self {
            received: false,
            amount: U128(0),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapReceived {
    pub amount: U128,
    pub token_in: AccountId,
}

impl Default for SwapReceived {
    fn default() -> Self {
        Self {
            amount: U128(0),
            token_in: "".parse().unwrap(),
        }
    }
}
