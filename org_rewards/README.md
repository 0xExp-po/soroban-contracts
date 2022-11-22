# Organization's reward contract
A smart contract written in Soroban to reward members of an organization who perform meaningful tasks.

This smart contract provides an organization the ability of:
- Define the reward token for the organization.
- Define the custom rewards and their respective compensation.
- Add members.
- Revoke memberships.
- Reward members according to the organization's specific rules.
- Return the balance of a member who leaves the organization.

## Contract workflow
1. Generate an admin account.
2. Create and initialize the stellar token. This step relies on the built-in token contract. \
  **Note:** the initialization for the token contract must be skipped if the token already exists in the stellar network.
3. Initialize the organization contract with your custom rewards.
4. Generate a signature to fund the organization's token balance using the administrator's account.
5. Fund the balance of the contract using the previously generated signature.
6. Add members to the organization.
7. Create a signature to enable token transfer for the accounts. \
  **Note:** a signature is required for each transfer transaction.
8. Reward members.

**Note:** Signatures are required to execute any functions involving calls to **privileged functions** of the token contract. [Token Contract Interface](https://soroban.stellar.org/docs/common-interfaces/token).

## Revoke membership
1. Approve the transaction using the token contract.
2. Transfer the balance to the organization by revoking the membership.

## Setup
For setting up your environment, visit: [Soroban setup](https://soroban.stellar.org/docs/getting-started/setup)

## Testing
For testing the contract run `cargo test -- --show-output`
