#![cfg(test)]

use super::{OrganizationContract, OrganizationContractClient, Identifier};

use soroban_sdk::{symbol, vec, Env, testutils::{Accounts, Logger}, BigInt, IntoVal};
use soroban_auth::{Signature, testutils::ed25519};

extern crate std;

use crate::token::{self, TokenMetadata, Client as TokenClient,};

//#[test]

// fn test_xfer_and_xfer_from() {
//     let env = Env::default();

//     // USERS
//     let user_1 = env.accounts().generate();
//     let user_2 = env.accounts().generate();
//     let spender = env.accounts().generate();
//     let admin = env.accounts().generate();

//     // IDENTIFIERS
//     let user1_id = Identi::Account(user_1.clone());
//     let user2_id = Identi::Account(user_2.clone());
//     let spender_id = Identi::Account(spender.clone());
//     let admin_id = Identi::Account(admin.clone());

//     // CREATE TOKEN CONTRACT
//     let token_id = env.register_contract_token(None);
//     let token_client = TokenClient::new(&env, &token_id);
//     let token_id_identifier = Identi::Contract(token_id.clone());

//     token_client.init(
//         &admin_id,
//         &TokenMetadata {
//             name: "Mitkoin".into_val(&env),
//             symbol: "MTK".into_val(&env),
//             decimals: 7,
//         },
//     );

//     /// CREATE OUR CUSTOM CONTRACT
//     let contract_id = env.register_contract(None, ReturnFundsContract);
//     let contract_client = ReturnFundsContractClient::new(&env, &contract_id);
//     contract_client.initialize(&admin, &token_id);

//     token_client.with_source_account(&admin).mint(
//         &Signature::Invoker, 
//         &BigInt::zero(&env), 
//         &admin_id, 
//         &BigInt::from_u32(&env, 10000)
//     );

//     token_client.with_source_account(&admin).mint(
//         &Signature::Invoker, 
//         &BigInt::zero(&env), 
//         &spender_id, 
//         &BigInt::from_u32(&env, 5000)
//     );

//     let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
//     std::println!("USER 1 BALANCE -> {:?}", user1_balance);

//     let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
//     std::println!("USER 2 BALANCE -> {:?}", user2_balance);

//     let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
//     std::println!("SPENDER BALANCE -> {:?}", spender_balance);

//     let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
//     std::println!("ADMIN BALANCE -> {:?}", admin_balance);

//     let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
//     std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
//     std::println!("===========================");

//     token_client.with_source_account(&admin).xfer(
//         &Signature::Invoker, 
//         &BigInt::zero(&env), 
//         &user2_id, 
//         &BigInt::from_u32(&env, 1000)
//     );

//     let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
//     std::println!("USER 1 BALANCE -> {:?}", user1_balance);

//     let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
//     std::println!("USER 2 BALANCE -> {:?}", user2_balance);

//     let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
//     std::println!("SPENDER BALANCE -> {:?}", spender_balance);

//     let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
//     std::println!("ADMIN BALANCE -> {:?}", admin_balance);

//     let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
//     std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
//     std::println!("===========================");

//     token_client.with_source_account(&spender).approve(
//         &Signature::Invoker,
//         &BigInt::zero(&env),
//         &user2_id,
//         &BigInt::from_u32(&env, 100)
//     );
    
//     token_client.with_source_account(&user_2).xfer_from(
//         &Signature::Invoker,
//         &BigInt::zero(&env),
//         &spender_id,
//         &user1_id,
//         &BigInt::from_u32(&env, 100),
//     );

//     let user1_balance = token_client.with_source_account(&admin).balance(&user1_id);
//     std::println!("USER 1 BALANCE -> {:?}", user1_balance);

//     let user2_balance = token_client.with_source_account(&admin).balance(&user2_id);
//     std::println!("USER 2 BALANCE -> {:?}", user2_balance);

//     let spender_balance = token_client.with_source_account(&admin).balance(&spender_id);
//     std::println!("SPENDER BALANCE -> {:?}", spender_balance);

//     let admin_balance = token_client.with_source_account(&admin).balance(&admin_id);
//     std::println!("ADMIN BALANCE -> {:?}", admin_balance);

//     let contract_balance = token_client.with_source_account(&admin).balance(&token_id_identifier);
//     std::println!("CONTRACT BALANCE -> {:?}", admin_balance);
//     std::println!("===========================");
    
    
// }

#[test]
fn test_sign() {
    let env = Env::default();

    // USERS
    let (usr1_id, usr1_sign) = ed25519::generate(&env);
    let (usr2_id, usr2_sign) = ed25519::generate(&env);
    let (spender_id, spender_sign) = ed25519::generate(&env);
    let (admin_id, admin_sign) = ed25519::generate(&env);

    // APPROVAL USER
    let approval_user = env.accounts().generate();
    let approval_user_id = Identifier::Account(approval_user.clone());
    
    /// CREATE OUR CUSTOM CONTRACT
    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);
    
    let org_name = symbol!("Kommit");
    contract_client.initialize(&admin_id, &org_name);

    let token_contract_id = contract_client.get_tc_id();
    let token_client = token::Client::new(&env, &token_contract_id);

    let nonce = token_client.nonce(&admin_id);

    let approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_contract_id,
        symbol!("mint"),
        (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, 10000)),
    );
    let balance = contract_client.get_bal();

    let fetched_org_name = contract_client.org_name();
    std::println!("======= [{:?}] CONTRACT START ========:", fetched_org_name);
    std::println!("======= ADMIN BALANCE START ========: {}", balance);
    std::println!("======= CONTRACT BALANCE - START ========: {}", token_client.balance(&Identifier::Contract(token_contract_id.clone())));

    std::println!("===============");

    contract_client.fund_c(&approval_sign);

    let balance = contract_client.get_bal();
    std::println!("======= ADMIN BALANCE - FUND ========: {}", balance);
    std::println!("======= CONTRACT BALANCE - FUND ========: {}", token_client.balance(&Identifier::Contract(token_contract_id.clone())));
    std::println!("===============");

    let nonce = token_client.nonce(&admin_id);
    // This is the test call, but the contract call arguments and signature payload
    // would be the same for the real contract call too.
    let xfer_approval_sign = ed25519::sign(
        &env,
        // Signer has the private key of the admin.
        &admin_sign,
        // Identifier of the token contract.
        &token_contract_id,
        // Name of the contract function we call.
        symbol!("xfer"),
        // Arguments of the contract function call.
        // Notice that instead of the signature (first `mint` argument), public key
        // is used as the first argument here.
        (&admin_id, &nonce, &approval_user_id, &BigInt::from_u32(&env, 35)),
    );

    contract_client.thank_m(&xfer_approval_sign, &approval_user_id);

    let token_id = contract_client.get_tc_id();

    let client = token::Client::new(&env, &token_id);

    std::println!("======= ADMIN BALANCE - AFTER XFER ========: {}", client.balance(&admin_id));
    std::println!("======= CONTRACT BALANCE - AFTER XFER ========: {}", client.balance(&Identifier::Contract(token_id.clone())));

    std::println!("======= APPROBAL USER BALANCE - AFTER XFER ========: {}", client.balance(&approval_user_id));
    std::println!("===============");

    token_client.with_source_account(&approval_user).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id),
        &client.balance(&approval_user_id)
    );

    std::println!("======= APPROBAL USER BALANCE - AFTER APPROVE ========: {}", client.balance(&approval_user_id));

   contract_client.remove_m(&approval_user_id);
    
    std::println!("======= ADMIN BALANCE - AFTER REMOVE ========: {}", client.balance(&admin_id));
    std::println!("======= CONTRACT BALANCE - AFTER REMOVE ========: {}", client.balance(&Identifier::Contract(token_id.clone())));
    std::println!("======= APPROBAL USER BALANCE - AFTER REMOVE ========: {}", client.balance(&approval_user_id));
    
    // let logs = env.logger().all();
    // std::println!("======= LOGS ========: {}", logs.join("\n"));
}