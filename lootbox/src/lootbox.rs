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

#[no_mangle]
pub extern "C" fn call() {}
