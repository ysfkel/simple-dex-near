use near_sdk::json_types::U128;
use near_sdk::ONE_YOCTO;
use near_units::parse_near;
use serde_json::json;
use std::{env, fs};
use workspaces::operations::Function;
use workspaces::result::ValueOrReceiptId;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

fn to_yocto(num: u128) -> U128 {
    U128::from(num * 10u128.pow(24))
}

fn get_token_total_supply() -> U128 {
    to_yocto(1_000_000_000)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token_total_supply = get_token_total_supply();
    let worker = workspaces::sandbox().await?;
    // deploy
    let amm = worker
        .dev_deploy(&std::fs::read(
            "../amm/target/wasm32-unknown-unknown/release/amm.wasm",
        )?)
        .await?;

    let ft_token_0 = worker
        .dev_deploy(&std::fs::read(
            "../token/target/wasm32-unknown-unknown/release/ft_token.wasm",
        )?)
        .await?;

    let ft_token_1 = worker
        .dev_deploy(&std::fs::read(
            "../token/target/wasm32-unknown-unknown/release/ft_token.wasm",
        )?)
        .await?;

    /// init tokens
    let res = ft_token_0
        .call("init_default")
        .args_json(json!({"owner_id":ft_token_0.id(), "total_supply":token_total_supply}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let res = ft_token_1
        .call("init_default")
        .args_json(json!({"owner_id":ft_token_1.id(), "total_supply":token_total_supply}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    // init amm
    let res = amm
        .call("init")
        .args_json(
            json!({"owner_id":amm.id(), "_token_0":ft_token_0.id(), "_token_1":ft_token_1.id()}),
        )
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // create accounts
    let account = worker.dev_create_account().await?;
    let alice = account
        .create_subaccount("alice")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;
    let bob = account
        .create_subaccount("bob")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    let charlie = account
        .create_subaccount("charlie")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    register_user(&ft_token_0, alice.id()).await?;
    register_user(&ft_token_0, amm.id()).await?;
    register_user(&ft_token_0, bob.id()).await?;
    register_user(&ft_token_0, charlie.id()).await?;

    register_user(&ft_token_1, alice.id()).await?;
    register_user(&ft_token_1, amm.id()).await?;
    register_user(&ft_token_1, bob.id()).await?;
    register_user(&ft_token_1, charlie.id()).await?;

    let total_supply: u128 = get_token_total_supply().into();
    let balance = U128(total_supply / 3);

    transfer_tokens(&alice, balance, &ft_token_0).await?;
    transfer_tokens(&alice, balance, &ft_token_1).await?;

    transfer_tokens(&charlie, balance, &ft_token_0).await?;
    transfer_tokens(&charlie, balance, &ft_token_1).await?;
    //tests
    //add liquidity
    test_deposit_liquidity(&alice, &ft_token_0, &ft_token_1, &amm, "ADD_LIQUIDITY").await?;
    test_add_liquidity(&alice, &ft_token_0, &ft_token_1, &amm).await?;
    //swap
    test_wap_token(&alice, &bob, &ft_token_0, &ft_token_1, &amm).await?;
    //remove liquidity
    test_deposit_liquidity(&charlie, &ft_token_0, &ft_token_1, &amm, "ADD_LIQUIDITY").await?;
    test_add_liquidity(&charlie, &ft_token_0, &ft_token_1, &amm).await?;
    test_remove_liquidity(&charlie, &ft_token_0, &ft_token_1, &amm).await?;
    Ok(())
}

async fn register_storage(user: &Account, contract: &Contract) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((user.id(), Option::<bool>::None))
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn transfer_tokens(user: &Account, amount: U128, contract: &Contract) -> anyhow::Result<()> {
    let res = contract
        .call("ft_transfer")
        .args_json(json!({"receiver_id":user.id(), "amount": amount }))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn transfer_to_user(
    sender: &Account,
    receiver: &Account,
    amount: U128,
    contract: &Contract,
) -> anyhow::Result<()> {
    let res = sender
        .call(contract.id(), "ft_transfer")
        .args_json(json!({"receiver_id":receiver.id(), "amount": amount }))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn test_deposit_liquidity(
    user: &Account,
    token_0: &Contract,
    token_1: &Contract,
    amm: &Contract,
    msg: &str,
) -> anyhow::Result<()> {
    // deposit liquidity
    let amount = 100u128;
    ft_transfer_call(user, token_0, amm, to_yocto(amount), msg).await?;
    ft_transfer_call(user, token_1, amm, to_yocto(amount), msg).await?;
    verify_received_liquidity_amount(user, token_0, amm, to_yocto(amount).into()).await?;
    verify_received_liquidity_amount(user, token_1, amm, to_yocto(amount).into()).await?;
    Ok(())
}

async fn test_add_liquidity(
    user: &Account,
    token_0: &Contract,
    token_1: &Contract,
    amm: &Contract,
) -> anyhow::Result<()> {
    verify_received_liquidity_amount(user, token_0, amm, to_yocto(100u128).into()).await?;
    verify_received_liquidity_amount(user, token_1, amm, to_yocto(100u128).into()).await?;
    add_liquidity(user, amm).await?;
    verify_received_liquidity_amount(user, token_0, amm, U128(0u128).into()).await?;
    verify_received_liquidity_amount(user, token_1, amm, U128(0u128).into()).await?;

    Ok(())
}

async fn test_wap_token(
    alice: &Account,
    bob: &Account,
    token_0: &Contract,
    token_1: &Contract,
    amm: &Contract,
) -> anyhow::Result<()> {
    verify_token_balance_eq(bob, token_1).await?;
    transfer_to_user(alice, bob, to_yocto(10u128), token_0).await?;
    ft_transfer_call(bob, token_0, amm, to_yocto(10u128), "SWAP_TOKEN").await?;
    swap(bob, token_0, amm).await?;
    verify_token_balance_greater_than(bob, token_1).await?;
    Ok(())
}

async fn test_remove_liquidity(
    user: &Account,
    token_0: &Contract,
    token_1: &Contract,
    amm: &Contract,
) -> anyhow::Result<()> {
    let mut token0_balance1 = get_token_balance(user, token_0).await?;
    let mut token1_balance1 = get_token_balance(user, token_1).await?;
    let res = get_shares(user, amm).await?;
    let mut shares: u128 = res.parse().unwrap();
    remove_liquidity(user, shares.into(), amm).await?;
    let res = get_shares(user, amm).await?;
    shares = res.parse().unwrap();
    assert_eq!(shares, 0);
    let mut token0_balance2 = get_token_balance(user, token_0).await?;
    let mut token1_balance2 = get_token_balance(user, token_1).await?;
    let _token0_balance1: u128 = token0_balance1.parse().unwrap();
    let _token1_balance1: u128 = token1_balance1.parse().unwrap();
    let _token0_balance2: u128 = token0_balance2.parse().unwrap();
    let _token1_balance2: u128 = token1_balance2.parse().unwrap();
    assert!(_token0_balance1 < _token0_balance2);
    assert!(_token1_balance1 < _token1_balance2);
    Ok(())
}

async fn add_liquidity(user: &Account, amm: &Contract) -> anyhow::Result<()> {
    let res = user
        .call(amm.id(), "add_liquidity")
        .args_json(json!({}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn remove_liquidity(user: &Account, shares: U128, amm: &Contract) -> anyhow::Result<()> {
    let res = user
        .call(amm.id(), "remove_liquidity")
        .args_json(json!({ "_shares": shares }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn swap(user: &Account, token: &Contract, amm: &Contract) -> anyhow::Result<()> {
    let res = user
        .call(amm.id(), "swap")
        .args_json(json!({ "token_id": token.id() }))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn ft_transfer_call(
    user: &Account,
    token: &Contract,
    amm: &Contract,
    amount: U128,
    msg: &str,
) -> anyhow::Result<()> {
    let res = user
        .call(token.id(), "ft_transfer_call")
        .args_json(json!({"receiver_id":amm.id(),"amount":amount,"msg":msg}))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());
    Ok(())
}

async fn verify_token_balance_greater_than(user: &Account, token: &Contract) -> anyhow::Result<()> {
    let res: String = token
        .call("ft_balance_of")
        .args_json(json!({"account_id":user.id()}))
        .max_gas()
        .transact()
        .await?
        .json()?;
    let token_amount: u128 = res.parse().unwrap();
    assert!(token_amount > 0u128);
    Ok(())
}

async fn verify_token_balance_eq(user: &Account, token: &Contract) -> anyhow::Result<()> {
    let res: String = token
        .call("ft_balance_of")
        .args_json(json!({"account_id":user.id()}))
        .max_gas()
        .transact()
        .await?
        .json()?;
    let token_amount: u128 = res.parse().unwrap();
    assert_eq!(token_amount, 0u128);
    Ok(())
}

async fn verify_received_liquidity_amount(
    user: &Account,
    token: &Contract,
    amm: &Contract,
    amount: u128,
) -> anyhow::Result<()> {
    let res: String = amm
        .call("get_received_liquidity_amount")
        .args_json(json!({"account_id":user.id(),"token_id":token.id()}))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?
        .json()?;
    assert_eq!(res, amount.to_string());
    Ok(())
}

async fn verify_reserve_has_correct_amount(
    user: &Account,
    amm: &Contract,
    amount: u128,
    function: &str,
) -> anyhow::Result<()> {
    let res: String = amm
        .call(function)
        .args_json(json!({}))
        .max_gas()
        .transact()
        .await?
        .json()?;
    assert_eq!(res, amount.to_string());
    Ok(())
}

async fn verify_user_shares_has_correct_amount(
    user: &Account,
    amm: &Contract,
    amount: u128,
    function: &str,
) -> anyhow::Result<()> {
    let res: String = amm
        .call(function)
        .args_json(json!({"account_id": user.id()}))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?
        .json()?;

    let shares: u128 = res.parse().unwrap();
    assert!(shares > 0u128);
    Ok(())
}

async fn register_user(contract: &Contract, account_id: &AccountId) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((account_id, Option::<bool>::None))
        .max_gas()
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}

async fn get_shares(
    user: &Account,
    amm: &Contract,
) -> anyhow::Result<String, workspaces::error::Error> {
    let res: String = amm
        .call("get_balance_of")
        .args_json(json!({"account_id":user.id()}))
        .max_gas()
        .transact()
        .await?
        .json()?;
    Ok(res)
}

async fn get_token_balance(
    user: &Account,
    token: &Contract,
) -> anyhow::Result<String, workspaces::error::Error> {
    let res: String = token
        .call("ft_balance_of")
        .args_json(json!({"account_id":user.id()}))
        .max_gas()
        .transact()
        .await?
        .json()?;
    Ok(res)
}
