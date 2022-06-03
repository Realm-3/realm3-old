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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxConnectionRequests: Get<u32>;
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum ConnectionStatus {
		Connected,
		Pending,
		Rejected,
		None,
	}

	impl Default for ConnectionStatus {
		fn default() -> Self {
			ConnectionStatus::None
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn following_cnt)]
	pub(super) type FollowingCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn followers_cnt)]
	pub(super) type FollowersCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connected_cnt)]
	pub(super) type ConnectedCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn following)]
	pub type Following<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connected)]
	pub type Connected<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), ConnectionStatus, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_requests)]
	pub type ConnectionRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::AccountId, T::MaxConnectionRequests>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [sender, destination]
		Follow(T::AccountId, T::AccountId),
		/// [sender, destination]
		Unfollow(T::AccountId, T::AccountId),
		/// [sender, destination, connection_status]
		Connect(T::AccountId, T::AccountId, ConnectionStatus),
		/// [sender, destination]
		RemoveConnection(T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		StorageOverflow,
		AlreadyFollowing,
		AlreadyConnected,
		ConnectionNotFound,
		ConnectionIsNotPending,
		AlreadyRequested,
		NotFollowing,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn follow(
			origin: OriginFor<T>,
			destination: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let key = (&sender, &destination);

			ensure!(!<Following<T>>::contains_key(key), Error::<T>::AlreadyFollowing);

			let new_following_cnt =
				Self::following_cnt(&sender).checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			let new_followers_cnt = Self::followers_cnt(&destination)
				.checked_add(1)
				.ok_or(Error::<T>::StorageOverflow)?;

			<Following<T>>::insert(key, true);
			<FollowingCnt<T>>::insert(&sender, new_following_cnt);
			<FollowersCnt<T>>::insert(&destination, new_followers_cnt);
			Self::deposit_event(Event::<T>::Follow(sender, destination));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn unfollow(
			origin: OriginFor<T>,
			destination: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let key = (&sender, &destination);

			ensure!(<Following<T>>::contains_key(key), Error::<T>::NotFollowing);

			let new_following_cnt =
				Self::following_cnt(&sender).checked_sub(1).ok_or(Error::<T>::StorageOverflow)?;
			let new_followers_cnt = Self::followers_cnt(&destination)
				.checked_sub(1)
				.ok_or(Error::<T>::StorageOverflow)?;

			<Following<T>>::remove(key);
			<FollowingCnt<T>>::insert(&sender, new_following_cnt);
			<FollowersCnt<T>>::insert(&destination, new_followers_cnt);
			Self::deposit_event(Event::<T>::Unfollow(sender, destination));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn connect(
			origin: OriginFor<T>,
			destination: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let key = (&sender, &destination);

			ensure!(<Connected<T>>::contains_key(key), Error::<T>::AlreadyConnected);

			<Connected<T>>::insert(key, ConnectionStatus::Pending);
			Self::deposit_event(Event::<T>::Connect(
				sender,
				destination,
				ConnectionStatus::Pending,
			));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn connect_response(
			origin: OriginFor<T>,
			destination: T::AccountId,
			accept: bool,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let key = (&destination, &sender);
			let connection: ConnectionStatus = Self::connected(key);

			ensure!(connection == ConnectionStatus::Pending, Error::<T>::ConnectionIsNotPending);

			if accept {
				let key_reversed = (&sender, &destination);
				let new_sender_connection_cnt = Self::connected_cnt(&sender)
					.checked_add(1)
					.ok_or(Error::<T>::StorageOverflow)?;
				let new_destination_connection_cnt = Self::connected_cnt(&destination)
					.checked_add(1)
					.ok_or(Error::<T>::StorageOverflow)?;

				<Connected<T>>::insert(key, ConnectionStatus::Connected);
				<Connected<T>>::insert(key_reversed, ConnectionStatus::Connected);
				<ConnectedCnt<T>>::insert(&sender, new_sender_connection_cnt);
				<ConnectedCnt<T>>::insert(&destination, new_destination_connection_cnt);
				Self::deposit_event(Event::<T>::Connect(
					sender,
					destination,
					ConnectionStatus::Connected,
				));
			} else {
				<Connected<T>>::insert(key, ConnectionStatus::Rejected);
				Self::deposit_event(Event::<T>::Connect(
					sender,
					destination,
					ConnectionStatus::Rejected,
				));
			}

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_connection(
			origin: OriginFor<T>,
			destination: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let key = (&sender, &destination);

			let connection: ConnectionStatus = Self::connected(key);

			ensure!(connection == ConnectionStatus::Connected, Error::<T>::ConnectionIsNotPending);

			let key_reversed = (&destination, &sender);
			let new_sender_connection_cnt =
				Self::connected_cnt(&sender).checked_sub(1).ok_or(Error::<T>::StorageOverflow)?;
			let new_destination_connection_cnt = Self::connected_cnt(&destination)
				.checked_sub(1)
				.ok_or(Error::<T>::StorageOverflow)?;

			<Connected<T>>::remove(key);
			<Connected<T>>::remove(key_reversed);
			<ConnectedCnt<T>>::insert(&sender, new_sender_connection_cnt);
			<ConnectedCnt<T>>::insert(&destination, new_destination_connection_cnt);
			Self::deposit_event(Event::<T>::RemoveConnection(sender, destination));

			Ok(().into())
		}
	}
}
