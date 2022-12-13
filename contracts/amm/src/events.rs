use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::AccountId;

use near_sdk::env;

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[must_use = "don't forget to `.emit()` this event"]
#[serde(rename_all = "snake_case")]
pub(crate) enum NearEvent<'a> {
    Nep141(Nep141Event<'a>),
}

impl<'a> NearEvent<'a> {
    fn to_json_string(&self) -> String {
        #[allow(clippy::redundant_closure)]
        serde_json::to_string(self)
            .ok()
            .unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }
    pub(crate) fn emit(self) {
        near_sdk::env::log_str(&self.to_json_event_string());
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct LiquidityAdded<'a> {
    pub account_id: &'a AccountId,
    pub shares: &'a U128,
    pub amount_0: &'a U128,
    pub amount_1: &'a U128,
}

impl LiquidityAdded<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }
    pub fn emit_many(data: &[LiquidityAdded<'_>]) {
        new_141_v1(Nep141EventKind::LiquidityAdded(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct LiquidityRemoved<'a> {
    pub account_id: &'a AccountId,
    pub shares: &'a U128,
    pub amount_0: &'a U128,
    pub amount_1: &'a U128,
}

impl LiquidityRemoved<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }
    pub fn emit_many(data: &[LiquidityRemoved<'_>]) {
        new_141_v1(Nep141EventKind::LiquidityRemoved(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct LiquidityReceived<'a> {
    pub account_id: &'a AccountId,
    pub token_id: &'a AccountId,
    pub amount: &'a U128,
}

impl LiquidityReceived<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }
    pub fn emit_many(data: &[LiquidityReceived<'_>]) {
        new_141_v1(Nep141EventKind::LiquidityReceived(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct TokensSwaped<'a> {
    pub account_id: &'a AccountId,
    pub token_in: &'a AccountId,
    pub amount_out: &'a U128,
}

impl TokensSwaped<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[TokensSwaped<'_>]) {
        new_141_v1(Nep141EventKind::TokensSwaped(data)).emit()
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct Nep141Event<'a> {
    version: &'static str,
    #[serde(flatten)]
    event_kind: Nep141EventKind<'a>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum Nep141EventKind<'a> {
    LiquidityAdded(&'a [LiquidityAdded<'a>]),
    TokensSwaped(&'a [TokensSwaped<'a>]),
    LiquidityRemoved(&'a [LiquidityRemoved<'a>]),
    LiquidityReceived(&'a [LiquidityReceived<'a>]),
}

fn new_141<'a>(version: &'static str, event_kind: Nep141EventKind<'a>) -> NearEvent<'a> {
    NearEvent::Nep141(Nep141Event {
        version,
        event_kind,
    })
}

fn new_141_v1(event_kind: Nep141EventKind) -> NearEvent {
    new_141("1.0.0", event_kind)
}
