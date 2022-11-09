#![cfg(test)]

use super::{ReturnFundsContract, ReturnFundsContractClient, Identifier};

use soroban_sdk::{symbol, vec, Env, testutils::{Accounts, Logger}, BigInt};
use soroban_auth::{Signature};

extern crate std;

use crate::token::{self, TokenMetadata};

#[test]

fn test() {
    let env = Env::default();
}