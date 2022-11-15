#![cfg(test)]

use super::{OrganizationContract, OrganizationContractClient, Identifier};

use soroban_sdk::{symbol, Env, testutils::{Accounts}, BigInt, IntoVal, BytesN, Map, Symbol};
use soroban_auth::{Signature, testutils::ed25519};

extern crate std;

use crate::token::{self, TokenMetadata};

fn create_and_init_token_contract(env: &Env, admin_id: &Identifier) -> (BytesN<32>, token::Client) {
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

    (token_id, token_client)
}

#[test]
fn happy_path() {
    let env = Env::default();

    // USERS
    let (admin_id, admin_sign) = ed25519::generate(&env);

    // APPROVAL USER
    let member = env.accounts().generate();
    let member_id = Identifier::Account(member.clone());

    // John Doe
    let doe_user = env.accounts().generate();
    
    // CREATE OUR CUSTOM CONTRACT
    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);
    
    // Initializate Contract with initial values.
    let reward_amount = 30;
    let allowed_funds_to_issue = 10000;
    let org_name = symbol!("Kommit");
    let items = [(symbol!("thank"), 30), (symbol!("congrat"), 25)];
    let rewards: Map<Symbol, u32> = Map::from_array(&env, items);

    std::println!("REWARDS {:?}", rewards.get(symbol!("thank")));

    contract_client.initialize(
        &admin_id, 
        &org_name, 
        &rewards,
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

    std::println!("=======================================================");
    std::println!("======= [{:?}] CONTRACT START ========:", fetched_org_name);
    std::println!("======= ADMIN BALANCE START ========: {}", balance);
    std::println!("=======================================================\n\n");

    contract_client.fund_c(&approval_sign);

    assert_eq!(
        contract_client.get_bal(),
        allowed_funds_to_issue,
        "Correct Funds found on contract"
    );

    let balance = contract_client.get_bal();
    std::println!("=======================================================");
    std::println!("======= ADMIN BALANCE - AFTER FUND ========: {}", balance);
    std::println!("=======================================================\n\n");

    let nonce = token_client.nonce(&admin_id);
    let xfer_approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("xfer"),
        (&admin_id, &nonce, &member_id, &BigInt::from_u32(&env, reward_amount)),
    );

    contract_client.add_m(&member);

    //Validate member was correctly inserted
    assert!(
        contract_client.get_m().contains(&member),
        "Member was successfully removed"
    );

    contract_client.reward_m(&xfer_approval_sign, &member, &symbol!("thank"));

    assert_eq!(
        token_client.balance(&member_id),
        BigInt::from_u32(&env, reward_amount),
        "Correct balance found on rewarded account"
    );

    std::println!("=======================================================");
    std::println!("======= ADMIN BALANCE - AFTER REWARD ========: {}", token_client.balance(&admin_id));
    std::println!("======= APPROBAL USER BALANCE - AFTER REWARD ========: {}", token_client.balance(&member_id));
    std::println!("=======================================================\n\n");

    contract_client.add_m(&doe_user);

    std::println!("======= CONTRACT MEMBERS ========: {:?}", contract_client.get_m());

    token_client.with_source_account(&member).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id),
        &token_client.balance(&member_id)
    );

    std::println!("======= APPROBAL USER BALANCE - AFTER APPROVE ========: {}", token_client.balance(&member_id));

    contract_client.remove_m(&member);

    // Member was correctly removed from organization
    assert!(
        !contract_client.get_m().contains(member),
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
        token_client.balance(&member_id),
        &BigInt::from_u32(&env, 0),
        "Contract admin gets back member funds"
    );

    std::println!("======= ADMIN BALANCE - AFTER REMOVE ========: {}", token_client.balance(&admin_id));
    std::println!("======= APPROBAL USER BALANCE - AFTER REMOVE ========: {}", token_client.balance(&member_id));
    std::println!("======= CONTRACT MEMBERS ========: {:?}", contract_client.get_m());
}

#[test]
#[should_panic(expected = "The user account you're trying to reward doesn't belong to the organization")]
fn remove_no_member_account() {
    let env = Env::default();

    let (admin_id, admin_sign) = ed25519::generate(&env);

    let doe_user = env.accounts().generate();
    
    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);

    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);
    
    let reward_amount = 300;
    let allowed_funds_to_issue = 1000;
    let org_name = symbol!("Kommit");
    let items = [(symbol!("thank"), 35), (symbol!("congrat"), 25)];
    let rewards: Map<Symbol, u32> = Map::from_array(&env, items);

    contract_client.initialize(
        &admin_id, 
        &org_name, 
        &rewards,
        &allowed_funds_to_issue,
        &token_id
    );

    let nonce = token_client.nonce(&admin_id);
    let approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("mint"),
        (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, allowed_funds_to_issue)),
    );

    contract_client.fund_c(&approval_sign);

    let xfer_approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("xfer"),
        (&admin_id, &nonce, &doe_user, &BigInt::from_u32(&env, reward_amount)),
    );

    contract_client.reward_m(&xfer_approval_sign, &doe_user, &symbol!("congrat"));
}

#[test]
#[should_panic(expected = "You are trying to remove an account that doesn't belong to your organization")]
fn reward_no_member_account() {
    let env = Env::default();

    let (admin_id, admin_sign) = ed25519::generate(&env);

    let doe_user = env.accounts().generate();
    let doe_user_id = Identifier::Account(doe_user.clone());

    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);

    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    let allowed_funds_to_issue = 1000;
    let org_name = symbol!("Kommit");
    let items = [(symbol!("thank"), 35), (symbol!("congrat"), 25)];
    let rewards: Map<Symbol, u32> = Map::from_array(&env, items);

    contract_client.initialize(
        &admin_id,
        &org_name,
        &rewards,
        &allowed_funds_to_issue,
        &token_id
    );

    let nonce = token_client.nonce(&admin_id);
    let approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("mint"),
        (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, allowed_funds_to_issue)),
    );

    contract_client.fund_c(&approval_sign);
    token_client.with_source_account(&doe_user).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id),
        &token_client.balance(&doe_user_id)
    );

    contract_client.remove_m(&doe_user);
}

#[test]
#[should_panic(expected = "The reward type you are trying to use isn't supported")]
fn reward_with_invalid_type() {
    let env = Env::default();

    let (admin_id, admin_sign) = ed25519::generate(&env);

    let doe_user = env.accounts().generate();

    let contract_id = env.register_contract(None, OrganizationContract);
    let contract_client = OrganizationContractClient::new(&env, &contract_id);

    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    let reward_amount = 300;
    let allowed_funds_to_issue = 1000;
    let org_name = symbol!("Kommit");
    let items = [(symbol!("thank"), 35), (symbol!("congrat"), 25)];
    let rewards: Map<Symbol, u32> = Map::from_array(&env, items);

    contract_client.initialize(
        &admin_id,
        &org_name,
        &rewards,
        &allowed_funds_to_issue,
        &token_id
    );

    let nonce = token_client.nonce(&admin_id);
    let approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("mint"),
        (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, allowed_funds_to_issue)),
    );

    contract_client.fund_c(&approval_sign);
    contract_client.add_m(&doe_user);

    let xfer_approval_sign = ed25519::sign(
        &env,
        &admin_sign,
        &token_id,
        symbol!("xfer"),
        (&admin_id, &nonce, &doe_user, &BigInt::from_u32(&env, reward_amount)),
    );

    contract_client.reward_m(&xfer_approval_sign, &doe_user, &symbol!("contribut"));
}
