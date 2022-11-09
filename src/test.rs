#![cfg(test)]

use super::{ReturnFundsContract, ReturnFundsContractClient, Identifier};

use soroban_sdk::{symbol, vec, Env, testutils::{Accounts, Logger}, BigInt, IntoVal};
use soroban_auth::{Signature};

extern crate std;

use crate::token::{self, TokenMetadata, Client as TokenClient,};

#[test]

fn test() {
    let env = Env::default();

    // USERS
    let user_1 = env.accounts().generate();
    let user_2 = env.accounts().generate();
    let spender = env.accounts().generate();
    let admin = env.accounts().generate();

    // IDENTIFIERS
    let user1_id = Identifier::Account(user_1.clone());
    let user2_id = Identifier::Account(user_2.clone());
    let spender_id = Identifier::Account(spender.clone());
    let admin_id = Identifier::Account(admin.clone());

    // CREATE TOKEN CONTRACT
    let token_id = env.register_contract_token(None);
    let token_client = TokenClient::new(&env, &token_id);
    let token_id_identifier = Identifier::Contract(token_id.clone());

    token_client.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "name".into_val(&env),
            symbol: "symbol".into_val(&env),
            decimals: 7,
        },
    );

    /// CREATE OUR CUSTOM CONTRACT
    let contract_id = env.register_contract(None, ReturnFundsContract);
    let charity_contract_client = ReturnFundsContractClient::new(&env, &contract_id);

    token_client.with_source_account(&admin).mint(
        &Signature::Invoker, 
        &BigInt::zero(&env), 
        &admin_id, 
        &BigInt::from_u32(&env, 10000)
    );

    token_client.with_source_account(&admin).mint(
        &Signature::Invoker, 
        &BigInt::zero(&env), 
        &spender_id, 
        &BigInt::from_u32(&env, 5000)
    );


    let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
    std::println!("USER 1 BALANCE -> {:?}", user1_balance);

    let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
    std::println!("USER 2 BALANCE -> {:?}", user2_balance);

    let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
    std::println!("SPENDER BALANCE -> {:?}", spender_balance);

    let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
    std::println!("ADMIN BALANCE -> {:?}", admin_balance);

    let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
    std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
    std::println!("===========================");

    token_client.with_source_account(&admin).xfer(
        &Signature::Invoker, 
        &BigInt::zero(&env), 
        &user2_id, 
        &BigInt::from_u32(&env, 1000)
    );

    let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
    std::println!("USER 1 BALANCE -> {:?}", user1_balance);

    let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
    std::println!("USER 2 BALANCE -> {:?}", user2_balance);

    let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
    std::println!("SPENDER BALANCE -> {:?}", spender_balance);

    let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
    std::println!("ADMIN BALANCE -> {:?}", admin_balance);

    let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
    std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
    std::println!("===========================");

    token_client.with_source_account(&spender).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &user2_id,
        &BigInt::from_u32(&env, 100)
    );
    
    token_client.with_source_account(&user_2).xfer_from(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &spender_id,
        &user1_id,
        &BigInt::from_u32(&env, 100),
    );

    let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
    std::println!("USER 1 BALANCE -> {:?}", user1_balance);

    let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
    std::println!("USER 2 BALANCE -> {:?}", user2_balance);

    let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
    std::println!("SPENDER BALANCE -> {:?}", spender_balance);

    let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
    std::println!("ADMIN BALANCE -> {:?}", admin_balance);

    let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
    std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
    std::println!("===========================");
    
    
}