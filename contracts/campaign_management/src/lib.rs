#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, BytesN, Env, String, Vec, Val, vec};

mod campaign_contract {
    soroban_sdk::contractimport!(
        file =
            "../campaign/target/wasm32-unknown-unknown/release/campaign.wasm"
    );
}

mod localcoin {
    soroban_sdk::contractimport!(
        file =
            "../localcoin/target/wasm32-unknown-unknown/release/localcoin.wasm"
    );
}

mod user_registry {
    soroban_sdk::contractimport!(
        file =
            "../user_registry/target/wasm32-unknown-unknown/release/user_registry.wasm"
    );
}

#[contracttype]
pub struct CampaignDetail {
    pub campaign: Address,
    pub token: Address,
    pub token_minted: i128,
    pub info: Vec<Val>
}

#[derive(Clone)]
#[contracttype]
pub enum DataKeys {
    UserRegistry,
    StableCoin,
    Campaigns,
    SaltCounter,
    CreatorCampaigns(Address)
}

#[contract]
pub struct CampaignManagement;

#[contractimpl]
impl CampaignManagement {
    // initialize contract
    pub fn initialize(env:Env, address:Address) {
        if Self::has_user_registry(env.clone()) {
            panic!("Contract already initialized.")
        }
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &address)
    }

    pub fn set_user_registry(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &address)
    }

    pub fn set_stable_coin_address(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::StableCoin;
        env.storage().instance().set(&key, &address)
    }

    pub fn create_campaign(env:Env, name:String, description:String, no_of_recipients:u32,
         token_address:Address, amount:i128, creator: Address) {

            creator.require_auth();

            // transfer stable coin to campaign management from 'creator' address
            let stable_coin_addr = Self::get_stable_coin(env.clone());
            let stable_coin_client = token::Client::new(&env, &stable_coin_addr);
            stable_coin_client.transfer(&creator, &env.current_contract_address(), &amount);
            
            let wasm_hash = env.deployer().upload_contract_wasm(campaign_contract::WASM);

            // dynamic salt count to deploy multiple contracts
            let salt_key = DataKeys::SaltCounter;
            let salt_count: u32 = env.storage().instance().get::<DataKeys, u32>(&salt_key).unwrap_or(0);
            let salt = BytesN::from_array(&env, &[salt_count.try_into().unwrap(); 32]);

            // deploy campaign contract
            let campaign_contract_addr = env.deployer().with_address(creator.clone(), salt).deploy(wasm_hash);

            // increase salt counter 
            env.storage().instance().set(&salt_key, &(salt_count + 1));

            // call set_campaign_info on camapign contract through client
            let campaign_client = campaign_contract::Client::new(&env, &campaign_contract_addr);
            campaign_client.set_campaign_info(&name, &description, &no_of_recipients, &token_address, &creator); 

            // mint stable coin equivalent tokens to campaign contract
            let token_client = localcoin::Client::new(&env, &token_address);
            token_client.mint(&campaign_contract_addr, &amount);

            // set campaign admin in user_registry contract
            let user_registry_addr = Self::get_user_registry(env.clone());
            let user_registry_client = user_registry::Client::new(&env, &user_registry_addr);
            user_registry_client.set_campaign_admin(&env.current_contract_address(), &campaign_contract_addr, &creator);
            
            // store all campaign
            let campaign_key = DataKeys::Campaigns;
            let mut campaign_list = Self::get_campaigns(env.clone());
            campaign_list.push_back(campaign_contract_addr.clone());
            env.storage().instance().set(&campaign_key, &campaign_list);

            // store campaigns info of the creator
            let key = DataKeys:: CreatorCampaigns(creator.clone());
            let mut creator_campaigns = Self::get_creator_campaigns(env.clone(), creator);

            let mut new_campaign_info: Vec<Val> = Vec::new(&env);
            new_campaign_info.push_back(name.to_val());
            new_campaign_info.push_back(description.to_val());
            new_campaign_info.push_back(no_of_recipients.into());

            let campaign_value = CampaignDetail {
                campaign: campaign_contract_addr,
                token: token_address,
                token_minted: amount,
                info: new_campaign_info
            };
            creator_campaigns.push_back(campaign_value);
            env.storage().instance().set(&key, &creator_campaigns)
    }

    pub fn request_campaign_settelment(env:Env, from:Address, amount:i128, token_address:Address) {
        // transaction should be sent from 'from' addesss
        from.require_auth();

        let user_registry = Self::get_user_registry(env.clone());
        let merchants = user_registry::Client::new(&env, &user_registry).get_verified_merchants();

        if !merchants.contains(&from) {
            panic!("Caller not merchant.")
        }

        // TODO: verify token and associated merchant
        // remove campaign id

        // this above TODO is not needed because the merchant will only receive the token associated to them
        // this is checked while the recipient transfers token to merchant

        let token_client = localcoin::Client::new(&env, &token_address);
        // verify balance of merchant
        let balance = token_client.balance_of(&from);
        if !balance >= amount {
            panic!("Insufficient token for settelment.")
        }

        // campaign_management burns the token from merchant 'from'
        token_client.burn(&from, &amount);

        // TODO: transfer stable coin to super admin
    }

    pub fn get_campaigns(env:Env) -> Vec<Address> {
        let key = DataKeys::Campaigns;
        if let Some(campaigns) = env.storage().instance().get::<DataKeys, Vec<Address>>(&key) {
            campaigns
        } else {
            vec![&env]
        }
    }

    pub fn get_creator_campaigns(env:Env, creator:Address) -> Vec<CampaignDetail> {
        let key = DataKeys:: CreatorCampaigns(creator);
        if let Some(campaigns_info) = env.storage().instance().get::<DataKeys, Vec<CampaignDetail>>(&key) {
            campaigns_info
        } else {
            vec![&env] 
        }
    }

    pub fn get_user_registry(env:Env) -> Address {
        let key = DataKeys::UserRegistry;
        if let Some(user_registry_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            user_registry_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_stable_coin(env:Env) -> Address {
        let key = DataKeys::StableCoin;
        if let Some(stable_coin_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            stable_coin_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_super_admin(env:Env) -> Address {
        let user_registry_addr = Self::get_user_registry(env.clone());
        let client = user_registry::Client::new(&env, &user_registry_addr);
        client.get_super_admin()
    }

    pub fn has_user_registry(env:Env) -> bool {
        let key = DataKeys::UserRegistry;
        env.storage().instance().has(&key)
    }
}

// soroban contract deploy \
//   --wasm target/wasm32-unknown-unknown/release/campaign_management.wasm \
//   --source alice \
//   --network testnet

//   soroban contract invoke \
//   --id CATDHGFYOJESWUTXKVTU6OLH6SEHVNTY4DZWZ5FJLURO62GIAPBNARMT \
//   --source alice \
//   --network testnet \
//   -- \
//   create_campaign \
//   --name "Hello 2" \
//   --description "How u doin" \
//   --no_of_recipients 3 \
//   --token_address CDOYR5LVRTZZABLXIOG6WQV7Z63UIABS6AHM3QZHZPSTJR5F4F4G3FO2 \
//   --amount 10000 \
//   --creator GB6A2R4B7MSB7HDD56DC4KIUCML3QGF2IT4JLTFHJNMHGGCJOVS3TELN

//   localcoin addr - CA6NUSK2W5GR5PAGGLOUUYZ7E67DQJTZFRDZRXY4TS3AF3ZZYTUMTN3T
  
//   soroban contract invoke \
//   --id CDOYR5LVRTZZABLXIOG6WQV7Z63UIABS6AHM3QZHZPSTJR5F4F4G3FO2 \
//   --source alice \
//   --network testnet \
//   -- \
//   balance_of \
//   --id CDH4PTQRZPXB4YZLFUB3K4N5RG2BUUPBQ723CKXZMMJHTJEYK6HW4GFE 
  
