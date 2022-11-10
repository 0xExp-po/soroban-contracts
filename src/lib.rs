#![no_std]
#![allow(warnings)]

use soroban_sdk::{contractimpl, contracttype, log, symbol, vec, Env, IntoVal, Symbol, Vec, BytesN, AccountId, BigInt, RawVal};

use soroban_auth::{Identifier, Signature};

use crate::token::{TokenMetadata};

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

const ORG_NAME: Symbol = symbol!("ORG_NAME");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    AdminId,
    ThankVal,
    CongratVal,
    Members
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Member {
    name: Symbol,
    account: AccountId
}

pub struct OrganizationContract;

pub trait OrganizationContractTrait {
    // Sets the admin and the vault's token id
    fn initialize(
        e: Env,
        admin: Identifier,
        org_name: Symbol
    ) -> BytesN<32>;

    // Add a member to the contract
    fn add_m(env: Env, name: Symbol, account: AccountId);

    // Remove a kommitter from the contract and get backs its MTK balance
    fn remove_m(env: Env, from: AccountId);

    // Send thanks to a kommitter
    fn thank_m(e: Env, token_approval_sig: Signature, to: Identifier);

    // Send congratz to a kommitter
    fn congrat_m(env: Env, approval_sign: Signature, to: Identifier);

    fn get_tc_id(env: Env) -> BytesN<32>;

    fn get_bal(env: Env) -> BigInt;
    
    fn get_m(env: Env) -> Vec<Member>;

    fn org_name(env: Env) -> Symbol;
    
    // Add the initial balance to the contract
    fn fund_c(env: Env, approval_sign: Signature);
}

#[contractimpl]
impl OrganizationContractTrait for OrganizationContract {
    fn initialize(
        env: Env, 
        admin: Identifier,
        org_name: Symbol
    ) -> BytesN<32> {
        set_admin_id(&env, &admin);
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

        set_organization_name(&env, org_name);

        set_token_id(&env, &token_id);

        set_thank_value(&env, BigInt::from_i32(&env, 35));

        set_congrat_value(&env, BigInt::from_i32(&env, 25));

        token_id
    }

    fn add_m(env: Env, name: Symbol, account: AccountId) {
        add_member(&env, name, account);
    }
    
    fn remove_m(env: Env, from: AccountId) {
        remove_member(&env, &from);
    }

    fn thank_m(env: Env, approval_sign: Signature, to: Identifier) {
        thank_member(&env, &approval_sign, &to);
    }
    
    fn congrat_m(env: Env, approval_sign: Signature, to: Identifier) {
        congrat_member(&env, &approval_sign, &to);
    }

    fn get_tc_id(env: Env) -> BytesN<32> {
        get_token_contract_id(&env)
    }

    fn get_bal(env: Env) -> BigInt {
        get_contract_balance(&env)
    }

    fn org_name(env: Env) -> Symbol {
        get_organization_name(&env)
    }

    fn fund_c(env: Env, approval_sign: Signature) {
        fund_contract_balance(&env, &approval_sign);
    }

    fn get_m(env: Env) -> Vec<Member> {
        get_members(&env)
    }
}

// ORGANIZATION
fn add_member(env: &Env, name: Symbol, account: AccountId) {
    let member = Member {
        name: name,
        account: account
    };

    let mut members = get_members(&env);
    members.push_back(member);

    let key = DataKey::Members;
    env.data().set(key, members);
}

fn remove_member(env: &Env, from: &AccountId) {
    // Remove kommitter from the kommitters vector
    // Bring back it's MKT's to the contract balance
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, &tc_id);

    let admin_id = get_admin_id(&env);
    let from_identifier = get_account_identifier(from.clone());
    let kommitter_balance = client.balance(&from_identifier);

    client.xfer_from(
        &Signature::Invoker, 
        &BigInt::zero(&env), 
        &from_identifier, 
        &admin_id,
        &kommitter_balance
    );
}

fn get_members<T: soroban_sdk::TryFromVal<Env, RawVal> + soroban_sdk::IntoVal<Env, RawVal>>(
    e: &Env,
) -> Vec<T> {
    let key = DataKey::Members;
    e.data()
        .get(key)
        .unwrap_or(Ok(vec![e])) // if no members on vector
        .unwrap()
}

fn fund_contract_balance(env: &Env, approval_sign: &Signature) {
    let token_id = get_token_contract_id(&env);
    let admin_id = get_admin_id(&env);
    let token_client = token::Client::new(&env, &token_id);
    
    let nonce = token_client.nonce(&admin_id);
    token_client.mint(&approval_sign, &nonce, &admin_id, &BigInt::from_u32(&env, 10000));
}

fn congrat_member(env: &Env, approval_sign: &Signature, to: &Identifier) {
    // Validate "to" is a member of the contract
    transfer(&env, &approval_sign, &to, &get_congrat_value(&env));
}

fn thank_member(env: &Env, approval_sign: &Signature, to: &Identifier) {
    // Validate "to" is a member of the contract
    transfer(&env, &approval_sign, &to, &get_thank_value(&env));
}

fn transfer(env: &Env, approval_sign: &Signature, to: &Identifier, amount: &BigInt) {
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, tc_id);

    let admin_id = get_admin_id(&env);
    let nonce = client.nonce(&admin_id);

    client.xfer(&approval_sign, &nonce, &to, &amount);
}

fn get_contract_balance(env: &Env) -> BigInt {
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, tc_id);

    let admin_id = get_admin_id(&env);

    client.balance(&admin_id)
}
fn set_organization_name(env: &Env, new_value: Symbol) {
    env.data().set(ORG_NAME, new_value);
}

fn get_organization_name(env: &Env) -> Symbol {
    env.data().get(ORG_NAME).unwrap().unwrap()
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

fn set_admin_id(env: &Env, account_id: &Identifier) {
    env.data().set(DataKey::AdminId, account_id);
}

// TOKEN CONTRACT
fn get_token_contract_id(env: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    env.data().get(key).unwrap().unwrap()
}

fn set_token_id(e: &Env, token_id: &BytesN<32>) {
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
