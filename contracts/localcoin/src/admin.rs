use soroban_sdk::{Address, Env};

use crate::storage_types::DataKey;

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);
}

pub fn write_issuance(e: &Env, id: &Address) {
    let key = DataKey::IssuanceManagement;
    e.storage().instance().set(&key, id);
}

pub fn read_issuance(e: &Env) -> Address {
    let key = DataKey::IssuanceManagement;
    e.storage().instance().get(&key).unwrap()
}

pub fn has_issuance(e: &Env) -> bool {
    let key = DataKey::IssuanceManagement;
    e.storage().instance().has(&key)
}
