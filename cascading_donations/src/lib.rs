#![no_std]

use soroban_sdk::{contractimpl, contracttype, symbol, vec, Env, BigInt, BytesN, Vec, Symbol, Address, RawVal};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Name,
    TkContract,
    ChildRecip // Vec<Recipient>
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Recipient {
    name: Symbol,
    dest: Address,
    percentage: u32,
}

// CHILDREN
fn get_children(env: &Env) -> Vec<Recipient> {
    let key = DataKey::ChildRecip;
    env.data().get(key).unwrap().unwrap()
}

fn set_children(env: &Env, new_children: &Vec<Recipient>) {
    env.data().set(DataKey::ChildRecip, new_children);
}

// TOKEN CONTRACT
fn get_token_contract_id(env: &Env) -> BytesN<32> {
    let key = DataKey::TkContract;
    env.data().get(key).unwrap().unwrap()
}

fn set_token_contract_id(e: &Env, token_id: &BytesN<32>) {
    e.data().set(DataKey::TkContract, token_id);
}

fn donate_to_child(env: &Env, child_address: &Identifier, percentage: &u32, base_balance: &BigInt) {
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, &tc_id);

    let amount: BigInt = (base_balance * percentage) / 100;
    
    client.xfer(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &child_address,
        &amount
    );
}

fn donate_to_parent_child(env: &Env, recipient_contract_id: &BytesN<32>, percentage: &u32, base_balance: &BigInt, parents: &Vec<Address>) {
    donate_to_child(&env, &Identifier::Contract(recipient_contract_id.clone()), &percentage, &base_balance);
    
    let args: Vec<RawVal> = vec![
        &env,
        parents.to_raw()
    ];

    env.invoke_contract(&recipient_contract_id, &symbol!("donate_ch"), args)
}

fn apply_donation_type(env: &Env, child: &Recipient, base_balance: &BigInt, parents: &Vec<Address>) {
    let destination = child.dest.clone();

    if parents.contains(destination) {
        panic!("Circular cascading aren't allowed, verify the children from {:?}", child.name);
    }

    match &child.dest {
        Address::Contract(contract_id) => donate_to_parent_child(&env, &contract_id, &child.percentage, &base_balance, &parents),
        Address::Account(account_id) =>  donate_to_child(&env, &Identifier::Account(account_id.clone()), &child.percentage, &base_balance),
    }
}

fn apply_children_donations(env: &Env, base_balance: &BigInt, parents: &Vec<Address>) {
    for child in get_children(&env) {
        match child {
            Ok(recipient) => apply_donation_type(env, &recipient, &base_balance, &parents),
            Err(error) => panic!("Problem reading the recipient: {:?}", error),
        }
    }
}

fn apply_main_donation(env: &Env, donor: &Identifier, amount: &BigInt) {
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, &tc_id);

    let contract = env.current_contract();
    let contract_identifier = Identifier::Contract(contract.clone());

    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donor,
        &contract_identifier,
        &amount
    );

    let contract_balance = client.balance(&contract_identifier);
    let contract_address = Address::Contract(contract);

    let parents: Vec<Address> = vec![
        &env,
        contract_address
    ];

    apply_children_donations(&env, &contract_balance, &parents);
}
pub struct CascadingDonationContract;

pub trait CascadingDonationContractTrait {
    fn initialize(env: Env, tc_id: BytesN<32>, children: Vec<Recipient>);
    fn donate(env: Env, amount: BigInt, donor: Identifier);
    fn donate_ch(env: Env, parents: Vec<Address>);
    fn s_children(env: Env, new_children: Vec<Recipient>);
    fn g_children(env: Env) -> Vec<Recipient>;
}

#[contractimpl]
impl CascadingDonationContractTrait for CascadingDonationContract {
    fn initialize(env: Env, tc_id: BytesN<32>, children: Vec<Recipient>) {
        set_token_contract_id(&env, &tc_id);
        set_children(&env, &children)
    }

    fn donate(env: Env, amount: BigInt, donor: Identifier) {
        apply_main_donation(&env, &donor, &amount);
    }

    fn donate_ch(env: Env, parents: Vec<Address>) {
        let tc_id = get_token_contract_id(&env);
        let client = token::Client::new(&env, &tc_id);

        let contract = env.current_contract();
        let contract_identifier = Identifier::Contract(contract.clone());

        let contract_balance = client.balance(&contract_identifier);
        let contract_address = Address::Contract(contract);

        let mut updated_parent: Vec<Address> = parents.clone();

        updated_parent.push_back(contract_address);

        apply_children_donations(&env, &contract_balance, &updated_parent);
    }

    fn s_children(env: Env, new_children: Vec<Recipient>) {
        set_children(&env, &new_children);
    }

    fn g_children(env: Env) -> Vec<Recipient> {
        get_children(&env)
    }
}

#[cfg(test)]
mod test;
