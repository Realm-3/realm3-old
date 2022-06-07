#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use scale_info::prelude::{vec, vec::Vec};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

pub type Area = u32;
pub type Pro = (Profession, Option<Vec<Area>>);

#[derive(Encode, Decode, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Content {
	/// No content.
	None,
	/// A raw vector of bytes.
	#[allow(clippy::upper_case_acronyms)]
	IPFS(Vec<u8>),
}

impl From<Content> for Vec<u8> {
	fn from(content: Content) -> Vec<u8> {
		match content {
			Content::None => vec![],
			Content::IPFS(vec_u8) => vec_u8,
		}
	}
}

impl Default for Content {
	fn default() -> Self {
		Self::None
	}
}

impl Content {
	pub fn is_none(&self) -> bool {
		self == &Self::None
	}

	pub fn is_ipfs(&self) -> bool {
		matches!(self, Self::IPFS(_))
	}

	pub fn is_valid(&self) -> bool {
		match self {
			Self::None => true,
			Self::IPFS(cid) => {
				let len = cid.len();

				len == 46 || len == 59
			},
		}
	}
}

// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// pub struct WhoAndWhen<AccountId, BlockNumber, Moment> {
// 	pub account: AccountId,
// 	pub block: BlockNumber,
// 	pub time: Moment,
// }
//
// impl<AccountId, BlockNumber, Moment> WhoAndWhen<AccountId, BlockNumber, Moment> {
// 	pub fn new(account: T::AccountId) -> Self {
// 		WhoAndWhen {
// 			account,
// 			block: <system::Pallet<T>>::block_number(),
// 			time: <pallet_timestamp::Pallet<T>>::now(),
// 		}
// 	}
// }

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SocialAccount<BlockNumber> {
	pub followers_count: u32,
	pub following_accounts_count: u16,
	pub reputation: u32,
	pub profile: Option<NewProfile<BlockNumber>>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NewProfile<BlockNumber> {
	pub created: BlockNumber,
	pub updated: Option<BlockNumber>,
	pub content: Content,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Profile<Hash> {
	pub name: Option<Vec<u8>>,
	pub email: Option<Vec<u8>>,
	pub username: Option<Vec<u8>>,
	pub photo: Option<Vec<u8>>,
	pub dob: Option<Vec<u8>>,
	pub bio: Option<Vec<u8>>,
	pub gender: Option<Gender>,
	pub profession: Option<Vec<u8>>,
	pub about: About,
	pub business: Business<Hash>,
}

impl<Hash> Profile<Hash> {
	pub fn merge(old: Self, new: Self) -> Self {
		let about = About {
			biography: Self::check_value(old.about.biography, new.about.biography),
			service_area: Self::check_value(old.about.service_area, new.about.service_area),
			education: Self::check_value(old.about.education, new.about.education),
			awards: Self::check_value(old.about.awards, new.about.awards),
			specialties: Self::check_value(old.about.specialties, new.about.specialties),
			languages: Self::check_value(old.about.languages, new.about.languages),
		};

		let business = Business {
			profession: Self::check_value(
				Some(old.business.profession),
				Some(new.business.profession),
			)
			.unwrap(),
			field: Self::check_value(old.business.field, new.business.field),
			company: Self::check_value(old.business.company, new.business.company),
			website: Self::check_value(old.business.website, new.business.website),
			phone: Self::check_value(old.business.phone, new.business.phone),
			email: Self::check_value(old.business.email, new.business.email),
		};

		Self {
			name: Self::check_value(old.name, new.name),
			email: Self::check_value(old.email, new.email),
			username: Self::check_value(old.username, new.username),
			photo: Self::check_value(old.photo, new.photo),
			dob: Self::check_value(old.dob, new.dob),
			bio: Self::check_value(old.bio, new.bio),
			gender: Self::check_value(old.gender, new.gender),
			profession: Self::check_value(old.profession, new.profession),
			about,
			business,
		}
	}

	fn check_value<K>(old: Option<K>, new: Option<K>) -> Option<K> {
		match new {
			Some(n) => Option::Some(n),
			None => old,
		}
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct About {
	pub biography: Option<Vec<u8>>,
	pub service_area: Option<ServiceArea>,
	pub education: Option<Vec<Vec<u8>>>,
	pub awards: Option<Vec<Vec<u8>>>,
	pub specialties: Option<Vec<Vec<u8>>>,
	pub languages: Option<Vec<Vec<u8>>>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ServiceArea {
	pub countries: Vec<Vec<u8>>,
	pub provinces: Vec<Vec<u8>>,
	pub areas: Vec<Vec<u8>>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Business<Hash> {
	pub profession: Profession,
	pub field: Option<Field>,
	pub company: Option<Hash>,
	pub website: Option<Vec<u8>>,
	pub phone: Option<Vec<u8>>,
	pub email: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Gender {
	None,
	Male,
	Female,
	Other,
}

impl Default for Gender {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Profession {
	None,
	RealEstateBroker,
	RealEstateAgent,
	RealEstateAttorney,
	MortgageBroker,
	HomeInspector,
	ResidentialAppraiser,
	CommercialAppraiser,
	RealEstatePhotographer,
	HomeStager,
	Architect,
	InteriorDesigner,
	RealEstateInvestor,
	PropertyManager,
	LeasingConsultant,
	RealEstateFinancialAnalyst,
	RealEstateMarketingSpecialist,
	RealEstateEscrowOfficer,
	EscrowOfficer,
	ForeclosureSpecialist,
	RealEstateDeveloper,
	RealEstateWholesaler,
	RealEstateAssistant,
	RealEstateLoanOfficer,
	Other(Option<OtherProfessions>),
}

impl Default for Profession {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OtherProfessions {
	CorporateRealEstateManager,
	CommunityDevelopmentManager,
	ComplianceSpecialist,
	LandAdministrationManager,
	LeaseAdministrator,
	MortgageCollectionManager,
	MortgageLoanOfficer,
	RetailRealEstateManager,
	RealEstateZoningManager,
	RealAstateAndRelocationDirector,
}

#[derive(Encode, Decode, Clone, PartialEq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Field {
	None,
	Residential,
	Commercial,
	Both,
}

impl Default for Field {
	fn default() -> Self {
		Self::None
	}
}
