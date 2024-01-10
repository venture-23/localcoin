#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, String, Address, Map, Val, Vec, IntoVal, vec, map};

mod test;

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
    Owner,
    TokenAddress,
    CampaignManagement,
    CampaignInfo,
    RecipientsStatus,
    VerifiedRecipientList,
    IsEnded,
    AmountReceived(Address)
}

#[contractimpl]
impl Campaign {

    pub fn set_campaign_end_status(env:Env, status:bool) {
        let campaign_management = Self::get_campaign_management(env.clone());
        campaign_management.require_auth();

        let status_key = DataKeys::IsEnded;
        env.storage().instance().set(&status_key, &status);
    }

    pub fn set_campaign_info(env:Env, name:String, description:String, no_of_recipients:u32, token_address:Address,
         creator:Address, campaign_management:Address, location:String) {

            // set_campaign_info is only called by campaign_mamagenent contract
            if Self::has_administrator(env.clone()) {
                panic!("Campaign info already set.")
            }
            let owner_key = DataKeys::Owner;
            env.storage().instance().set(&owner_key, &creator);

            let token_key = DataKeys::TokenAddress;
            env.storage().instance().set(&token_key, &token_address);

            let token_client = localcoin::Client::new(&env, &token_address);
            let token_name = token_client.name();

            let campaign_management_key = DataKeys::CampaignManagement;
            env.storage().instance().set(&campaign_management_key, &campaign_management);

            let status_key = DataKeys::IsEnded;
            env.storage().instance().set(&status_key, &false);
            
            let mut campaign_info: Map<String, Val> = Map::new(&env);
            campaign_info.set(String::from_str(&env, "name"), name.to_val());
            campaign_info.set(String::from_str(&env, "description"), description.to_val());
            campaign_info.set(String::from_str(&env, "no_of_recipients"), no_of_recipients.into());
            campaign_info.set(String::from_str(&env, "token_address"), token_address.to_val());
            campaign_info.set(String::from_str(&env, "token_name"), token_name.to_val());
            campaign_info.set(String::from_str(&env, "creator"), creator.to_val());
            campaign_info.set(String::from_str(&env, "location"), location.to_val());

            // set campaign info
            let campaign_key = DataKeys:: CampaignInfo;
            env.storage().instance().set(&campaign_key, &campaign_info);
            // emit event
            env.events().publish((creator, campaign_info), "Campaign info set.");
    }

    // transfer token to recipient
    pub fn transfer_tokens_to_recipient(env:Env, to:Address, amount:i128) {
        let owner = Self::get_owner(env.clone());
        owner.require_auth();

        let verified_recipient_list = Self::get_verified_recipients(env.clone());
        if !(verified_recipient_list.contains(to.clone())) {
            panic!("Recipient not verified.")
        }

        let amount_key = DataKeys::AmountReceived(to.clone());
        let previous_received = Self::get_amount_received(env.clone(), to.clone());
        env.storage().instance().set(&amount_key, &(previous_received + amount));

        let token_addr = Self::get_token_address(env.clone());
        let current_contract_addr = env.current_contract_address();

        let token_client = localcoin::Client::new(&env, &token_addr);
        token_client.transfer(&current_contract_addr, &to, &amount);
        // emit event
        env.events().publish((current_contract_addr, to, amount), "Token transferred to recipient.");
    }

    pub fn join_campaign(env:Env, username:String, recipient:Address) {
        recipient.require_auth();

        let key: DataKeys = DataKeys::RecipientsStatus;
        let mut recipients_status = Self::get_recipients_status(env.clone());
        
        if recipients_status.contains_key(username.clone()) {
            panic!("Campaign already joined.")
        }
        recipients_status.set(username.clone(), (false, recipient.clone()));
        env.storage().instance().set(&key, &recipients_status);

        env.events().publish((username, recipient), "Campaign joined.");
    }

    pub fn verify_recipients(env:Env, usernames:Vec<String>) {
        let owner = Self::get_owner(env.clone());
        owner.require_auth();

        let key: DataKeys = DataKeys::RecipientsStatus;
        let mut recipients_status = Self::get_recipients_status(env.clone());
        let recipient_key =  DataKeys::VerifiedRecipientList;
        let mut verified_recipient_list = Self::get_verified_recipients(env.clone());

        for username in usernames.iter() {
            if !(recipients_status.contains_key(username.clone())) {
                panic!("Given list contains username thet has't joined campaign.")
            }
            let (_, recipient) = recipients_status.get_unchecked(username.clone());            
            if verified_recipient_list.contains(recipient.clone()) {
                panic!("Given list contains already verified username.")
            }
            recipients_status.set(username, (true, recipient.clone()));
            verified_recipient_list.push_back(recipient);
        }
        env.storage().instance().set(&key, &recipients_status);
        env.storage().instance().set(&recipient_key, &verified_recipient_list);
    }

    pub fn recipient_limit_exceeded(env:Env) -> bool {
        let verified_recipient_length = Self::get_verified_recipients(env.clone()).len();
        let no_of_recipient = Self::get_campaign_info(env.clone()).get_unchecked(String::from_str(&env, "no_of_recipients")).into_val(&env);
        if verified_recipient_length >= no_of_recipient {
            return true
        }
        return false
    }

    pub fn get_recipients_status(env:Env) -> Map<String, (bool, Address)> {
        let key = DataKeys::RecipientsStatus;
        if let Some(recipients_status) = env.storage().instance().get::<DataKeys,  Map<String, (bool, Address)>>(&key) {
            recipients_status
        } else {
            map![&env]
        }
    }

    pub fn get_verified_recipients(env:Env) -> Vec<Address> {
        let key = DataKeys::VerifiedRecipientList;
        if let Some(recipients) = env.storage().instance().get::<DataKeys, Vec<Address>>(&key) {
            recipients
        } else {
            vec![&env]
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

    pub fn get_campaign_management(env:Env) -> Address {
        let key = DataKeys::CampaignManagement;
        if let Some(campaign_mamagenent) = env.storage().instance().get::<DataKeys, Address>(&key) {
            campaign_mamagenent
        } else {
            panic!("Camapign management address not set.");
        }
    }

    pub fn get_campaign_info(env:Env) -> Map<String, Val> {
        let key = DataKeys::CampaignInfo;
        if let Some(campaign_info) = env.storage().instance().get::<DataKeys, Map<String, Val>>(&key) {
            campaign_info
        } else {
            Map::new(&env) 
        }
    }

    pub fn get_campaign_balance(env:Env) -> i128 {
        let token_addr = Self::get_token_address(env.clone());
        let current_contract_addr = env.current_contract_address();
        let token_client = localcoin::Client::new(&env, &token_addr);

        let balance = token_client.balance(&current_contract_addr);
        balance
    }

    pub fn get_amount_received(env:Env, recipient:Address) -> i128 {
        let key = DataKeys::AmountReceived(recipient);
        let amount = env.storage().instance().get::<DataKeys, i128>(&key).unwrap_or(0);
        amount
    }

    pub fn get_owner(env:Env) ->Address {
        let key = DataKeys::Owner;
        if let Some(owner) = env.storage().instance().get::<DataKeys, Address>(&key) {
            owner
        } else {
            panic!("Owner address not set.");
        }
    }

    pub fn is_ended(env:Env) -> bool {
        let key = DataKeys::IsEnded;
        let status = env.storage().instance().get::<DataKeys, bool>(&key).unwrap_or(false);
        status
    }

    pub fn has_administrator(env:Env) -> bool {
        let key = DataKeys::Owner;
        env.storage().instance().has(&key)
    }

}