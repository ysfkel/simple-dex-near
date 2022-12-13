use near_sdk::json_types::U128;

pub fn get_yocto() -> u128 {
    10u128.pow(24)
}

pub fn to_yocto(num: u128) -> U128 {
    U128::from(num * 10u128.pow(24))
}

pub fn to_dec(num: u128) -> u128 {
    num / get_yocto()
}
