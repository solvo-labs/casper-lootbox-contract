use crate::{ alloc::string::{ ToString }, utils::get_current_address };
use alloc::{ collections::BTreeMap, vec::Vec };
use casper_contract::contract_api::storage;
use casper_types::{ URef, account::AccountHash };

pub enum LootboxEvent {
    Purchase {
        caller: AccountHash,
        lootbox_count: u64,
        item_count: u64,
    },
}

pub fn emit(event: &LootboxEvent) {
    let mut events = Vec::new();
    let mut param = BTreeMap::new();
    param.insert(
        "contract_package_hash",
        get_current_address().as_contract_package_hash().unwrap().to_string()
    );
    match event {
        LootboxEvent::Purchase { caller, lootbox_count, item_count } => {
            param.insert("event_type", "purchase".to_string());
            param.insert("caller", caller.to_string());
            param.insert("lootbox_count", lootbox_count.to_string());
            param.insert("item_count", item_count.to_string());
        }
    }
    events.push(param);
    for param in events {
        let _: URef = storage::new_uref(param);
    }
}
