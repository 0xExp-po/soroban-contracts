#![no_std]
#![allow(warnings)]

use soroban_sdk::{contractimpl, contracttype, log, symbol, vec, Env, IntoVal, Symbol, Vec, BytesN, AccountId, BigInt, RawVal};

use soroban_auth::{Identifier, Signature};

use crate::token::{TokenMetadata};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

const ORG_NAME: Symbol = symbol!("ORG_NAME");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenId,
    AdminId,
    Reward,
    Members
}

pub struct OrganizationContract;

pub trait OrganizationContractTrait {
    fn initialize(
        e: Env,
        admin: Identifier,
        org_name: Symbol
    ) -> BytesN<32>;

    fn add_m(env: Env, account: AccountId);

    fn remove_m(env: Env, from: AccountId);

    fn reward_m(e: Env, token_approval_sig: Signature, to: AccountId);

    fn get_tc_id(env: Env) -> BytesN<32>;

    fn get_bal(env: Env) -> BigInt;
    
    fn get_m(env: Env) -> Vec<AccountId>;

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

        set_reward_value(&env, BigInt::from_i32(&env, 35));

        token_id
    }

    fn add_m(env: Env, account: AccountId) {
        add_member(&env, account);
    }
    
    fn remove_m(env: Env, from: AccountId) {
        remove_member(&env, &from);
    }

    fn reward_m(env: Env, approval_sign: Signature, to: AccountId) {
        reward_member(&env, &approval_sign, &to);
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

    fn get_m(env: Env) -> Vec<AccountId> {
        get_members(&env)
    }
}

// VALIDATIONS
fn is_member(env: &Env, to: &AccountId) -> bool {
    let mut members: Vec<AccountId> = get_members(&env);

    members.contains(to)
} 

// ORGANIZATION
fn add_member(env: &Env, account: AccountId) {
    let mut members = get_members(&env);

    members.push_back(account);

    let key = DataKey::Members;
    env.data().set(key, members);
}

fn remove_member(env: &Env, from: &AccountId) {
    // Remove member from the members vector
    let mut members: Vec<AccountId> = get_members(&env);
    
    let index;

    match members.first_index_of(from) {
        Some(i) => index = i,
        None => panic!("You are trying to remove an account that doesn't belong to your organization"),
    }

    members.remove(index);

    let key = DataKey::Members;
    env.data().set(key, members);

    // Bring back it's TOKEN's to the admin
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

fn reward_member(env: &Env, approval_sign: &Signature, to: &AccountId) {
    // Validate "to" is a member of the contract
    if !is_member(&env, &to) {
        panic!("The user account doesn't belong to the organization");
    }
    transfer(&env, &approval_sign, &get_account_identifier(to.clone()), &get_reward_value(&env));
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
fn set_reward_value(env: &Env, new_value: BigInt) {
    env.data().set(DataKey::Reward, new_value);
}

fn get_reward_value(env: &Env) -> BigInt {
    let key = DataKey::Reward;
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
