#![no_std]

#[cfg(feature = "testutils")]
extern crate std;

mod test;
pub mod testutils;

use soroban_auth::{Identifier, Signature};

#[allow(unused_imports)]
use soroban_sdk::{contractimpl, contracttype, log, vec, BigInt, BytesN, Env, RawVal, Vec}; // keeping the log import for future debugging work

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    Admin,
    Candidates,
    NumWinners,
    Ticket,
    Nonce(Identifier),
}

#[derive(Clone)]
#[contracttype]
pub struct Auth {
    pub sig: Signature,
    pub nonce: BigInt,
}

fn get_random(max: u32) -> u32 {
    fastrand::u32(..max)
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

fn put_candidate(e: &Env, candidate: Identifier) {
    let mut candidates = get_candidates(e);
    candidates.push_back(candidate);

    let key = DataKey::Candidates;
    e.data().set(key, candidates);
}

fn get_candidates<T: soroban_sdk::TryFromVal<Env, RawVal> + soroban_sdk::IntoVal<Env, RawVal>>(
    e: &Env,
) -> Vec<T> {
    let key = DataKey::Candidates;
    e.data()
        .get(key)
        .unwrap_or(Ok(vec![e])) // if no candidates participated
        .unwrap()
}

fn put_num_winners(e: &Env, num_winners: BigInt) {
    let key = DataKey::NumWinners;
    e.data().set(key, num_winners);
}

fn get_num_winners(e: &Env) -> BigInt {
    let key = DataKey::NumWinners;
    e.data().get(key).unwrap_or(Ok(BigInt::zero(e))).unwrap()
}

fn put_ticket_price(e: &Env, ticket_price: BigInt) {
    let key = DataKey::Ticket;
    e.data().set(key, ticket_price);
}

fn get_ticket_price(e: &Env) -> BigInt {
    let key = DataKey::Ticket;
    e.data().get(key).unwrap().unwrap()
}

fn put_token_id(e: &Env, token_id: BytesN<32>) {
    let key = DataKey::TokenId;
    e.data().set(key, token_id);
}

fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    e.data().get(key).unwrap().unwrap()
}

fn get_token_balance(e: &Env) -> BigInt {
    let contract_id = get_token_id(e);
    token::Client::new(e, contract_id).balance(&get_contract_id(e))
}

fn transfer(e: &Env, to: Identifier, amount: &BigInt) {
    let client = token::Client::new(e, get_token_id(e));
    client.xfer(
        &Signature::Invoker,
        &client.nonce(&Signature::Invoker.identifier(e)),
        &to,
        amount,
    );
}

fn transfer_in_vault(e: &Env, from: Identifier, amount: BigInt) {
    let client = token::Client::new(e, get_token_id(e));
    let vault_id = get_contract_id(e);

    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &from,
        &vault_id,
        &amount,
    )
}

fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.data().has(key)
}

fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.data().get_unchecked(key).unwrap()
}

fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.data().set(key, id);
}

pub fn check_admin(e: &Env, auth: &Signature) {
    let auth_id = auth.identifier(&e);
    if auth_id != read_administrator(&e) {
        panic!("not authorized by admin")
    }
}

fn read_nonce(e: &Env, id: &Identifier) -> BigInt {
    let key = DataKey::Nonce(id.clone());
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

fn verify_and_consume_nonce(e: &Env, auth: &Signature, expected_nonce: &BigInt) {
    match auth {
        Signature::Invoker => {
            if BigInt::zero(&e) != expected_nonce {
                panic!("nonce should be zero for Invoker")
            }
            return;
        }
        _ => {}
    }

    let id = auth.identifier(&e);
    let key = DataKey::Nonce(id.clone());
    let nonce = read_nonce(e, &id);

    if nonce != expected_nonce {
        panic!("incorrect nonce")
    }
    e.data().set(key, &nonce + 1);
}

pub trait LotteryContractTrait {
    // Sets the admin and the vault's token id
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        num_winners: BigInt,
        ticket_price: BigInt,
    );

    // Returns the nonce for the admin
    fn nonce(e: Env) -> BigInt;

    // deposit shares into the vault: mints the vault shares to "from"
    fn buy_ticket(e: Env, from: Identifier);

    // run the lottery
    fn run(e: Env, admin_auth: Auth);

    // get vault shares for a user
    fn get_price(e: Env) -> BigInt;
}

pub struct LotteryContract;

#[contractimpl]
impl LotteryContractTrait for LotteryContract {
    fn initialize(
        e: Env,
        admin: Identifier,
        token_id: BytesN<32>,
        num_winners: BigInt,
        ticket_price: BigInt,
    ) {
        if has_administrator(&e) {
            panic!("admin is already set");
        }

        write_administrator(&e, admin);

        put_token_id(&e, token_id);
        put_num_winners(&e, num_winners);
        put_ticket_price(&e, ticket_price);
    }

    fn nonce(e: Env) -> BigInt {
        read_nonce(&e, &read_administrator(&e))
    }

    fn buy_ticket(e: Env, from: Identifier) {
        let ticket_price = get_ticket_price(&e);
        transfer_in_vault(&e, from.clone(), ticket_price);
        put_candidate(&e, from);
    }

    fn get_price(e: Env) -> BigInt {
        get_ticket_price(&e)
    }

    fn run(e: Env, admin_auth: Auth) {
        let admin_id = admin_auth.sig.identifier(&e);

        check_admin(&e, &admin_auth.sig);
        verify_and_consume_nonce(&e, &admin_auth.sig, &admin_auth.nonce);

        let candidates: Vec<Identifier> = get_candidates(&e);
        let max = candidates.len();

        let num_winners = get_num_winners(&e);
        let balance = get_token_balance(&e);
        let payout = balance / num_winners.clone(); // this will leave out a remainder to be redeemed by the admin as fees

        let mut winners_idx: Vec<u32> = Vec::new(&e);
        for _ in 0..num_winners.to_u32() {
            let rand = get_random(max);

            winners_idx.push_back(rand); // one candidate might be winning more than once
        }

        for idx in winners_idx {
            let id = candidates.get_unchecked(idx.unwrap()).unwrap();
            transfer(&e, id, &payout)
        }

        let updated_bal = get_token_balance(&e);
        transfer(&e, admin_id, &updated_bal) // transfering fees to the admin
    }
}
