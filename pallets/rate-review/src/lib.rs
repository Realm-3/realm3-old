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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, WeakBoundedVec};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::{vec, vec::Vec};

	const MAX_RATE: u8 = 5;
	const LIKES_SEGMENT_LENGTH: u32 = 256;

	type Rate = u8;
	type Review<AccountId, MaxReviewLength> =
		(AccountId, Rate, WeakBoundedVec<u8, MaxReviewLength>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxReviewLength: Get<u32>;

		#[pallet::constant]
		type MaxReplyLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn property_review)]
	pub type PropertyReview<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		Vec<Review<T::AccountId, T::MaxReviewLength>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pro_review)]
	pub type ProReview<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<Review<T::AccountId, T::MaxReviewLength>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn property_rate)]
	pub type PropertyRate<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Rate, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pro_rate)]
	pub type ProRate<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Rate, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn property_reviews_cnt)]
	pub type PropertyReviewsCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pro_reviews_cnt)]
	pub type ProReviewsCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn property_reviews_likes)]
	pub type PropertyReviewLikes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		WeakBoundedVec<T::AccountId, ConstU32<LIKES_SEGMENT_LENGTH>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pro_reviews_likes)]
	pub type ProReviewsLikes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		WeakBoundedVec<T::AccountId, ConstU32<LIKES_SEGMENT_LENGTH>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn property_reviews_likes_cnt)]
	pub type PropertyReviewsLikesCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pro_reviews_likes_cnt)]
	pub type ProReviewsLikesCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn property_replies)]
	pub type PropertyReplies<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::Hash, T::AccountId),
		(T::AccountId, WeakBoundedVec<u8, T::MaxReplyLength>),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pro_replies)]
	pub type ProReplies<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, T::AccountId),
		(T::AccountId, WeakBoundedVec<u8, T::MaxReplyLength>),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn property_reply_cnt)]
	pub type PropertyReplyCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pro_reply_cnt)]
	pub type ProReplyCnt<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [property, who]
		PropertyReviewCreated(T::Hash, T::AccountId),
		/// [pro, who]
		ProReviewCreated(T::AccountId, T::AccountId),
		/// [property, who]
		PropertyReviewLikeCreated(T::Hash, T::AccountId),
		/// [pro, who]
		ProReviewLikeCreated(T::AccountId, T::AccountId),
		/// [pro, replied, who]
		PropertyReplyCreated(T::Hash, T::AccountId, T::AccountId),
		/// [pro, replied, who]
		ProReplyCreated(T::AccountId, T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidRate,
		PropertyReviewAlreadyCreated,
		ProReviewAlreadyCreated,
		PropertyLikeLimitReached,
		ProLikeLimitReached,
		PropertyReviewsOverflow,
		ProReviewsOverflow,
		PropertyReviewAlreadyLiked,
		ProReviewAlreadyLiked,
		PropertyReviewsLikesOverflow,
		ProReviewsLikesOverflow,
		PropertyReplyOverflow,
		ProReplyOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// -------------------- Property
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_property_review(
			origin: OriginFor<T>,
			property_id: T::Hash,
			rate: Rate,
			review: WeakBoundedVec<u8, T::MaxReviewLength>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(rate <= MAX_RATE, Error::<T>::InvalidRate);

			let mut reviews = Self::get_property_reviews(&property_id);

			ensure!(
				reviews.clone().iter().find(|&x| x.0 == sender).is_none(),
				Error::<T>::PropertyReviewAlreadyCreated
			);

			reviews.push((sender.clone(), rate, review));

			let new_rate = Self::add_to_rate(
				Self::property_rate(&property_id),
				rate,
				Self::property_reviews_cnt(&property_id),
			);
			let new_cnt = Self::property_reviews_cnt(&property_id)
				.checked_add(1)
				.ok_or(Error::<T>::PropertyReviewsOverflow)?;

			PropertyReview::<T>::insert(&property_id, reviews);
			PropertyRate::<T>::insert(&property_id, new_rate);
			PropertyReviewsCnt::<T>::insert(&property_id, new_cnt);
			Self::deposit_event(Event::<T>::PropertyReviewCreated(property_id, sender));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn like_property_review(origin: OriginFor<T>, property_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let likes = PropertyReviewLikes::<T>::get(&property_id);

			ensure!(!likes.contains(&sender), Error::<T>::PropertyReviewAlreadyLiked);

			let mutate_result =
				PropertyReviewLikes::<T>::try_mutate(&property_id, |x| x.try_push(sender.clone()));

			ensure!(mutate_result.is_ok(), Error::<T>::PropertyLikeLimitReached);

			let new_cnt = Self::property_reviews_likes_cnt(&property_id)
				.checked_add(1)
				.ok_or(Error::<T>::PropertyReviewsLikesOverflow)?;

			PropertyReviewsLikesCnt::<T>::insert(&property_id, new_cnt);
			Self::deposit_event(Event::<T>::PropertyReviewLikeCreated(property_id, sender));

			Ok(())
		}

		// -------------------- Pro
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_pro_review(
			origin: OriginFor<T>,
			pro_id: T::AccountId,
			rate: Rate,
			review: WeakBoundedVec<u8, T::MaxReviewLength>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(rate <= MAX_RATE, Error::<T>::InvalidRate);

			let mut reviews = Self::get_pro_reviews(&pro_id);

			ensure!(
				reviews.clone().iter().find(|&x| x.0 == sender).is_none(),
				Error::<T>::ProReviewAlreadyCreated
			);

			reviews.push((sender.clone(), rate, review));

			let new_rate =
				Self::add_to_rate(Self::pro_rate(&pro_id), rate, Self::pro_reviews_cnt(&pro_id));
			let new_cnt = Self::pro_reviews_cnt(&pro_id)
				.checked_add(1)
				.ok_or(Error::<T>::ProReviewsOverflow)?;

			ProReview::<T>::insert(&pro_id, reviews);
			ProRate::<T>::insert(&pro_id, new_rate);
			ProReviewsCnt::<T>::insert(&pro_id, new_cnt);
			Self::deposit_event(Event::<T>::ProReviewCreated(pro_id, sender));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn like_pro_review(origin: OriginFor<T>, pro_id: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let likes = ProReviewsLikes::<T>::get(&pro_id);

			ensure!(!likes.contains(&sender), Error::<T>::ProReviewAlreadyLiked);

			let mutate_result =
				ProReviewsLikes::<T>::try_mutate(&pro_id, |x| x.try_push(sender.clone()));

			ensure!(mutate_result.is_ok(), Error::<T>::ProLikeLimitReached);

			let new_cnt = Self::pro_reviews_likes_cnt(&pro_id)
				.checked_add(1)
				.ok_or(Error::<T>::ProReviewsLikesOverflow)?;

			ProReviewsLikesCnt::<T>::insert(&pro_id, new_cnt);
			Self::deposit_event(Event::<T>::ProReviewLikeCreated(pro_id, sender));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_property_reviews(
			property_id: &T::Hash,
		) -> Vec<Review<T::AccountId, T::MaxReviewLength>> {
			PropertyReview::<T>::get(property_id).unwrap_or(vec![])
		}

		fn get_pro_reviews(pro_id: &T::AccountId) -> Vec<Review<T::AccountId, T::MaxReviewLength>> {
			ProReview::<T>::get(pro_id).unwrap_or(vec![])
		}

		fn add_to_rate(current_rate: Rate, new_rate: Rate, total_rates: u32) -> Rate {
			(((total_rates * current_rate as u32) + new_rate as u32) / (total_rates + 1_u32))
				as Rate
		}
	}
}
