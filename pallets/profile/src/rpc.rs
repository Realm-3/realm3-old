#![cfg_attr(not(feature = "std"), no_std)]
use crate::{Config, Pallet, Pros};
use codec::{Decode, Encode};
use rp_profile::{Area, Content, Profession};
use scale_info::prelude::vec::Vec;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct ProProfile<AccountId> {
	pub id: AccountId,
	pub cid: Vec<u8>,
	pub followers_count: u32,
	pub following_accounts_count: u16,
	pub reputation: u32,
}

impl<T: Config> Pallet<T> {
	pub fn get_pros(
		profession: Profession,
		area: Area,
		offset: u64,
		limit: u16,
	) -> Vec<ProProfile<T::AccountId>> {
		let pro_ids = Pros::<T>::iter()
			.filter(|x| x.1 .0 == profession)
			.filter(|x| {
				if let Some(areas) = &x.1 .1 {
					return areas.contains(&area);
				}
				false
			})
			.map(|x| x.0)
			.collect::<Vec<T::AccountId>>();

		let mut pros = Vec::<ProProfile<T::AccountId>>::new();

		for i in offset as usize.. {
			match pro_ids.get(i) {
				Some(pro_id) => {
					if let Some(pro) = Self::get_social_account(&pro_id) {
						if let Some(profile) = pro.profile {
							if let Content::IPFS(cid) = profile.content {
								pros.push(ProProfile {
									id: pro_ids[i].clone(),
									cid,
									followers_count: pro.followers_count,
									following_accounts_count: pro.following_accounts_count,
									reputation: pro.reputation,
								});
							}
						}
					}
				},
				None => break,
			};

			if pros.len() >= limit as usize {
				break;
			}
		}

		pros
	}
}
