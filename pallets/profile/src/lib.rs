#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, sp_runtime::traits::BlockNumberProvider};
	use frame_system::pallet_prelude::*;
	use rp_profile::{Area, Content, NewProfile, Pro, Profession, SocialAccount};
	use scale_info::prelude::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pros)]
	pub type Pros<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Pro, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pro_counts)]
	pub type ProCounts<T: Config> = StorageMap<_, Blake2_128Concat, Profession, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn social_account_by_id)]
	pub type SocialAccounts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SocialAccount<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn usernames)]
	pub type Usernames<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [account_id]
		ProfileCreated(T::AccountId),
		/// [account_id]
		ProfileUpdated(T::AccountId),
		/// [account_id, profession]
		ProCreated(T::AccountId, Profession),
		/// [account_id]
		ProUpdated(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidContent,
		ProfileAlreadyCreated,
		UsernameAlreadyExists,
		NoUpdatesForProfile,
		SocialAccountNotFound,
		AccountHasNoProfile,
		AlreadyPro,
		InvalidProfession,
		InvalidAreas,
		ProAccountNeeded,
		ProCountsOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight((
		100_000,
		DispatchClass::Normal,
		Pays::No
		))]
		pub fn create_profile(
			origin: OriginFor<T>,
			username: T::Hash,
			content: Content,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(content.is_valid(), Error::<T>::InvalidContent);

			let mut social_account = Self::get_or_new_social_account(&sender);
			ensure!(social_account.profile.is_none(), Error::<T>::ProfileAlreadyCreated);
			ensure!(!Self::is_username_exist(username.clone()), Error::<T>::UsernameAlreadyExists);

			social_account.profile = Some(NewProfile {
				created: <frame_system::Pallet<T>>::current_block_number(),
				updated: None,
				content,
			});

			SocialAccounts::<T>::insert(&sender, social_account);
			Usernames::<T>::insert(&sender, username);
			Self::deposit_event(Event::<T>::ProfileCreated(sender));

			Ok(())
		}

		#[pallet::weight((
		100_000,
		DispatchClass::Normal,
		Pays::No
		))]
		pub fn update_profile(origin: OriginFor<T>, update: Option<Content>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(update.is_some(), Error::<T>::NoUpdatesForProfile);

			let mut social_account: SocialAccount<T::BlockNumber> =
				Self::social_account_by_id(&sender).ok_or(Error::<T>::SocialAccountNotFound)?;
			let mut profile = social_account.profile.ok_or(Error::<T>::AccountHasNoProfile)?;
			let mut is_update_applied = false;
			let mut _old_data = Option::<Content>::default();

			if let Some(content) = update {
				if content != profile.content {
					ensure!(content.is_valid(), Error::<T>::InvalidContent);

					_old_data = Some(profile.content);
					profile.content = content;
					is_update_applied = true;
				}
			}

			if is_update_applied {
				profile.updated = Some(<frame_system::Pallet<T>>::current_block_number());
				social_account.profile = Some(profile.clone());

				SocialAccounts::<T>::insert(&sender, social_account);
				// T::AfterProfileUpdated::after_profile_updated(owner.clone(), &profile, old_data);

				Self::deposit_event(Event::<T>::ProfileUpdated(sender));
			}

			Ok(())
		}

		#[pallet::weight((
		100_000,
		DispatchClass::Normal,
		Pays::No
		))]
		pub fn become_pro(
			origin: OriginFor<T>,
			profession: Profession,
			areas: Option<Vec<Area>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(profession != Profession::default(), Error::<T>::InvalidProfession);
			ensure!(
				Self::get_or_new_social_account(&sender).profile.is_some(),
				Error::<T>::AccountHasNoProfile
			);
			ensure!(!Pros::<T>::contains_key(&sender), Error::<T>::AlreadyPro);

			if let Some(areas) = &areas {
				ensure!(areas.len() > 0, Error::<T>::InvalidAreas);
			}

			if Self::should_increase_pro_cnt(&sender, &profession, areas.clone()) {
				let new_cnt = Self::pro_counts(&profession)
					.checked_add(1)
					.ok_or(Error::<T>::ProCountsOverflow)?;

				Pros::<T>::insert(&sender, (profession, areas));
				ProCounts::<T>::insert(&profession, new_cnt);
			} else {
				Pros::<T>::insert(&sender, (profession, areas));
			}

			Self::deposit_event(Event::<T>::ProCreated(sender, profession));

			Ok(())
		}

		#[pallet::weight((
		100_000,
		DispatchClass::Normal,
		Pays::No
		))]
		pub fn update_pro_area(origin: OriginFor<T>, areas: Vec<Area>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(areas.len() > 0, Error::<T>::InvalidAreas);
			let pro = Self::pros(&sender).ok_or(Error::<T>::ProAccountNeeded)?;

			if Self::should_increase_pro_cnt(&sender, &Profession::default(), Some(areas.clone())) {
				let new_cnt =
					Self::pro_counts(&pro.0).checked_add(1).ok_or(Error::<T>::ProCountsOverflow)?;

				Pros::<T>::insert(&sender, (pro.0, Some(areas)));
				ProCounts::<T>::insert(&pro.0, new_cnt);
			} else {
				Pros::<T>::insert(&sender, (pro.0, Some(areas)));
			}

			Self::deposit_event(Event::<T>::ProUpdated(sender));

			Ok(())
		}

		#[pallet::weight((
		100_000,
		DispatchClass::Normal,
		Pays::No
		))]
		pub fn update_profession(origin: OriginFor<T>, profession: Profession) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pro = Self::pros(&sender).ok_or(Error::<T>::ProAccountNeeded)?;

			if Self::should_increase_pro_cnt(&sender, &profession, None) {
				let new_cnt = Self::pro_counts(&profession)
					.checked_add(1)
					.ok_or(Error::<T>::ProCountsOverflow)?;

				Pros::<T>::insert(&sender, (profession, pro.1));
				ProCounts::<T>::insert(&profession, new_cnt);
			} else {
				Pros::<T>::insert(&sender, (profession, pro.1));
			}

			Self::deposit_event(Event::<T>::ProUpdated(sender));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_username_exist(username: T::Hash) -> bool {
			Usernames::<T>::iter_values().any(|x| x == username)
		}

		pub fn get_or_new_social_account(
			account_id: &T::AccountId,
		) -> SocialAccount<T::BlockNumber> {
			Self::get_social_account(account_id).unwrap_or(SocialAccount {
				followers_count: 0,
				following_accounts_count: 0,
				reputation: 1,
				profile: None,
			})
		}

		pub fn get_social_account(
			account_id: &T::AccountId,
		) -> Option<SocialAccount<T::BlockNumber>> {
			Self::social_account_by_id(account_id)
		}

		fn should_increase_pro_cnt(
			account_id: &T::AccountId,
			profession: &Profession,
			areas: Option<Vec<Area>>,
		) -> bool {
			let pro: Option<Pro> = Self::pros(account_id);

			if let Some(pro) = pro {
				if pro.0 != Profession::default() && profession != &Profession::default() {
					return false;
				} else if !pro.1.unwrap_or_default().is_empty()
					&& !areas.unwrap_or_default().is_empty()
				{
					return false;
				}
			} else {
				if profession != &Profession::default() {
					return false;
				} else if areas == None {
					return false;
				} else if areas.unwrap_or_default().is_empty() {
					return false;
				}
			}

			true
		}
	}
}
