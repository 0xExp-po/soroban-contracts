#![no_std]
#![allow(warnings)]

use soroban_sdk::{contractimpl, contracttype, log, symbol, vec, Env, IntoVal, Symbol, Vec, BytesN, AccountId, BigInt};

use soroban_auth::{Identifier, Signature};

use crate::token::{TokenMetadata};

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    AdminId
}
pub struct ReturnFundsContract;

#[contractimpl]
impl ReturnFundsContract {
    pub fn init(env: Env) {

    }
}

#[cfg(test)]
mod test;
