use near_sdk::Gas;

pub(crate) const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
pub(crate) const GAS_FOR_FT_TRANSFER_CALL: Gas =
    Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
