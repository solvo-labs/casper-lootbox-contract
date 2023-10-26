#![allow(dead_code)]

use casper_contract::{
    contract_api::runtime,
    contract_api::storage,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ U512, CLTyped, URef, bytesrepr::FromBytes };

use crate::error::Error;
use core::convert::TryInto;

fn current_timestamp() -> U512 {
    let time: u64 = runtime::get_blocktime().into();
    time.into()
}

pub fn get_key<T: FromBytes + CLTyped>(name: &str) -> T {
    let key = runtime
        ::get_key(name)
        .unwrap_or_revert_with(Error::FatalError)
        .try_into()
        .unwrap_or_revert_with(Error::FatalError);
    storage
        ::read(key)
        .unwrap_or_revert_with(Error::FatalError)
        .unwrap_or_revert_with(Error::FatalError)
}

pub(crate) fn read_from<T>(name: &str) -> T where T: FromBytes + CLTyped {
    let uref = get_uref(name);
    let value: T = storage::read(uref).unwrap_or_revert().unwrap_or_revert();
    value
}

/// Gets [`URef`] under a name.
pub(crate) fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name).ok_or(Error::FatalError).unwrap_or_revert();
    key.try_into().unwrap_or_revert()
}
