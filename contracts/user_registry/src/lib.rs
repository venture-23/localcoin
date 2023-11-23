#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, String, Env, Vec, Val, Map, vec, map};

#[contract]
pub struct UserRegisrty;

#[derive(Clone)]
#[contracttype]
pub enum DataKeys{
    SuperAdmin,
    VerifiedMerchantList,
    MerchantsInfo(Address),
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

    pub fn merchant_registration(env:Env, merchant_wallet:Address, proprietor:String, phone_no:String, store_name:String, location:String) {
        let key = DataKeys::MerchantsInfo(merchant_wallet);

        let merchant_info: Map<String, Val> = map![
            &env,
            (String::from_slice(&env, "status"), false.into()),
            (String::from_slice(&env, "proprietor"), proprietor.to_val()),
            (String::from_slice(&env, "phone_no"), phone_no.to_val()),
            (String::from_slice(&env, "store_name"), store_name.to_val()),
            (String::from_slice(&env, "location"), location.to_val())
            ];
        env.storage().instance().set(&key, &merchant_info)
    }

    pub fn verify_merchant(env:Env, merchant_addr:Address) {
        // super admin verifies new merchant
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        // update merchant status to true
        let key = DataKeys::MerchantsInfo(merchant_addr.clone());
        let mut merchant_info = Self::get_merchant_info(env.clone(), merchant_addr.clone());
        merchant_info.set(String::from_slice(&env, "status"), true.into());
        env.storage().instance().set(&key, &merchant_info);

        // store a list of merchants
        let key = DataKeys::VerifiedMerchantList;
        let mut merchants_list = Self::get_verified_merchants(env.clone());
        merchants_list.push_back(merchant_addr);
        env.storage().instance().set(&key, &merchants_list)
    }

    pub fn set_campaign_admin(env:Env, campaign_management:Address, campaign:Address, admin:Address) {
        // TODO: check if this auth works
        campaign_management.require_auth();

        let key = DataKeys::CampaignAdmin(campaign);
        env.storage().instance().set(&key, &admin)
    }

    pub fn get_merchant_info(env:Env, merchant_addr:Address) -> Map<String, Val> {
        let key = DataKeys::MerchantsInfo(merchant_addr);
        if let Some(merchant_info) = env.storage().instance().get::<DataKeys, Map<String, Val>>(&key) {
            merchant_info
        } else {
            map![&env]
        }
    }

    pub fn get_verified_merchants(env:Env) -> Vec<Address> {
        let key = DataKeys::VerifiedMerchantList;
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

