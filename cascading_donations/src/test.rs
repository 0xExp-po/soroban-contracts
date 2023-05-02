#![cfg(test)]

use super::{CascadingDonationContract, CascadingDonationContractClient, Identifier, Recipient};
use soroban_sdk::{symbol, vec, Env, testutils::{Accounts}, BigInt, IntoVal, BytesN, Vec};
use soroban_auth::{Signature};


use crate::token::{self, TokenMetadata};

extern crate std;

fn create_and_init_token_contract(env: &Env, admin_id: &Identifier) -> (BytesN<32>, token::Client) {
    let token_id = env.register_contract_token(None);
    let token_client = token::Client::new(&env, &token_id);

    token_client.init(
        &admin_id,
        &TokenMetadata {
            name: "USD Coin".into_val(&env),
            symbol: "USDC".into_val(&env),
            decimals: 7,
        },
    );

    (token_id, token_client)
}

#[test]
fn basic_donation_without_cascade() {
    /*
        [EXAMPLE]
        MAIN PROJECT
        |-   dependency_1
        |-   dependency_2

        Expected workflow
        1. Donate 1000 to MAIN_PROJECT
        2. The MAIN PROJECT receives the donation
        3. Auto donate to dependencie_1 with 10 percentege with a xfer
        4. Auto donate to dependencie_2 with 30 percentege with a xfer

        Expected Result = {
            MAIN_PROJECT -> 600
            dependency_1 -> 100
            dependency_2 -> 300
        }
    */

    let env = Env::default();

    let admin = env.accounts().generate();
    let admin_id = Identifier::Account(admin.clone());

    let donor = env.accounts().generate();
    let donor_id = Identifier::Account(donor.clone());

    // CONTRACT
    let contract_id = env.register_contract(None, CascadingDonationContract);
    let contract_client = CascadingDonationContractClient::new(&env, &contract_id);

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    // CHILDREN ACCOUNTS
    let dependency_1 = env.accounts().generate();
    let dependency_1_id = Identifier::Account(dependency_1.clone());

    let dependency_2 = env.accounts().generate();
    let dependency_2_id = Identifier::Account(dependency_2.clone());

    // CHILDREN RECIPIENTS
    let child_1 =
    Recipient {
            dest: soroban_sdk::Address::Account(dependency_1.clone()),
            name: symbol!("dep_1"),
            percentage: 10
        };

    let child_2 =
    Recipient {
            dest: soroban_sdk::Address::Account(dependency_2.clone()),
            name: symbol!("dep_2"),
            percentage: 30
        };

    let mut children: Vec<Recipient> = vec![&env];
    children.push_back(child_1);
    children.push_back(child_2);

    contract_client.initialize(&token_id, &children);

    // FUND DONOR ACCOUNT
    token_client.with_source_account(&admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donor_id,
        &BigInt::from_u32(&env, 2000)
    );

    token_client.with_source_account(&donor).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_u32(&env, 1000)
    );

    std::println!("==================================");
    std::println!("======= DONOR BALANCE ========: {:?}", token_client.balance(&donor_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
    std::println!("======= dependency_1 BALANCE ========: {:?}", token_client.balance(&dependency_1_id));
    std::println!("======= dependency_2 BALANCE ========: {:?}", token_client.balance(&dependency_2_id));
    std::println!("==================================");

    contract_client.with_source_account(&donor).donate(&BigInt::from_u32(&env, 1000), &donor_id);

    assert_eq!(
        token_client.balance(&Identifier::Contract(contract_id.clone())),
        &BigInt::from_u32(&env, 600),
        "Main project gets the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependency_1_id),
        &BigInt::from_u32(&env, 100),
        "Dependencie 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependency_2_id),
        &BigInt::from_u32(&env, 300),
        "Dependencie 2 receives the correct balance"
    );

}

#[test]
fn contract_with_parent_children() {
        /*
        [EXAMPLE]
        MAIN PROJECT
        |-   dependency_1
        |-   dependency_2
        |--     sub_dependency_1
        |--     sub_dependency_2

        Expected workflow
        1. Donate 1000 to MAIN PROJECT
        2. The MAIN PROJECT receives the donation
        3. Auto donate to dependency_1 with a xfer
        4. Auto donate to dependency_2 with a donation invocation (Should be a contract since is a child with sub childs)
        5. The dependency_2 receives the donation
        6. Auto donate to sub_dependency_1 with a xfer
        7. Auto donate to sub_dependency_2 with a xfer

        Expected Result = {
            MAIN_PROJECT -> 600
            dependency_1 -> 200
            CHILD PARENT (dependency_2) -> 120
            sub_dependency_1 -> 40
            sub_dependency_2 -> 40
        }
    */

    let env = Env::default();

    // USERS
    let admin = env.accounts().generate();
    let admin_id = Identifier::Account(admin.clone());

    let donor = env.accounts().generate();
    let donor_id = Identifier::Account(donor.clone());

    // PARENT CONTRACT (PARENT PROJECT)
    let contract_id = env.register_contract(None, CascadingDonationContract);
    let contract_client = CascadingDonationContractClient::new(&env, &contract_id);

    // PARENT PROJECT CHILDREN ACCOUNTS
    let dependency_1 = env.accounts().generate();
    let dependency_1_id = Identifier::Account(dependency_1.clone());

    // PARENT CHILD CONTRACT (CHILD CONTRACT)
    let child_contract_id = env.register_contract(None, CascadingDonationContract);
    let child_contract_client = CascadingDonationContractClient::new(&env, &child_contract_id);

    // SUB PROJECT 2 CHILDREN
    let sub_dependency_1 = env.accounts().generate();
    let sub_dependency_1_id = Identifier::Account(sub_dependency_1.clone());

    let sub_dependency_2 = env.accounts().generate();
    let sub_dependency_2_id = Identifier::Account(sub_dependency_2.clone());

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    // CHILD PARENT CHILDREN
    let mut parent1_children: Vec<Recipient> = vec![&env];
    let parent_1_child_1 =
        Recipient {
            dest: soroban_sdk::Address::Account(sub_dependency_1.clone()),
            name: symbol!("subdep_1"),
            percentage: 20
        };

    let parent_1_child_2 =
        Recipient {
            dest: soroban_sdk::Address::Account(sub_dependency_2.clone()),
            name: symbol!("subdep_2"),
            percentage: 20
        };

    parent1_children.push_back(parent_1_child_1);
    parent1_children.push_back(parent_1_child_2);
    // END CHILD PARENT CHILDREN

    child_contract_client.initialize(&token_id, &parent1_children);
    std::println!("======= CHILD CONTRACT CHILDREN ========: {:?}", child_contract_client.g_children());
    std::println!("========================================:");

    //PARENT CHILDREN
    let child_parent_1 =
        Recipient {
            dest: soroban_sdk::Address::Contract(child_contract_id.clone()),
            name: symbol!("c_parent_1"),
            percentage: 20
        };

    let child_1 =
        Recipient {
            dest: soroban_sdk::Address::Account(dependency_1.clone()),
            name: symbol!("dep_1"),
            percentage: 20
        };
    // END CHILDREN

    let mut children: Vec<Recipient> = vec![&env];
    children.push_back(child_1);
    children.push_back(child_parent_1);

    contract_client.initialize(&token_id, &children);
    std::println!("======= MAIN CONTRACT CHILDREN ========: {:?}", contract_client.g_children());

    // FUND DONOR ACCOUNT
    token_client.with_source_account(&admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donor_id,
        &BigInt::from_u32(&env, 2000)
    );
    
    std::println!("==================================");
    std::println!("======= DONOR BALANCE ========: {:?}", token_client.balance(&donor_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
    std::println!("======= dependency_1 BALANCE ========: {:?}", token_client.balance(&dependency_1_id));
    std::println!("======= CHILD PARENT CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(child_contract_id.clone())));
    std::println!("======= sub_dependency_1 BALANCE ========: {:?}", token_client.balance(&sub_dependency_1_id));
    std::println!("======= sub_dependency_2 BALANCE ========: {:?}", token_client.balance(&sub_dependency_2_id));
    std::println!("==================================");

    token_client.with_source_account(&donor).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_u32(&env, 1000)
    );

    contract_client.with_source_account(&donor).donate(&BigInt::from_u32(&env, 1000), &donor_id);

    assert_eq!(
        token_client.balance(&Identifier::Contract(contract_id.clone())),
        &BigInt::from_u32(&env, 600),
        "Main project gets the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependency_1_id),
        &BigInt::from_u32(&env, 200),
        "Dependency 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&Identifier::Contract(child_contract_id.clone())),
        &BigInt::from_u32(&env, 120),
        "Parent Child receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&sub_dependency_1_id),
        &BigInt::from_u32(&env, 40),
        "Sub Dependency 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&sub_dependency_2_id),
        &BigInt::from_u32(&env, 40),
        "Sub Dependency 2 receives the correct balance"
    );
}

#[test]
#[should_panic(expected = "Circular cascading aren't allowed, verify the children from")]
fn contract_with_parent_children_and_circular_schema() {
    /*
    [EXAMPLE]
    MAIN PROJECT
    |-   dependency_1
    |-   dependency_2
    |--     sub_dependency_1
    |--     sub_dependency_2

    Expected workflow
    1. Donate 1000 to MAIN PROJECT
    2. The MAIN PROJECT receives the donation
    3. Auto donate to dependency_1 with a xfer
    4. Auto donate to dependency_2 with a donation invocation (Should be a contract since is a child with sub childs)
    5. The dependency_2 receives the donation
    6. Auto donate to sub_dependency_1 fails because it points to a parent recipient.
*/

let env = Env::default();

// USERS
let admin = env.accounts().generate();
let admin_id = Identifier::Account(admin.clone());

let donor = env.accounts().generate();
let donor_id = Identifier::Account(donor.clone());

// PARENT CONTRACT (PARENT PROJECT)
let contract_id = env.register_contract(None, CascadingDonationContract);
let contract_client = CascadingDonationContractClient::new(&env, &contract_id);

// PARENT PROJECT CHILDREN ACCOUNTS
let dependency_1 = env.accounts().generate();

// PARENT CHILD CONTRACT (CHILD CONTRACT)
let child_contract_id = env.register_contract(None, CascadingDonationContract);
let child_contract_client = CascadingDonationContractClient::new(&env, &child_contract_id);

// SUB PROJECT 2 CHILDREN
let sub_dependency_2 = env.accounts().generate();

// CREATE TOKEN CONTRACT
let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

// CHILD PARENT CHILDREN
let mut parent1_children: Vec<Recipient> = vec![&env];
let parent_1_child_1 =
    Recipient {
        dest: soroban_sdk::Address::Contract(child_contract_id.clone()), // Here one of the child recipients pints to the main one (main contract).
        name: symbol!("subdep_1"),
        percentage: 20
    };

let parent_1_child_2 =
    Recipient {
        dest: soroban_sdk::Address::Account(sub_dependency_2.clone()),
        name: symbol!("subdep_2"),
        percentage: 20
    };

parent1_children.push_back(parent_1_child_1);
parent1_children.push_back(parent_1_child_2);
// END CHILD PARENT CHILDREN

child_contract_client.initialize(&token_id, &parent1_children);

//PARENT CHILDREN
let child_parent_1 =
    Recipient {
        dest: soroban_sdk::Address::Contract(child_contract_id.clone()),
        name: symbol!("c_parent_1"),
        percentage: 20
    };

let child_1 =
    Recipient {
        dest: soroban_sdk::Address::Account(dependency_1.clone()),
        name: symbol!("dep_1"),
        percentage: 20
    };
// END CHILDREN

let mut children: Vec<Recipient> = vec![&env];
children.push_back(child_1);
children.push_back(child_parent_1);

contract_client.initialize(&token_id, &children);

// FUND DONOR ACCOUNT
token_client.with_source_account(&admin).mint(
    &Signature::Invoker,
    &BigInt::zero(&env),
    &donor_id,
    &BigInt::from_u32(&env, 2000)
);

token_client.with_source_account(&donor).approve(
    &Signature::Invoker,
    &BigInt::zero(&env),
    &Identifier::Contract(contract_id.clone()),
    &BigInt::from_u32(&env, 1000)
);

contract_client.with_source_account(&donor).donate(&BigInt::from_u32(&env, 1000), &donor_id);
}

