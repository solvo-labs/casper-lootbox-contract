use core::ops::{ Add, Sub, Mul, Div };

use alloc::{ string::{ String, ToString }, vec::Vec, vec, boxed::Box };

use crate::{ error::Error, utils::{ get_key, self, read_from } };

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

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(NAME);
    let description: String = runtime::get_named_arg(DESCRIPTION);
    let asset: String = runtime::get_named_arg(ASSET);

    //utils
    let owner: AccountHash = runtime::get_caller();
    let now: u64 = runtime::get_blocktime().into();

    let mut named_keys = NamedKeys::new();

    named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());
    named_keys.insert(DESCRIPTION.to_string(), storage::new_uref(description.clone()).into());
    named_keys.insert(ASSET.to_string(), storage::new_uref(asset.clone()).into());
    named_keys.insert(OWNER.to_string(), storage::new_uref(owner.clone()).into());

    let mut entry_points = EntryPoints::new();

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

// pub fn check_admin_account() {
//     let admin: AccountHash = get_key(OWNER);
//     let caller = runtime::get_caller();
//     if admin != caller {
//         runtime::revert(Error::AdminError);
//     }
// }

// pub fn get_approved(contract_hash: ContractHash, owner: Address, token_id: u64) -> Option<Key> {
//     runtime::call_contract::<Option<Key>>(
//         contract_hash,
//         "get_approved",
//         runtime_args! {
//         "owner" => owner,
//         "token_id" => token_id
//       }
//     )
// }

// pub fn transfer(contract_hash: ContractHash, sender: Key, recipient: Key, token_id: u64) -> () {
//     runtime::call_contract::<()>(
//         contract_hash,
//         "transfer",
//         runtime_args! {
//           "token_id" => token_id,
//           "source_key" => sender,
//           "target_key" => recipient,
//       }
//     )
// }
