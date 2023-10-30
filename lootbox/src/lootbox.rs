use core::{ ops::{ Add, Sub, Mul, Div }, char::MAX };

use alloc::{ string::{ String, ToString }, vec::Vec, vec, boxed::Box };

use crate::{ error::Error, utils::{ get_key, get_current_address, self, read_from } };

use casper_types::{
    account::AccountHash,
    U256,
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
};
use casper_types_derive::{ CLTyped, FromBytes, ToBytes };

use casper_contract::contract_api::{ runtime, storage, system, account };
use casper_contract::unwrap_or_revert::UnwrapOrRevert;

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

//entry points
const ENTRY_POINT_ADD_ITEM: &str = "add_item";

#[derive(Clone, Debug, CLTyped, ToBytes, FromBytes)]
pub struct Item {
    pub id: U256,
    pub name: String,
    pub rarity: U256,
}

// admin function
#[no_mangle]
pub extern "C" fn add_item() {
    check_admin_account();
    let items: Vec<Item> = utils::read_from(ITEMS);

    if items.len() == 0 {
        storage::new_dictionary(ITEM_OWNERS).unwrap_or_default();
    }

    let token_id: u64 = runtime::get_named_arg(TOKEN_ID);
    let item_name: String = runtime::get_named_arg(ITEM_NAME);

    let contract_address = get_current_address();
    let caller: AccountHash = runtime::get_caller();
    let collection: Key = utils::read_from(NFT_COLLECTION);
    let mut items: Vec<Item> = utils::read_from(ITEMS);

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    get_approved(collection_hash, caller.into(), token_id).unwrap_or_revert_with(
        Error::NotApproved
    );

    // check owner is caller
    transfer(collection_hash, caller.into(), contract_address.into(), token_id);

    items.push(Item { id: items.len().into(), name: item_name, rarity: U256::zero() })
}

#[no_mangle]
pub extern "C" fn call() {
    //constructor
    let name: String = runtime::get_named_arg(NAME);
    let description: String = runtime::get_named_arg(DESCRIPTION);
    let asset: String = runtime::get_named_arg(ASSET);
    let nft_collection: Key = runtime::get_named_arg(NFT_COLLECTION);
    let lootbox_price: U512 = runtime::get_named_arg(LOOTBOX_PRICE);
    let items_per_lootbox: U256 = runtime::get_named_arg(ITEMS_PER_LOOTBOX);
    let max_lootboxes: U256 = runtime::get_named_arg(MAX_LOOTBOXES);
    let max_items: U256 = runtime::get_named_arg(MAX_ITEMS);

    // init
    let item_count: U256 = U256::zero();
    let lootbox_count: U256 = U256::zero();

    let items: Vec<Item> = [].to_vec();

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
    named_keys.insert(ITEMS.to_string(), storage::new_uref(items.clone()).into());

    // entrypoints
    let add_item_entry_point = EntryPoint::new(
        ENTRY_POINT_ADD_ITEM,
        vec![Parameter::new(ITEM_NAME, CLType::String), Parameter::new(TOKEN_ID, CLType::U64)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(add_item_entry_point);

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
}

pub fn check_admin_account() {
    let admin: AccountHash = get_key(OWNER);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::AdminError);
    }
}

pub fn get_random_item_id() {}

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
