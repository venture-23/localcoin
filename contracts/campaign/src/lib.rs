#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, String, Address, Symbol};

#[contract]
pub struct Campaign;

#[derive(Clone)]
#[contracttype]
pub struct CampaignInfo {
    pub name: String,
    pub description: String,
    pub number_of_participants: u32,
    pub creator: Address
}

const CAMP_INFO: Symbol = symbol_short!("CAMP_INFO");

#[contractimpl]
impl Campaign {

    pub fn set_campaign_info(e: Env, name:String, description:String, no_of_recipients:u32, creator: Address){
        let campaign_info = CampaignInfo{
            name,
            description,
            number_of_participants,
            creator
        };
        e.storage().instance().set(&CAMP_INFO, &campaign_info)
    }
}
