#![cfg(any(test, feature = "testutils"))]

use crate::{token::Signature, LotteryContractClient};
use soroban_auth::Identifier;

use soroban_sdk::{AccountId, BigInt, BytesN, Env};

pub fn register_test_contract(e: &Env, contract_id: &[u8; 32]) {
    let contract_id = BytesN::from_array(e, contract_id);
    e.register_contract(&contract_id, crate::LotteryContract {});
}

pub struct LotteryContract {
    env: Env,
    contract_id: BytesN<32>,
}

impl LotteryContract {
    fn client(&self) -> LotteryContractClient {
        LotteryContractClient::new(&self.env, &self.contract_id)
    }

    pub fn new(env: &Env, contract_id: &[u8; 32]) -> Self {
        Self {
            env: env.clone(),
            contract_id: BytesN::from_array(env, contract_id),
        }
    }

    pub fn initialize(
        &self,
        admin: &Identifier,
        token_id: &[u8; 32],
        num_winners: BigInt,
        ticket_price: BigInt,
    ) {
        self.client().initialize(
            admin,
            &BytesN::from_array(&self.env, token_id),
            &num_winners,
            &ticket_price,
        );
    }

    pub fn nonce(&self) -> BigInt {
        self.client().nonce()
    }

    pub fn run(&self, admin: AccountId) {
        self.env.set_source_account(&admin);
        self.client().run(&crate::Auth {
            sig: Signature::Invoker,
            nonce: BigInt::zero(&self.env),
        });
    }

    pub fn buy_ticket(&self, from: Identifier) {
        self.client().buy_ticket(&from)
    }

    pub fn get_price(&self) -> BigInt {
        self.client().get_price()
    }
}
