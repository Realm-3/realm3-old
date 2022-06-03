#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::{BlockNumberProvider, Saturating, Zero},
		sp_std::collections::btree_set::BTreeSet,
		traits::{Currency, ExistenceRequirement},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::{boxed::Box, vec::Vec};

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
	}

	#[derive(
		Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub struct Faucet<BlockNumber, Balance> {
		pub enabled: bool,
		pub period: BlockNumber,
		pub period_limit: Balance,
		pub drip_limit: Balance,
	}

	impl<BlockNumber, Balance> Faucet<BlockNumber, Balance> {
		pub fn new(period: BlockNumber, period_limit: Balance, drip_limit: Balance) -> Self {
			Self { enabled: true, period, period_limit, drip_limit }
		}
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct FaucetUpdate<BlockNumber, Balance> {
		pub enabled: Option<bool>,
		pub period: Option<BlockNumber>,
		pub period_limit: Option<Balance>,
		pub drip_limit: Option<Balance>,
	}

	#[derive(
		Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub struct Drip<BlockNumber, Balance> {
		pub next_period_at: BlockNumber,
		pub dripped_in_current_period: Balance,
	}

	#[pallet::storage]
	#[pallet::getter(fn faucets)]
	pub type Faucets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Faucet<T::BlockNumber, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn drip_info)]
	pub type DripInfo<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Drip<T::BlockNumber, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_faucets: Vec<(T::AccountId, T::BlockNumber, BalanceOf<T>, BalanceOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { initial_faucets: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (a, b, c, d) in &self.initial_faucets {
				let faucet: Faucet<T::BlockNumber, BalanceOf<T>> =
					Faucet { enabled: true, period: *b, period_limit: *c, drip_limit: *d };
				Faucets::<T>::insert(a, faucet);
			}
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FaucetAdded(T::AccountId),
		FaucetUpdated(T::AccountId),
		FaucetsRemoved(Vec<T::AccountId>),
		/// [faucet, recipient, amount]
		Dripped(
			T::AccountId, // Faucet account
			T::AccountId, // Recipient account
			BalanceOf<T>, // Amount dripped
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		FaucetNotFound,
		FaucetAlreadyAdded,
		NoFreeBalanceOnFaucet,
		NotEnoughFreeBalanceOnFaucet,
		NoFaucetsProvided,
		NoUpdatesProvided,
		NothingToUpdate,
		FaucetDisabled,
		NotFaucetOwner,
		RecipientEqualsFaucet,
		DripLimitCannotExceedPeriodLimit,

		ZeroPeriodProvided,
		ZeroPeriodLimitProvided,
		ZeroDripLimitProvided,
		ZeroDripAmountProvided,

		PeriodLimitReached,
		DripLimitReached,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_faucet(
			origin: OriginFor<T>,
			faucet: T::AccountId,
			period: T::BlockNumber,
			period_limit: BalanceOf<T>,
			drip_limit: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::ensure_period_not_zero(&period)?;
			Self::ensure_period_limit_not_zero(&period_limit)?;
			Self::ensure_drip_limit_not_zero(&drip_limit)?;
			Self::ensure_drip_limit_lte_period_limit(&drip_limit, &period_limit)?;

			ensure!(Self::faucets(&faucet).is_none(), Error::<T>::FaucetAlreadyAdded);
			ensure!(
				T::Currency::free_balance(&faucet) >= T::Currency::minimum_balance(),
				Error::<T>::NoFreeBalanceOnFaucet
			);

			let new_faucet =
				Faucet::<T::BlockNumber, BalanceOf<T>>::new(period, period_limit, drip_limit);

			Faucets::<T>::insert(&faucet, new_faucet);
			Self::deposit_event(Event::<T>::FaucetAdded(faucet));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_faucet(
			origin: OriginFor<T>,
			faucet: T::AccountId,
			update: Box<FaucetUpdate<T::BlockNumber, BalanceOf<T>>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let has_updates = update.enabled.is_some()
				|| update.period.is_some()
				|| update.period_limit.is_some()
				|| update.drip_limit.is_some();

			ensure!(has_updates, Error::<T>::NoUpdatesProvided);

			let mut settings = Self::require_faucet(&faucet)?;

			// `true` if there is at least one updated field.
			let mut should_update = false;

			if let Some(enabled) = update.enabled {
				if enabled != settings.enabled {
					settings.enabled = enabled;
					should_update = true;
				}
			}

			if let Some(period) = update.period {
				Self::ensure_period_not_zero(&period)?;

				if period != settings.period {
					settings.period = period;
					should_update = true;
				}
			}

			if let Some(period_limit) = update.period_limit {
				Self::ensure_period_limit_not_zero(&period_limit)?;

				if period_limit != settings.period_limit {
					Self::ensure_drip_limit_lte_period_limit(&settings.drip_limit, &period_limit)?;

					settings.period_limit = period_limit;
					should_update = true;
				}
			}

			if let Some(drip_limit) = update.drip_limit {
				Self::ensure_drip_limit_not_zero(&drip_limit)?;

				if drip_limit != settings.drip_limit {
					Self::ensure_drip_limit_lte_period_limit(&drip_limit, &settings.period_limit)?;

					settings.drip_limit = drip_limit;
					should_update = true;
				}
			}

			ensure!(should_update, Error::<T>::NothingToUpdate);

			Faucets::<T>::insert(faucet.clone(), settings);
			Self::deposit_event(Event::<T>::FaucetUpdated(faucet));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_faucets(origin: OriginFor<T>, faucets: Vec<T::AccountId>) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(!faucets.len().is_zero(), Error::<T>::NoFaucetsProvided);

			let unique_faucets = faucets.iter().collect::<BTreeSet<_>>();
			for faucet in unique_faucets.iter() {
				Faucets::<T>::remove(faucet);
			}

			Self::deposit_event(Event::<T>::FaucetsRemoved(faucets));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn drip(
			origin: OriginFor<T>, // Should be a faucet account
			recipient: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let faucet = ensure_signed(origin)?;

			// Validate input values
			ensure!(faucet != recipient, Error::<T>::RecipientEqualsFaucet);
			ensure!(amount > Zero::zero(), Error::<T>::ZeroDripAmountProvided);

			let settings = Self::require_faucet(&faucet)?;
			ensure!(settings.enabled, Error::<T>::FaucetDisabled);
			ensure!(amount <= settings.drip_limit, Error::<T>::DripLimitReached);

			let faucet_balance = T::Currency::free_balance(&faucet);
			ensure!(amount <= faucet_balance, Error::<T>::NotEnoughFreeBalanceOnFaucet);

			let current_block = <frame_system::Pallet<T>>::current_block_number();
			let mut drip_info = Self::drip_info(&faucet, &recipient);

			if drip_info.next_period_at <= current_block {
				// 	Move to the next period and reset the period stats
				drip_info.next_period_at = current_block.saturating_add(settings.period);
				drip_info.dripped_in_current_period = Zero::zero();
			}

			// Calculate have many tokens still can be dripped in the current period
			let tokens_left_in_current_period =
				settings.period_limit.saturating_sub(drip_info.dripped_in_current_period);

			ensure!(amount <= tokens_left_in_current_period, Error::<T>::PeriodLimitReached);

			T::Currency::transfer(&faucet, &recipient, amount, ExistenceRequirement::KeepAlive)?;

			drip_info.dripped_in_current_period =
				amount.saturating_add(drip_info.dripped_in_current_period);

			Faucets::<T>::insert(&faucet, settings);
			DripInfo::<T>::insert(&faucet, &recipient, drip_info);

			Self::deposit_event(Event::<T>::Dripped(faucet, recipient, amount));
			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn require_faucet(
			faucet: &T::AccountId,
		) -> Result<Faucet<T::BlockNumber, BalanceOf<T>>, DispatchError> {
			Ok(Self::faucets(faucet).ok_or(Error::<T>::FaucetNotFound)?)
		}

		fn ensure_period_not_zero(period: &T::BlockNumber) -> DispatchResult {
			ensure!(*period > Zero::zero(), Error::<T>::ZeroPeriodProvided);
			Ok(())
		}

		fn ensure_period_limit_not_zero(period_limit: &BalanceOf<T>) -> DispatchResult {
			ensure!(*period_limit > Zero::zero(), Error::<T>::ZeroPeriodLimitProvided);
			Ok(())
		}

		fn ensure_drip_limit_not_zero(drip_limit: &BalanceOf<T>) -> DispatchResult {
			ensure!(*drip_limit > Zero::zero(), Error::<T>::ZeroDripLimitProvided);
			Ok(())
		}

		fn ensure_drip_limit_lte_period_limit(
			drip_limit: &BalanceOf<T>,
			period_limit: &BalanceOf<T>,
		) -> DispatchResult {
			ensure!(drip_limit <= period_limit, Error::<T>::DripLimitCannotExceedPeriodLimit);
			Ok(())
		}
	}
}
