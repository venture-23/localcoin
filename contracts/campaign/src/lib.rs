#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, String, Address, Vec, vec, Val};

mod localcoin {
    soroban_sdk::contractimport!(
        file =
            "../localcoin/target/wasm32-unknown-unknown/release/localcoin.wasm"
    );
}

#[contract]
pub struct Campaign;

#[derive(Clone)]
#[contracttype]
pub enum DataKeys{
    Admin,
    TokenAddress,
    CampaignInfo(Address)
}

#[contractimpl]
impl Campaign {

    pub fn set_campaign_info(env:Env, name:String, description:String, no_of_recipients:u32, token_address:Address, creator: Address){

        // set_campaign_info is only called by campaign_mamagenent contract
        if Self::has_administrator(env.clone()) {
            panic!("Campaign info already set.")
        }
        let owner_key = DataKeys::Admin;
        env.storage().instance().set(&owner_key, &creator);

        let token_key = DataKeys::TokenAddress;
        env.storage().instance().set(&token_key, &token_address);
        
        let mut campaign_info: Vec<Val> = Vec::new(&env);
        campaign_info.push_back(name.to_val());
        campaign_info.push_back(description.to_val());
        campaign_info.push_back(no_of_recipients.into());

        // set campaign info
        let campaign_address = env.current_contract_address();
        let campaign_key = DataKeys:: CampaignInfo(campaign_address);
        env.storage().instance().set(&campaign_key, &campaign_info)
    }

    // transfer token to recipient
    pub fn transfer_tokens_to_recipient(env:Env, to:Address, amount:i128) {
        let owner = Self::get_owner(env.clone());
        owner.require_auth();

        let token_addr = Self::get_token_address(env.clone());
        let current_contract_addr = env.current_contract_address();

        let token_client = localcoin::Client::new(&env, &token_addr);
        token_client.transfer(&current_contract_addr, &to, &amount);
    }

    pub fn has_administrator(env:Env) -> bool {
        let key = DataKeys::Admin;
        env.storage().instance().has(&key)
    }

    pub fn get_owner(env:Env) ->Address {
        let key = DataKeys::Admin;
        if let Some(owner) = env.storage().instance().get::<DataKeys, Address>(&key) {
            owner
        } else {
            panic!("Admin address not set.");
        }
    }

    pub fn get_token_address(env:Env) -> Address {
        let key = DataKeys::TokenAddress;
        if let Some(token) = env.storage().instance().get::<DataKeys, Address>(&key) {
            token
        } else {
            panic!("Token address not set.");
        }
    }

    pub fn get_campaign_info(env:Env, campaign:Address) -> Vec<Val> {
        let key = DataKeys:: CampaignInfo(campaign);
        if let Some(campaign_info) = env.storage().instance().get::<DataKeys, Vec<Val>>(&key) {
            campaign_info
        } else {
            vec![&env] 
        }
    }
}


// soroban contract deploy \
//   --wasm target/wasm32-unknown-unknown/release/campaign.wasm \
//   --source alice \
//   --network testnet

//   soroban contract invoke \
//   --id CDX5RNFOJGOXVMITENBG4CFLDA4L7TS3FZIS6JUB74NTRLMG6RRT2MY4 \
//   --source alice \
//   --network testnet \
//   -- \
//   set_campaign_info \
//   --name "Hello 2" \
//   --description "How u doin" \
//   --no_of_recipients 3 \
//   --creator GB6A2R4B7MSB7HDD56DC4KIUCML3QGF2IT4JLTFHJNMHGGCJOVS3TELN
