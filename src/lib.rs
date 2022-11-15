#![no_std]

use soroban_sdk::{contractimpl, contracttype, vec, Env, Symbol, Vec, BytesN, AccountId, BigInt, RawVal, Map};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    OrgName,
    TokenId,
    AdminId,
    Rewards,
    Members,
    AllowedF
}

// VALIDATIONS
fn is_member(env: &Env, to: &AccountId) -> bool {
    let members: Vec<AccountId> = get_members(&env);

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
    let member_balance = client.balance(&from_identifier);

    client.xfer_from(
        &Signature::Invoker, 
        &BigInt::zero(&env), 
        &from_identifier, 
        &admin_id,
        &member_balance
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
    token_client.mint(&approval_sign, &nonce, &admin_id, &get_allowed_funds_to_issue(&env));
}

fn reward_member(env: &Env, approval_sign: &Signature, to: &AccountId, reward_type: &Symbol) {
    if !is_member(&env, &to) {
        panic!("The user account you're trying to reward doesn't belong to the organization");
    }

    if !is_reward_valid(&env, &reward_type) {
        panic!("The reward type you are trying to use isn't supported")
    }

    let reward_value = get_reward_by_type(&env, &reward_type);
    transfer(&env, &approval_sign, &get_account_identifier(to.clone()), &BigInt::from_u32(&env, reward_value));
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
    env.data().set(DataKey::OrgName, new_value);
}

fn get_organization_name(env: &Env) -> Symbol {
    env.data().get(DataKey::OrgName).unwrap().unwrap()
}

// FUNDS ALLOWED TO ISSUE
fn set_allowed_funds_to_issue(env: &Env, new_value: BigInt) {
    env.data().set(DataKey::AllowedF, new_value);
}

fn get_allowed_funds_to_issue(env: &Env) -> BigInt {
    env.data().get(DataKey::AllowedF).unwrap().unwrap()
}

// REWARDS
fn is_reward_valid(env: &Env, key: &Symbol) -> bool {
    let rewards = get_rewards(&env);

    rewards.contains_key(key.clone())
}

fn set_rewards(env: &Env, reward_types: &Map<Symbol, u32>) {
    env.data().set(DataKey::Rewards, reward_types);
}

fn get_rewards(env: &Env) -> Map<Symbol, u32> {
    let key = DataKey::Rewards;
    env.data().get(key).unwrap().unwrap()
}

fn get_reward_by_type(env: &Env, r_type: &Symbol) -> u32 {
    let key = DataKey::Rewards;
    let rewards: Map<Symbol, u32> = env.data().get(key).unwrap().unwrap();

    rewards.get(r_type.clone()).unwrap().unwrap()
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

pub struct OrganizationContract;

pub trait OrganizationContractTrait {
    fn initialize(
        e: Env,
        admin: Identifier,
        org_name: Symbol,
        rewards: Map<Symbol, u32>,
        fund_amount: u32,
        token_c_id:BytesN<32>
    );

    fn add_m(env: Env, account: AccountId);

    fn remove_m(env: Env, from: AccountId);

    fn reward_m(e: Env, token_approval_sig: Signature, to: AccountId, r_type: Symbol);

    fn get_tc_id(env: Env) -> BytesN<32>;

    fn get_bal(env: Env) -> BigInt;
    
    fn get_m(env: Env) -> Vec<AccountId>;

    fn org_name(env: Env) -> Symbol;
    
    fn fund_c(env: Env, approval_sign: Signature);
}

#[contractimpl]
impl OrganizationContractTrait for OrganizationContract {
    fn initialize(
        env: Env, 
        admin: Identifier,
        org_name: Symbol,
        rewards: Map<Symbol, u32>,
        fund_amount: u32,
        token_c_id: BytesN<32>
    ) {
        set_admin_id(&env, &admin);
        
        set_organization_name(&env, org_name);

        set_allowed_funds_to_issue(&env, BigInt::from_u32(&env, fund_amount));

        set_token_id(&env, &token_c_id);

        set_rewards(&env, &rewards);
    }

    fn add_m(env: Env, account: AccountId) {
        add_member(&env, account);
    }
    
    fn remove_m(env: Env, from: AccountId) {
        remove_member(&env, &from);
    }

    fn reward_m(env: Env, approval_sign: Signature, to: AccountId, r_type: Symbol) {
        reward_member(&env, &approval_sign, &to, &r_type);
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

#[cfg(test)]
mod test;
