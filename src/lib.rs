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
    AdminId,
    ThankVal,
    CongratVal,
}

#[derive(Clone)]
#[contracttype]
pub enum Kommitter {
    Name(Symbol),
    Account(AccountId),
}

#[derive(Clone)]
#[contracttype]
pub struct Directory {
    kommitters: Vec<Kommitter>
}
pub struct ReturnFundsContract;

pub trait ReturnFundsContractTrait {
    // Sets the admin and the vault's token id
    fn initialize(
        e: Env,
        admin: Identifier
    ) -> BytesN<32>;

    // Add a kommitter as a member of the contract
    fn add_k(e: Env, kommitter: Kommitter);

    // Remove a kommitter from the contract and get backs its MTK balance
    fn remove_k(e: Env, from: Identifier);

    // Send thanks to a kommitter
    fn thank_k(e: Env, token_approval_sig: Signature, to: Identifier);

    // Send congratz to a kommitter
    fn congrat_k(e: Env, to: AccountId);

    fn get_tc_id(env: Env) -> BytesN<32>;

    fn get_bal(env: Env) -> BigInt;
    
    // Add the initial balance to the contract
    fn fund_c(env: Env, approval_sign: Signature);
}

#[contractimpl]
impl ReturnFundsContractTrait for ReturnFundsContract {
    fn initialize(
        env: Env, 
        admin: Identifier
    ) -> BytesN<32> {
        put_admin_id(&env, &admin);
        let token_id = env.register_contract_token(None);
        let token_client = token::Client::new(&env, &token_id);

        token_client.init(
            &admin,
            &TokenMetadata {
                name: "Mitkoin".into_val(&env),
                symbol: "MTK".into_val(&env),
                decimals: 7,
            },
        );

        let admin_balance = admin.clone();
        log!(&env, "======> BALANCE -> {}", token_client.balance(&admin_balance));

        put_token_id(&env, &token_id);

        // Set Thanks amount reward
        // env.data().set(DataKey::ThankVal, BigInt::from_i32(&env, 35));
        set_thank_value(&env, BigInt::from_i32(&env, 35));

        // Set Congratz amount reward
        set_congrat_value(&env, BigInt::from_i32(&env, 25));
        // env.data().set(DataKey::CongratVal, BigInt::from_i32(&env, 25));

        token_id
    }

    fn add_k(env: Env, kommitter: Kommitter) {
        // Push a kommitter into the kommitters vector
    }
    
    // Imply xfer_from
    fn remove_k(env: Env, from: Identifier) {
        // Remove kommitter from the kommitters vector
        // Bring back it's MKT's to the contract balance
        let tc_id = get_token_contract_id(&env);
        let client = token::Client::new(&env, &tc_id);

        let admin_id = get_admin_id(&env);

        let kommitter_balance = client.balance(&from);

        client.xfer_from(
            &Signature::Invoker, 
            &BigInt::zero(&env), 
            &from, 
            &admin_id,
            &kommitter_balance
        );
    }

    // Imply xfer
    fn thank_k(env: Env, approval_sign: Signature, to: Identifier) {
        // Transfer 35 MTK's to "to"
        let tc_id = get_token_contract_id(&env);
        let client = token::Client::new(&env, tc_id);

        let admin_id = get_admin_id(&env);
        let nonce = client.nonce(&admin_id);

        client.xfer(&approval_sign, &nonce, &to, &get_thank_value(&env));
    }

    // Imply xfer
    fn congrat_k(env: Env, to: AccountId) {
        // Transfer 25 MTK's to "to"
    }

    fn get_tc_id(env: Env) -> BytesN<32> {
        get_token_contract_id(&env)
    }

    fn get_bal(env: Env) -> BigInt {
        let tc_id = get_token_contract_id(&env);
        let client = token::Client::new(&env, tc_id);

        let admin_id = get_admin_id(&env);

        client.balance(&admin_id)
    }

    fn fund_c(env: Env, approval_sign: Signature) {
        let token_id = get_token_contract_id(&env);
        let admin_id = get_admin_id(&env);
        let token_client = token::Client::new(&env, &token_id);
        
        let nonce = token_client.nonce(&admin_id);
        token_client.mint(&approval_sign, &nonce, &admin_id, &BigInt::from_u32(&env, 10000));
    }
}

// REWARDS
fn set_thank_value(env: &Env, new_value: BigInt) {
    env.data().set(DataKey::ThankVal, new_value);
}

fn get_thank_value(env: &Env) -> BigInt {
    let key = DataKey::ThankVal;
    env.data().get(key).unwrap().unwrap()
}

fn set_congrat_value(env: &Env, new_value: BigInt) {
    env.data().set(DataKey::CongratVal, new_value);
}

fn get_congrat_value(env: &Env) -> BigInt {
    let key = DataKey::CongratVal;
    env.data().get(key).unwrap().unwrap()
}

// ADMIN
fn get_admin_id(env: &Env) -> Identifier {
    let key = DataKey::AdminId;
    env.data().get(key).unwrap().unwrap()
}

fn put_admin_id(env: &Env, account_id: &Identifier) {
    env.data().set(DataKey::AdminId, account_id);
}

// TOKEN CONTRACT
fn get_token_contract_id(env: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    env.data().get(key).unwrap().unwrap()
}

fn put_token_id(e: &Env, token_id: &BytesN<32>) {
    e.data().set(DataKey::TokenId, token_id);
}

// IDENTIFIER WRAPPERS
pub fn get_account_identifier(account_id: AccountId) -> Identifier {
    Identifier::Account(account_id)
}

pub fn get_contract_identifier(contract_id: BytesN<32>) -> Identifier {
    Identifier::Contract(contract_id)
}

#[cfg(test)]
mod test;
