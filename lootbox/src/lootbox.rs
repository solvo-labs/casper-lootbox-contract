use core::ops::Add;

use alloc::{ string::{ String, ToString }, vec::Vec, vec };

use crate::{
    error::Error,
    utils::{ get_key, get_current_address, self },
    events::{ emit, LootboxEvent },
};

use casper_types::{
    account::AccountHash,
    EntryPoint,
    Key,
    ContractHash,
    EntryPointAccess,
    CLType,
    Parameter,
    EntryPointType,
    EntryPoints,
    contracts::NamedKeys,
    U512,
    RuntimeArgs,
    runtime_args,
    URef,
    CLValue,
};
use casper_types_derive::{ CLTyped, FromBytes, ToBytes };

use casper_contract::contract_api::{ runtime, storage, system };
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use tiny_keccak::{ Sha3, Hasher };

const OWNER: &str = "owner";
const NAME: &str = "name";
const DESCRIPTION: &str = "description";
const ASSET: &str = "asset";
const LOOTBOX_PRICE: &str = "lootbox_price";
const ITEMS_PER_LOOTBOX: &str = "items_per_lootbox";
const MAX_LOOTBOXES: &str = "max_lootboxes";
const LOOTBOX_COUNT: &str = "lootbox_count";
const MAX_ITEMS: &str = "max_items";
const ITEM_COUNT: &str = "item_count";
const ITEMS: &str = "items";
const ITEM_OWNERS: &str = "item_owners";
const NFT_COLLECTION: &str = "nft_collection";
const TOKEN_ID: &str = "token_id";
const ITEM_NAME: &str = "item_name";
const DEPOSITED_ITEM_COUNT: &str = "deposited_item_count";
const ITEM_INDEX: &str = "item_index";
const PURSE: &str = "purse";

//entry points
const ENTRY_POINT_ADD_ITEM: &str = "add_item";
const ENTRY_POINT_INIT: &str = "init";
const ENTRY_POINT_PURCHASE: &str = "purchase";
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_GET_PRICE: &str = "get_price";
const ENTRY_POINT_GET_PURSE: &str = "get_purse";

#[derive(Clone, Debug, CLTyped, ToBytes, FromBytes)]
pub struct Item {
    pub id: u64,
    pub rarity: u64,
    pub token_id: u64,
    pub name: String,
}

// admin function
#[no_mangle]
pub extern "C" fn add_item() {
    check_admin_account();

    let token_id: u64 = runtime::get_named_arg(TOKEN_ID);
    let item_name: String = runtime::get_named_arg(ITEM_NAME);

    let contract_address = get_current_address();
    let caller: AccountHash = runtime::get_caller();
    let collection: Key = utils::read_from(NFT_COLLECTION);
    let deposited_item_count: u64 = utils::read_from(DEPOSITED_ITEM_COUNT);
    let max_items: u64 = utils::read_from(MAX_ITEMS);

    if deposited_item_count >= max_items {
        runtime::revert(Error::MaxItemCount);
    }

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    get_approved(collection_hash, caller.into(), token_id).unwrap_or_revert_with(
        Error::NotApproved
    );

    // check owner is caller
    transfer(collection_hash, caller.into(), contract_address.into(), token_id);

    let items_dict: URef = *runtime::get_key(ITEMS).unwrap().as_uref().unwrap();

    storage::dictionary_put(items_dict, &deposited_item_count.to_string(), Item {
        id: deposited_item_count.into(),
        rarity: 0,
        token_id,
        name: item_name,
    });

    runtime::put_key(
        DEPOSITED_ITEM_COUNT,
        storage::new_uref(deposited_item_count.add(1u64)).into()
    );
}

#[no_mangle]
pub extern "C" fn purchase() {
    let lootbox_count: u64 = utils::read_from(LOOTBOX_COUNT);
    let max_lootboxes: u64 = utils::read_from(MAX_LOOTBOXES);

    if lootbox_count > max_lootboxes {
        runtime::revert(Error::LootboxLimit);
    }

    let items_per_lootbox: u64 = utils::read_from(ITEMS_PER_LOOTBOX);
    let mut item_count: u64 = utils::read_from(ITEM_COUNT);
    let max_items: u64 = utils::read_from(MAX_ITEMS);
    let amount: U512 = utils::read_from(LOOTBOX_PRICE);

    let items = *runtime::get_key(ITEMS).unwrap().as_uref().unwrap();
    let item_owners = *runtime::get_key(ITEM_OWNERS).unwrap().as_uref().unwrap();
    let caller: AccountHash = runtime::get_caller();

    let purse = match runtime::get_key(PURSE) {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key(PURSE, new_purse.into());
            new_purse
        }
    };

    for i in 0..items_per_lootbox {
        if item_count >= max_items {
            break;
        }

        let item_id = get_random_item_id(max_items);

        let data: Item = storage
            ::dictionary_get::<Item>(items, &item_id.to_string())
            .unwrap()
            .unwrap();

        storage::dictionary_put(items, &item_id.to_string(), Item {
            id: data.id,
            rarity: i,
            token_id: data.token_id,
            name: data.name,
        });

        storage::dictionary_put(item_owners, &data.id.to_string(), caller);

        item_count += 1;
    }

    runtime::put_key(ITEM_COUNT, storage::new_uref(item_count).into());
    runtime::put_key(LOOTBOX_COUNT, storage::new_uref(lootbox_count.add(1u64)).into());

    // system
    //     ::transfer_from_purse_to_purse(account::get_main_purse(), purse, amount, None)
    //     .unwrap_or_revert();

    emit(&&(LootboxEvent::Purchase { caller, lootbox_count, item_count }))
}

#[no_mangle]
pub extern "C" fn claim() {
    let item_index: u64 = utils::read_from(ITEM_INDEX);

    let item_owners = *runtime::get_key(ITEM_OWNERS).unwrap().as_uref().unwrap();

    let to_account = storage
        ::dictionary_get::<AccountHash>(item_owners, &item_index.to_string())
        .unwrap()
        .unwrap_or_revert_with(Error::ClaimNotFound);

    let items = *runtime::get_key(ITEMS).unwrap().as_uref().unwrap();
    let data: Item = storage
        ::dictionary_get::<Item>(items, &item_index.to_string())
        .unwrap()
        .unwrap();

    let collection: Key = utils::read_from(NFT_COLLECTION);
    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    let contract_address = get_current_address();

    transfer(collection_hash, contract_address.into(), Key::Account(to_account), data.token_id)
}

#[no_mangle]
pub extern "C" fn get_price() {
    let price: U512 = utils::read_from(LOOTBOX_PRICE);

    runtime::ret(CLValue::from_t(price).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let raffle_purse = match runtime::get_key(PURSE) {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key(PURSE, new_purse.into());
            new_purse
        }
    };

    runtime::ret(CLValue::from_t(raffle_purse.into_add()).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn init() {
    storage::new_dictionary(ITEM_OWNERS).unwrap_or_default();
    storage::new_dictionary(ITEMS).unwrap_or_default();
}

#[no_mangle]
pub extern "C" fn call() {
    //constructor
    let name: String = runtime::get_named_arg(NAME);
    let description: String = runtime::get_named_arg(DESCRIPTION);
    let asset: String = runtime::get_named_arg(ASSET);
    let nft_collection: Key = runtime::get_named_arg(NFT_COLLECTION);
    let lootbox_price: U512 = runtime::get_named_arg(LOOTBOX_PRICE);
    let items_per_lootbox: u64 = runtime::get_named_arg(ITEMS_PER_LOOTBOX);
    let max_lootboxes: u64 = runtime::get_named_arg(MAX_LOOTBOXES);
    let max_items: u64 = runtime::get_named_arg(MAX_ITEMS);

    // init
    let item_count: u64 = 0u64;
    let lootbox_count: u64 = 0u64;
    let deposited_item_count: u64 = 0u64;

    //utils
    let owner: AccountHash = runtime::get_caller();
    let now: u64 = runtime::get_blocktime().into();

    // named keys
    let mut named_keys = NamedKeys::new();
    named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());
    named_keys.insert(DESCRIPTION.to_string(), storage::new_uref(description.clone()).into());
    named_keys.insert(ASSET.to_string(), storage::new_uref(asset.clone()).into());
    named_keys.insert(NFT_COLLECTION.to_string(), storage::new_uref(nft_collection.clone()).into());
    named_keys.insert(OWNER.to_string(), storage::new_uref(owner.clone()).into());
    named_keys.insert(LOOTBOX_PRICE.to_string(), storage::new_uref(lootbox_price.clone()).into());
    named_keys.insert(
        ITEMS_PER_LOOTBOX.to_string(),
        storage::new_uref(items_per_lootbox.clone()).into()
    );
    named_keys.insert(MAX_LOOTBOXES.to_string(), storage::new_uref(max_lootboxes.clone()).into());
    named_keys.insert(LOOTBOX_COUNT.to_string(), storage::new_uref(lootbox_count.clone()).into());
    named_keys.insert(MAX_ITEMS.to_string(), storage::new_uref(max_items.clone()).into());
    named_keys.insert(ITEM_COUNT.to_string(), storage::new_uref(item_count.clone()).into());
    named_keys.insert(
        DEPOSITED_ITEM_COUNT.to_string(),
        storage::new_uref(deposited_item_count.clone()).into()
    );

    // entrypoints
    let add_item_entry_point = EntryPoint::new(
        ENTRY_POINT_ADD_ITEM,
        vec![Parameter::new(ITEM_NAME, CLType::String), Parameter::new(TOKEN_ID, CLType::U64)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let init_entry_point = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let purchase_entry_point = EntryPoint::new(
        ENTRY_POINT_PURCHASE,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let claim_entry_point = EntryPoint::new(
        ENTRY_POINT_CLAIM,
        vec![Parameter::new(ITEM_INDEX, CLType::U64)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let get_price_entry_point = EntryPoint::new(
        ENTRY_POINT_GET_PRICE,
        vec![],
        CLType::U512,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let get_purse_entry_point = EntryPoint::new(
        ENTRY_POINT_GET_PURSE,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(add_item_entry_point);
    entry_points.add_entry_point(init_entry_point);
    entry_points.add_entry_point(purchase_entry_point);
    entry_points.add_entry_point(claim_entry_point);
    entry_points.add_entry_point(get_price_entry_point);
    entry_points.add_entry_point(get_purse_entry_point);

    // contract design
    let str1 = name.clone() + "_" + &now.to_string();
    let str2 = String::from("lootbox_package_hash_");
    let str3 = String::from("lootbox_access_uref_");
    let str4 = String::from("lootbox_contract_hash_");
    let hash_name = str2 + &str1;
    let uref_name = str3 + &str1;
    let contract_hash_text = str4 + &str1;

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_name.to_string()),
        Some(uref_name.to_string())
    );

    runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, runtime_args! {});
}

pub fn check_admin_account() {
    let admin: AccountHash = get_key(OWNER);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::AdminError);
    }
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut result: u64 = 0;
    for i in 0..8 {
        result |= (bytes[i] as u64) << ((7 - i) * 8);
    }
    result
}

pub fn get_random_item_id(max_items: u64) -> u64 {
    let now: u64 = runtime::get_blocktime().into();
    let mut sha3 = Sha3::v256();
    let input = now.to_string();

    sha3.update(input.as_ref());

    let mut hash_bytes = [0u8; 32]; // SHA-3-256 for 32 byte
    sha3.finalize(&mut hash_bytes);

    let hash_number = bytes_to_u64(&hash_bytes);

    let item_id = hash_number % max_items;

    return item_id;
}

pub fn get_approved(contract_hash: ContractHash, owner: Key, token_id: u64) -> Option<Key> {
    runtime::call_contract::<Option<Key>>(
        contract_hash,
        "get_approved",
        runtime_args! {
        "owner" => owner,
        "token_id" => token_id
      }
    )
}

pub fn transfer(contract_hash: ContractHash, sender: Key, recipient: Key, token_id: u64) -> () {
    runtime::call_contract::<()>(
        contract_hash,
        "transfer",
        runtime_args! {
          "token_id" => token_id,
          "source_key" => sender,
          "target_key" => recipient,
      }
    )
}
