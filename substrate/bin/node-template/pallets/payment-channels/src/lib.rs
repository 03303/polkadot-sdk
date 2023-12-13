//! # Payment Channels Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//!
//!

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use frame_system::WeightInfo;

use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;

use frame_support::pallet_prelude::*;
use frame_support::PalletId;
use frame_support::traits::{Currency, Len, ReservableCurrency, ExistenceRequirement::AllowDeath};
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;
use sp_runtime::{
	Saturating,
	traits::{AccountIdConversion, IdentifyAccount, Verify, Zero}
};

pub use pallet::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type HashId<T> = <T as frame_system::Config>::Hash;
	pub type OrganizationId<T> = HashId<T>;
	pub type ServiceId<T> = HashId<T>;
	pub type ChannelId<T> = HashId<T>;

	pub type MemberRankId = u32;

	pub type NameVec<T> = BoundedVec<u8, <T as Config>::MaxNameLength>;
	pub type MetadataVec<T> = BoundedVec<u8, <T as Config>::MaxMetadataLength>;

	/// OrganizationSpecs(owner, hash)
	pub type OrganizationSpecs<T> = (<T as frame_system::Config>::AccountId, HashId<T>);
	pub type ServiceSpecs<T> = (OrganizationSpecs<T>, HashId<T>);
	pub type ChannelSpecs<T> = (<T as frame_system::Config>::AccountId, HashId<T>);

	#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Organization<Hash, AccountId, NameVec, MetadataVec> {
		id: Hash,
		owner: AccountId,
		services: u32,
		name: NameVec,
		members: u32,
		metadata: MetadataVec,
	}

	#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Service<Hash, AccountId, NameVec, MetadataVec, Balance, BlockNumber> {
		id: Hash,
		owner: AccountId,
		organization: Hash,
		channels: u32,
		name: NameVec,
		version: u32,
		metadata: MetadataVec,
		price: Balance,
		minimum_calls: u32,
		expiration_threshold: BlockNumber,
		trials: u32,
	}

	#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Channel<Hash, AccountId, Balance, BlockNumber> {
		id: Hash,
		owner: AccountId,
		organization: Hash,
		service: Hash,
		version: u32,
		counter: u32,
		price: Balance,
		calls: u32,
		expiration: BlockNumber,
	}

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type MaxOrganizationMembers: Get<u32>;
		#[pallet::constant]
		type MaxNameLength: Get<u32>;
		#[pallet::constant]
		type MaxMetadataLength: Get<u32>;

		type Currency: ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type OrganizationDeposit: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type ServiceDeposit: Get<BalanceOf<Self>>;

		type Signature: Verify<Signer = Self::Signer> + Parameter;
		type Signer: IdentifyAccount<AccountId = Self::AccountId>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn organizations)]
	pub(super) type Organizations<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		OrganizationId<T>,
		Organization<HashId<T>, T::AccountId, NameVec<T>, MetadataVec<T>>,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub(super) type OrganizationMembers<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		OrganizationId<T>,
		Twox64Concat,
		T::AccountId,
		MemberRankId,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub(super) type Services<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		OrganizationId<T>,
		Twox64Concat,
		ServiceId<T>,
		Service<
			HashId<T>,
			T::AccountId,
			NameVec<T>,
			MetadataVec<T>,
			BalanceOf<T>,
			BlockNumberFor<T>
		>,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn channels)]
	pub(super) type Channels<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		ChannelId<T>,
		Channel<HashId<T>, T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
		OptionQuery
	>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrganizationCreated { id: OrganizationId<T>, owner: T::AccountId, members: u32 },
		OrganizationDeleted { id: OrganizationId<T>, owner: T::AccountId },

		ServiceCreated { id: HashId<T>, owner: T::AccountId, organization: HashId<T>, price: BalanceOf<T> },
		ServiceDeleted { id: HashId<T>, owner: T::AccountId, organization: HashId<T> },
		ServiceUpdated { id: HashId<T>, owner: T::AccountId, organization: HashId<T>, version: u32 },

		ChannelCreated {
			id: HashId<T>,
			owner: T::AccountId,
			organization: HashId<T>,
			service: HashId<T>,
			version: u32,
			calls: u32,
			funds: BalanceOf<T>,
			expiration: BlockNumberFor<T>,
		},
		ChannelUpdated {
			id: HashId<T>,
			owner: T::AccountId,
			organization: HashId<T>,
			service: HashId<T>,
			version: u32,
			calls: u32,
			funds: BalanceOf<T>,
			expiration: BlockNumberFor<T>,
		},
		ChannelExpiredClaimed { id: ChannelId<T>, by: T::AccountId, funds: BalanceOf<T> },
		ChannelDeleted { id: ChannelId<T>, by: T::AccountId, funds: BalanceOf<T> },
		ChannelClaimed {
			id: ChannelId<T>,
			by: T::AccountId,
			counter: u32,
			funds: BalanceOf<T>
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		OrganizationExists,
		OrganizationNotFound,
		OrganizationNotOwner,

		ServiceExists,
		ServiceNotFound,
		ServiceNotOwner,
		ServiceNotOrgMember,

		ChannelExists,
		ChannelNotFound,
		ChannelNotOwner,
		ChannelLowNumberOfCalls,
		ChannelInvalidExpiration,

		ClaimNotAllowed,
		ClaimNotExpired,
		ClaimLowCounter,
		ClaimNotEnoughFunds,
		ClaimInvalidSigner,
		ClaimInvalidSignature,

		InvalidBlockNumber,
		InsufficientFunds,
	}

	/// The pallet's dispatchable functions ([`Call`]s).
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000_000)]
		pub fn create_organization(
			origin: OriginFor<T>,
			name: NameVec<T>,
			members: Option<Vec<T::AccountId>>,
			metadata: MetadataVec<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			T::Currency::reserve(&owner, T::OrganizationDeposit::get())
				.map_err(|_| Error::<T>::InsufficientFunds)?;

			let organization_id = Self::hash_name(owner.clone(), name.clone());

			ensure!(!Organizations::<T>::contains_key(&owner, &organization_id), Error::<T>::OrganizationExists);

			if let Some(members) = &members {
				for member in members {
					OrganizationMembers::<T>::insert(organization_id.clone(), member, 1);
				}
			}

			let mut members_count = 0;
			for _ in OrganizationMembers::<T>::iter_key_prefix(organization_id.clone()) {
				members_count += 1;
			}

			let organization = Organization {
				id: organization_id.clone(),
				owner: owner.clone(),
				services: 0,
				name,
				metadata,
				members: members_count.clone(),
			};

			Organizations::<T>::insert(owner.clone(), organization_id.clone(), organization);
			OrganizationMembers::<T>::insert(organization_id.clone(), owner.clone(), 0);

			Self::deposit_event(Event::OrganizationCreated { id: organization_id, owner, members: members_count });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000_000)]
		pub fn delete_organization(
			origin: OriginFor<T>,
			name: NameVec<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let organization_id = Self::hash_name(owner.clone(), name.clone());

			let organization = Organizations::<T>::get(
				owner.clone(), organization_id.clone()).ok_or(Error::<T>::OrganizationNotFound)?;

			ensure!(owner == organization.owner, Error::<T>::OrganizationNotOwner);

			T::Currency::unreserve(&owner, T::OrganizationDeposit::get());

			Organizations::<T>::remove(owner.clone(), organization_id.clone());
			OrganizationMembers::<T>::drain_prefix(organization_id.clone());

			Self::deposit_event(Event::OrganizationDeleted { id: organization_id, owner });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000_000)]
		pub fn create_service(
			origin: OriginFor<T>,
			organization: OrganizationSpecs<T>,
			name: NameVec<T>,
			price: BalanceOf<T>,
			minimum_calls: u32,
			expiration_threshold: BlockNumberFor<T>,
			trials: u32,
			metadata: MetadataVec<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let (org_owner, organization_id) = organization;

			let mut organization = Organizations::<T>::get(
				org_owner.clone(), organization_id.clone()).ok_or(Error::<T>::OrganizationNotFound)?;

			if let Some(_rank) = OrganizationMembers::<T>::get(organization_id.clone(), owner.clone()) {

				T::Currency::reserve(&owner, T::ServiceDeposit::get())
					.map_err(|_| Error::<T>::InsufficientFunds)?;

				let service_id = Self::hash_name(owner.clone(), name.clone());

				ensure!(!Services::<T>::contains_key(&organization_id, &service_id), Error::<T>::ServiceExists);

				organization.services += 1;

				let service = Service {
					id: service_id.clone(),
					owner: owner.clone(),
					channels: 0,
					organization: organization_id.clone(),
					name,
					version: 1,
					price: price.clone(),
					minimum_calls,
					expiration_threshold,
					trials,
					metadata,
				};

				Organizations::<T>::insert(org_owner.clone(), organization_id.clone(), organization);
				Services::<T>::insert(organization_id.clone(), service_id.clone(), service);

				Self::deposit_event(Event::ServiceCreated { id: service_id, owner, organization: organization_id.clone(), price });
			} else {
				return Err(Error::<T>::ServiceNotOrgMember.into());
			}
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000_000)]
		pub fn delete_service(
			origin: OriginFor<T>,
			service: ServiceSpecs<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let ((org_owner, organization_id), service_id) = service;

			let mut organization = Organizations::<T>::get(
				org_owner.clone(),
				organization_id.clone()
			).ok_or(Error::<T>::OrganizationNotFound)?;
			organization.services -= 1;

			let service = Services::<T>::get(
				organization_id.clone(),
				service_id.clone()
			).ok_or(Error::<T>::ServiceNotFound)?;

			ensure!(owner == service.owner, Error::<T>::ServiceNotOwner);

			T::Currency::unreserve(&owner, T::ServiceDeposit::get());

			Organizations::<T>::insert(org_owner.clone(), organization_id.clone(), organization);
			Services::<T>::remove(organization_id.clone(), service_id.clone());

			Self::deposit_event(Event::ServiceDeleted { id: service_id, owner, organization: organization_id });
			Ok(())
		}

		/// This will invalidate all open channels for the current service (via version).
		/// Forcing users to also update their channels or open new ones.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000_000)]
		pub fn update_service(
			origin: OriginFor<T>,
			service: ServiceSpecs<T>,
			name: Option<NameVec<T>>,
			price: Option<BalanceOf<T>>,
			minimum_calls: Option<u32>,
			expiration_threshold: Option<BlockNumberFor<T>>,
			trials: Option<u32>,
			metadata: Option<MetadataVec<T>>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let ((_, organization_id), service_id) = service;

			let mut service = Services::<T>::get(
				organization_id.clone(),
				service_id.clone()
			).ok_or(Error::<T>::ServiceNotFound)?;

			ensure!(owner == service.owner, Error::<T>::ServiceNotOwner);

			service.name = name.unwrap_or(service.name);
			service.price = price.unwrap_or(service.price);
			service.minimum_calls = minimum_calls.unwrap_or(service.minimum_calls);
			service.expiration_threshold = expiration_threshold.unwrap_or(service.expiration_threshold);
			service.trials = trials.unwrap_or(service.trials);
			service.metadata = metadata.unwrap_or(service.metadata);

			let version = service.version + 1;
			service.version = version.clone();

			Services::<T>::insert(organization_id.clone(), service_id.clone(), service);
			Self::deposit_event(Event::ServiceUpdated { id: service_id, owner, organization: organization_id, version });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000_000)]
		pub fn open_channel(
			origin: OriginFor<T>,
			service: ServiceSpecs<T>,
			calls: u32,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let ((_, organization_id), service_id) = service;

			let mut service = Services::<T>::get(
				organization_id.clone(),
				service_id.clone()
			).ok_or(Error::<T>::ServiceNotFound)?;

			ensure!(calls >= service.minimum_calls, Error::<T>::ChannelLowNumberOfCalls);

			let channel_id = Self::hash_channel_id(
				owner.clone(),
				organization_id.clone(),
				service_id.clone(),
			);
			ensure!(!Channels::<T>::contains_key(&owner, &channel_id), Error::<T>::ChannelExists);

			let price = service.price.clone();
			let funds = price.saturating_mul(calls.into());

			T::Currency::transfer(&owner, &Self::account_id(), funds.into(), AllowDeath)?;

			service.channels += 1;

			let calls = calls.clone();

			let bn = frame_system::Pallet::<T>::block_number();
			let expiration = bn + service.expiration_threshold.clone();

			let channel = Channel {
				id: channel_id.clone(),
				owner: owner.clone(),
				organization: organization_id.clone(),
				service: service_id.clone(),
				version: service.version.clone(),
				counter: 0,
				price: price.clone(),
				calls: calls.clone(),
				expiration: expiration.clone(),
			};

			Services::<T>::insert(organization_id.clone(), service_id.clone(), service.clone());
			Channels::<T>::insert(owner.clone(), channel_id.clone(), channel);

			Self::deposit_event(Event::ChannelCreated {
				id: channel_id,
				owner,
				organization: organization_id,
				service: service_id,
				version: service.version,
				funds,
				calls,
				expiration
			});

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000_000)]
		pub fn update_channel(
			origin: OriginFor<T>,
			channel: ChannelSpecs<T>,
			calls: Option<u32>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let (channel_owner, channel_id) = channel;

			ensure!(owner == channel_owner, Error::<T>::ChannelNotOwner);

			let mut channel = Channels::<T>::get(
				channel_owner.clone(), channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			let (organization_id, service_id) = (channel.organization.clone(), channel.service.clone());

			let service = Services::<T>::get(
				organization_id.clone(), service_id.clone()).ok_or(Error::<T>::ServiceNotFound)?;

			// Returning funds to the Channel's owner
			let price = channel.price.clone();
			let mut remaining = BalanceOf::<T>::default();
			if channel.counter <= channel.calls {
				remaining = price.saturating_mul((channel.calls.clone() - channel.counter.clone()).into());
			}
			if !remaining.is_zero() {
				T::Currency::transfer(&Self::account_id(), &owner, remaining.into(), AllowDeath)?;
			}

			let calls = calls.unwrap_or(service.minimum_calls);
			ensure!(calls >= service.minimum_calls, Error::<T>::ChannelLowNumberOfCalls);

			let price = service.price.clone();

			let bn = frame_system::Pallet::<T>::block_number();
			let expiration = bn + service.expiration_threshold.clone();

			channel.version = service.version.clone();
			channel.price = price.clone();
			channel.calls = calls.clone();
			channel.expiration = expiration.clone();

			// Funding the Channel
			let funds = price.saturating_mul(calls.into());
			T::Currency::transfer(&owner, &Self::account_id(), funds.into(), AllowDeath)?;

			Self::deposit_event(Event::ChannelUpdated {
				id: channel_id,
				owner,
				organization: organization_id,
				service: service_id,
				version: service.version,
				funds,
				calls,
				expiration
			});

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(10_000_000)]
		pub fn claim_channel_funds(
			origin: OriginFor<T>,
			channel: ChannelSpecs<T>,
			counter: Option<u32>,
			signature: Option<T::Signature>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let (channel_owner, channel_id) = channel;

			let mut channel = Channels::<T>::get(
				channel_owner.clone(), channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			let (organization_id, service_id) = (channel.organization.clone(), channel.service.clone());

			let mut service = Services::<T>::get(
				organization_id.clone(), service_id.clone()).ok_or(Error::<T>::ServiceNotFound)?;

			let bn = frame_system::Pallet::<T>::block_number();

			let is_expired = channel.expiration <= bn;
			let version_changed = channel.version != service.version;

			let counter = counter.unwrap_or_default();

			let price = channel.price.clone();

			let mut remaining = BalanceOf::<T>::default();
			if channel.counter <= channel.calls {
				remaining = price.saturating_mul((channel.calls.clone() - channel.counter.clone()).into());
			}

			// There is no more funds to be claimed, delete the Channel and update the Service.
			if remaining.is_zero() {
				service.channels -= 1;
				Channels::<T>::remove(channel_owner.clone(), channel_id.clone());
				Services::<T>::insert(organization_id.clone(), service_id.clone(), service);
				Self::deposit_event(Event::ChannelDeleted { id: channel_id.clone(), by: owner.clone(), funds: remaining });
				return Ok(());
			}

			// By channel owner, channel must be expired or service has changed version.
			if owner == channel.owner {
				if is_expired || version_changed {
					T::Currency::transfer(&Self::account_id(), &owner, remaining.into(), AllowDeath)?;
					service.channels -= 1;
					Channels::<T>::remove(channel_owner.clone(), channel_id.clone());
					Services::<T>::insert(organization_id.clone(), service_id.clone(), service);
					Self::deposit_event(Event::ChannelExpiredClaimed { id: channel_id.clone(), by: owner.clone(), funds: remaining });
					return Ok(());
				} else if counter == 0 {
					return Err(Error::<T>::ClaimNotExpired.into());
				}
			}

			ensure!(counter > channel.counter, Error::<T>::ClaimLowCounter);

			let signer = channel.owner.clone();
			let signature = signature.ok_or(Error::<T>::ClaimInvalidSignature)?;

			let message = (
				b"modlpy/paych____",
				channel_id.clone(),
				service.version.clone(),
				counter.clone()
			).using_encoded(blake2_256);

			Self::validate_signature(&Encode::encode(&message), &signature, &signer)?;

			let mut claim = price.saturating_mul((counter.clone() - channel.counter.clone()).into());

			let max_claim = price.saturating_mul(channel.calls.clone().into());
			if claim > max_claim {
				claim = max_claim;
			}

			ensure!(claim <= remaining, Error::<T>::ClaimNotEnoughFunds);

			T::Currency::transfer(&Self::account_id(), &service.owner, claim.into(), AllowDeath)?;

			channel.counter = counter.clone();

			Channels::<T>::insert(channel_owner.clone(), channel_id.clone(), channel);
			Services::<T>::insert(organization_id.clone(), service_id.clone(), service);

			Self::deposit_event(Event::ChannelClaimed { id: channel_id, by: owner, counter, funds: claim });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
		pub fn hash_name(owner: T::AccountId, name: NameVec<T>) -> T::Hash {
			let hash = (
				b"modlpy/paych____",
				owner,
				name,
			).using_encoded(blake2_256);
			Decode::decode(&mut &hash[..]).expect("infinite length input; no invalid inputs for type; qed")
		}
		pub fn hash_channel_id(owner: T::AccountId, organization_id: T::Hash, service_id: T::Hash) -> T::Hash {
			let hash = (
				b"modlpy/paych____",
				owner,
				organization_id,
				service_id,
			).using_encoded(blake2_256);
			Decode::decode(&mut &hash[..]).expect("infinite length input; no invalid inputs for type; qed")
		}
		pub fn validate_signature(
			message: &Vec<u8>,
			signature: &T::Signature,
			signer: &T::AccountId,
		) -> DispatchResult {
			if signature.verify(&**message, &signer) {
				return Ok(())
			}

			// NOTE: for security reasons modern UIs implicitly wrap the data requested to sign into
			// <Bytes></Bytes>, that's why we support both wrapped and raw versions.
			let prefix = b"<Bytes>";
			let suffix = b"</Bytes>";
			let mut wrapped: Vec<u8> = Vec::with_capacity(message.len() + prefix.len() + suffix.len());
			wrapped.extend(prefix);
			wrapped.extend(message);
			wrapped.extend(suffix);

			ensure!(signature.verify(&*wrapped, &signer), Error::<T>::ClaimInvalidSignature);

			Ok(())
		}
	}
}
