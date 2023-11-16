#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, vec};

#[contract]
pub struct UserRegisrty;

#[derive(Clone)]
#[contracttype]
pub enum DataKeys{
    SuperAdmin,
    MerchantList,
    CampaignAdmin(Address)
}

#[contractimpl]
impl UserRegisrty {
    // initaialize contract
    pub fn initialize(env:Env, super_admin:Address){
        if Self::has_administrator(env.clone()) {
            panic!("Contract already initialized.")
        }
        let key = DataKeys::SuperAdmin;
        env.storage().instance().set(&key, &super_admin)
    }

    pub fn set_super_admin(env:Env, new_super_admin:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        let key = DataKeys::SuperAdmin;
        env.storage().instance().set(&key, &new_super_admin)
    }

    // super admin registers new merchants
    pub fn on_board_merchants(env:Env, merchant:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        let key = DataKeys::MerchantList;
        let mut merchants_list = Self::get_merchants(env.clone());
        merchants_list.push_back(merchant);
        env.storage().instance().set(&key, &merchants_list)
    }

    // set admin of campaigns
    pub fn set_campaign_admin(env:Env, campaign:Address, admin:Address) {
        let key = DataKeys::CampaignAdmin(campaign);
        env.storage().instance().set(&key, &admin)
    }

    pub fn get_merchants(env:Env) -> Vec<Address> {
        let key = DataKeys::MerchantList;
        if let Some(merchants_list) = env.storage().instance().get::<DataKeys, Vec<Address>>(&key) {
            merchants_list
        } else {
            vec![&env]
        }
    }

    pub fn get_campaign_admin(env:Env, campaign:Address) -> Address {
        let key = DataKeys::CampaignAdmin(campaign);
        if let Some(campaign_admin) = env.storage().instance().get::<DataKeys, Address>(&key) {
            campaign_admin
        } else {
            panic!("Contract doesn't exist.");
        }
    }

    pub fn get_super_admin(env:Env) -> Address {
        let key = DataKeys::SuperAdmin;
        if let Some(super_admin) = env.storage().instance().get::<DataKeys, Address>(&key) {
            super_admin
        } else {
            panic!("Super admin not set.");
        }
    }

    pub fn has_administrator(env:Env) -> bool {
        let key = DataKeys::SuperAdmin;
        env.storage().instance().has(&key)
    }
}

