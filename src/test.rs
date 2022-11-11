#![cfg(test)]

use super::{OrganizationContract, OrganizationContractClient, Identifier};

use soroban_sdk::{symbol, Env, testutils::{Accounts}, BigInt, IntoVal};
use soroban_auth::{Signature, testutils::ed25519};

extern crate std;

use crate::token::{self, TokenMetadata};

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
fn happy_path() {
    let env = Env::default();

    // USERS
    let (admin_id, admin_sign) = ed25519::generate(&env);

    // APPROVAL USER
    let approval_user = env.accounts().generate();
    let approval_user_id = Identifier::Account(approval_user.clone());

    // John Doe
    let doe_user = env.accounts().generate();
    
    // CREATE OUR CUSTOM CONTRACT
    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);

    // CREATE TOKEN CONTRACT
    let token_id = env.register_contract_token(None);
    let token_client = token::Client::new(&env, &token_id);

    token_client.init(
        &admin_id,
        &TokenMetadata {
            name: "Mmitkoin".into_val(&env),
            symbol: "MTK".into_val(&env),
            decimals: 7,
        },
    );
    
    // Initializate Contract with initial values.
    let reward_amount = 30;
    let allowed_funds_to_issue = 10000;
    let org_name = symbol!("Kommit");

    contract_client.initialize(
        &admin_id, 
        &org_name, 
        &reward_amount, 
        &allowed_funds_to_issue,
        &token_id
    );

    assert_eq!(
        contract_client.org_name(),
        org_name,
        "Correct name set on contract"
    );

    let nonce = token_client.nonce(&admin_id);
    let approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("mint"),
        (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, allowed_funds_to_issue)),
    );
    let balance = contract_client.get_bal();

    let fetched_org_name = contract_client.org_name();
    std::println!("======= [{:?}] CONTRACT START ========:", fetched_org_name);
    std::println!("======= ADMIN BALANCE START ========: {}", balance);
    std::println!("===============");

    contract_client.fund_c(&approval_sign);

    assert_eq!(
        contract_client.get_bal(),
        allowed_funds_to_issue,
        "Correct Funds found on contract"
    );

    let balance = contract_client.get_bal();
    std::println!("======= ADMIN BALANCE - AFTER FUND ========: {}", balance);
    std::println!("===============");

    let nonce = token_client.nonce(&admin_id);
    let xfer_approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("xfer"),
        (&admin_id, &nonce, &approval_user_id, &BigInt::from_u32(&env, reward_amount)),
    );

    contract_client.add_m(&approval_user);

    //Validate member was correctly inserted
    assert!(
        contract_client.get_m().contains(&approval_user),
        "Member was successfully removed"
    );

    contract_client.reward_m(&xfer_approval_sign, &approval_user);

    assert_eq!(
        token_client.balance(&approval_user_id),
        BigInt::from_u32(&env, reward_amount),
        "Correct balance found on rewarded account"
    );

    std::println!("======= ADMIN BALANCE - AFTER XFER ========: {}", token_client.balance(&admin_id));
    std::println!("======= APPROBAL USER BALANCE - AFTER XFER ========: {}", token_client.balance(&approval_user_id));
    std::println!("===============");

    contract_client.add_m(&doe_user);

    std::println!("======= CONTRACT MEMBERS ========: {:?}", contract_client.get_m());

    token_client.with_source_account(&approval_user).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id),
        &token_client.balance(&approval_user_id)
    );

    std::println!("======= APPROBAL USER BALANCE - AFTER APPROVE ========: {}", token_client.balance(&approval_user_id));

    contract_client.remove_m(&approval_user);

    // Member was correctly removed from organization
    assert!(
        !contract_client.get_m().contains(approval_user),
        "Member was successfully removed"
    );

    // Member funds got back into admin balance
    assert_eq!(
        token_client.balance(&admin_id),
        &BigInt::from_u32(&env, allowed_funds_to_issue),
        "Contract admin gets back member funds"
    );

    // Ensure Member funds where removed
    assert_eq!(
        token_client.balance(&approval_user_id),
        &BigInt::from_u32(&env, 0),
        "Contract admin gets back member funds"
    );

    std::println!("======= ADMIN BALANCE - AFTER REMOVE ========: {}", token_client.balance(&admin_id));
    std::println!("======= APPROBAL USER BALANCE - AFTER REMOVE ========: {}", token_client.balance(&approval_user_id));
    
    std::println!("======= CONTRACT MEMBERS ========: {:?}", contract_client.get_m());
}

#[test]
fn remove_no_member_account() {T

}

#[test]
fn reward_no_member_account() {
    
}