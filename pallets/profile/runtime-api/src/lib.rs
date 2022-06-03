#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Decode, Encode};
use pallet_profile::rpc::ProProfile;
use rp_profile::{Area, Profession};
use scale_info::TypeInfo;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait ProfileApi<AccountId, Hash> where
		AccountId: Encode + Decode  + Clone + PartialEq + TypeInfo,
		Hash: Encode + Decode + Default + Clone + PartialEq + TypeInfo,
	{
		fn get_pros(profession: Profession, area: Area, offset: u64, limit: u16) -> Vec<ProProfile<AccountId>>;

		fn is_username_exist(username: Hash) -> bool;
	}
}
