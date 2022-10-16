#![cfg(test)]
extern crate std;
use soroban_sdk::testutils::Logger;

use crate::testutils::{register_test_contract as register_lottery_contract, LotteryContract};
use crate::token::{self, TokenMetadata};
use rand::{thread_rng, RngCore};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{testutils::Accounts, AccountId, BigInt, BytesN, Env, IntoVal};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn create_token_contract(e: &Env, admin: &AccountId) -> ([u8; 32], token::Client) {
    let id = e.register_contract_token(None);
    let token = token::Client::new(e, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "USD coin".into_val(e),
            symbol: "USDC".into_val(e),
            decimals: 7,
        },
    );
    (id.into(), token)
}

fn create_lottery_vault_contract(
    e: &Env,
    admin: &AccountId,
    token_id: &[u8; 32],
    num_winners: BigInt,
    ticket_price: BigInt,
) -> ([u8; 32], LotteryContract) {
    let id = generate_contract_id();
    register_lottery_contract(e, &id);
    let lottery_vault = LotteryContract::new(e, &id);
    lottery_vault.initialize(
        &Identifier::Account(admin.clone()),
        token_id,
        num_winners,
        ticket_price,
    );
    (id, lottery_vault)
}

#[test]
fn test() {
    let e: Env = Default::default();
    let admin1 = e.accounts().generate(); // generating the usdc admin

    let user1 = e.accounts().generate();
    let user2 = e.accounts().generate();
    let user1_id = Identifier::Account(user1.clone());
    let user2_id = Identifier::Account(user2);

    let (contract1, usdc_token) = create_token_contract(&e, &admin1); // registered and initialized the usdc token contract
    let (lottery_contract_vault, lottery_vault) = create_lottery_vault_contract(
        &e,
        &user1,
        &contract1,
        BigInt::from_u32(&e, 1),
        BigInt::from_u32(&e, 5),
    ); // registered and initialized the vault token contract, with usdc as vault token

    let lottery_vault_id = Identifier::Contract(BytesN::from_array(&e, &lottery_contract_vault)); // the id of the vault

    // minting 1000 usdc to user1
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &BigInt::from_u32(&e, 1000),
    );

    // minting 1000 usdc to user2
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user2_id,
        &BigInt::from_u32(&e, 1000),
    );

    let price = lottery_vault.get_price();
    assert_eq!(price, 5);

    // user 1 buys a lottery ticket
    usdc_token.with_source_account(&user1).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );
    lottery_vault.buy_ticket(user1_id.clone());

    assert_eq!(
        usdc_token.with_source_account(&admin1).balance(&user1_id),
        995
    );

    assert_eq!(
        usdc_token
            .with_source_account(&admin1)
            .balance(&lottery_vault_id),
        5
    );

    lottery_vault.run(user1);
    assert_eq!(
        usdc_token
            .with_source_account(&admin1)
            .balance(&lottery_vault_id),
        0
    );
    assert_eq!(
        usdc_token.with_source_account(&admin1).balance(&user1_id),
        1000
    )
}

#[test]
fn test_sequence() {
    let e: Env = Default::default();
    let admin1 = e.accounts().generate(); // generating the usdc admin

    let user1 = e.accounts().generate();
    let user2 = e.accounts().generate();
    let user3 = e.accounts().generate();
    let user1_id = Identifier::Account(user1.clone());
    let user2_id = Identifier::Account(user2.clone());
    let user3_id = Identifier::Account(user3.clone());

    let user4 = e.accounts().generate();
    let user5 = e.accounts().generate();
    let user6 = e.accounts().generate();
    let user4_id = Identifier::Account(user4.clone());
    let user5_id = Identifier::Account(user5.clone());
    let user6_id = Identifier::Account(user6.clone());

    let user7 = e.accounts().generate();
    let user8 = e.accounts().generate();
    let user9 = e.accounts().generate();
    let user7_id = Identifier::Account(user7.clone());
    let user8_id = Identifier::Account(user8.clone());
    let user9_id = Identifier::Account(user9.clone());

    let (contract1, usdc_token) = create_token_contract(&e, &admin1); // registered and initialized the usdc token contract
    let (lottery_contract_vault, lottery_vault) = create_lottery_vault_contract(
        &e,
        &user1,
        &contract1,
        BigInt::from_u32(&e, 2),
        BigInt::from_u32(&e, 5),
    ); // registered and initialized the lottery_vault token contract, with usdc as lottery_vault token

    let lottery_vault_id = Identifier::Contract(BytesN::from_array(&e, &lottery_contract_vault)); // the id of the lottery's vault

    // minting 1000 usdc to all 9 users
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user2_id,
        &BigInt::from_u32(&e, 1000),
    );

    // minting 1000 usdc to user1
    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user3_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user4_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user5_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user6_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user7_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user8_id,
        &BigInt::from_u32(&e, 1000),
    );

    usdc_token.with_source_account(&admin1).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user9_id,
        &BigInt::from_u32(&e, 1000),
    );

    // making sure the price of each ticket is 5 dollars
    let price = lottery_vault.get_price();
    assert_eq!(price, 5);

    // users (inlcuding the lottery admin, user1) buy lottery tickets by first approving the contract to spend 5 usdc and then invoking the buy_ticket contract method

    usdc_token.with_source_account(&user1).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user1_id.clone());

    usdc_token.with_source_account(&user2).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user2_id.clone());

    usdc_token.with_source_account(&user3).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user3_id.clone());

    usdc_token.with_source_account(&user4).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user4_id.clone());

    usdc_token.with_source_account(&user5).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user5_id.clone());

    usdc_token.with_source_account(&user6).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user6_id.clone());

    usdc_token.with_source_account(&user7).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user7_id.clone());

    usdc_token.with_source_account(&user8).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user8_id.clone());

    usdc_token.with_source_account(&user9).approve(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &lottery_vault_id,
        &BigInt::from_u32(&e, 5),
    );

    lottery_vault.buy_ticket(user9_id.clone());

    // running the lottery, i.e sorting winners and rewarding them
    lottery_vault.run(user1);

    // print logs if you use the log! macro in the contract code
    let logs = e.logger().all();
    std::println!("{}", logs.join("\n"));

    std::println!(
        "
BALANCES
user1: {}
user2: {}
user3: {} 
user4: {} 
user5: {}
user6: {} 
user7: {}
user7: {}
user9: {}
",
        usdc_token.with_source_account(&admin1).balance(&user1_id),
        usdc_token.with_source_account(&admin1).balance(&user2_id),
        usdc_token.with_source_account(&admin1).balance(&user3_id),
        usdc_token.with_source_account(&admin1).balance(&user4_id),
        usdc_token.with_source_account(&admin1).balance(&user5_id),
        usdc_token.with_source_account(&admin1).balance(&user6_id),
        usdc_token.with_source_account(&admin1).balance(&user7_id),
        usdc_token.with_source_account(&admin1).balance(&user8_id),
        usdc_token.with_source_account(&admin1).balance(&user9_id),
    )
}
