#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch,
	ensure,
	traits::Get,
	StorageMap,
};
use frame_system::{self as system,ensure_signed};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// // The currency type that will be used to place deposits on nicks.
	// // It must implement ReservableCurrency.
	// // https://substrate.dev/rustdocs/v2.0.0/frame_support/traits/trait.ReservableCurrency.html
	// type Currency: ReservableCurrency<Self::AccountId>;

	// // The amount required to reserve a nick.
	// type ReservationFee: Get<BalanceOf<Self>>;

	// // A callback that will be invoked when a deposit is forfeited.
	// type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

	// // Origins are used to identify network participants and control access.
	// // This is used to identify the pallet's admin.
	// // https://substrate.dev/docs/en/knowledgebase/runtime/origin
	// type ForceOrigin: EnsureOrigin<Self::Origin>;

	// // This parameter is used to configure a nick's minimum length.
	// type MinLength: Get<usize>;

	// // This parameter is used to configure a nick's maximum length.
	// // https://substrate.dev/docs/en/knowledgebase/runtime/storage#create-bounds
	// type MaxLength: Get<usize>;

	// 附加题答案
	type MaxClaimLength: Get<u32>;


}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as PoeModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
		Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		/// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		/// The proof has already been claimed.
        ProofAlreadyClaimed,
        /// The proof does not exist, so it cannot be revoked.
        NoSuchProof,
        /// The proof is claimed by another account, so caller can't revoke it.
        NotProofOwner,
		NotOkProof,

		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		ProofTooLong,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		/// Allow a user to claim ownership of an unclaimed proof.
        #[weight = 10_000]
        pub fn create_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has not already been claimed.
            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyExist);

			// 第二题答案
			ensure!(T::MaxClaimLength::get() >= proof.len() as u32, Error::<T>::ProofTooLong);

            // Get the block number from the FRAME System module.
            let current_block = <frame_system::Module<T>>::block_number();

            // Store the proof with the sender and block number.
            Proofs::<T>::insert(&proof, (&sender, current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }

        /// Allow the owner to revoke their claim.
        #[weight = 10_000]
        pub fn revoke_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::ClaimNotExist);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&proof);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&proof);

            // Emit an event that the claim was erased.
            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }
        #[weight = 10_000]
        pub fn transfer_claim(origin, proof: Vec<u8>,dest:T::AccountId)->dispatch::DispatchResult{
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::ClaimNotExist);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&proof);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            Proofs::<T>::insert(&proof, (&dest, frame_system::Module::<T>::block_number()));
			Ok(())
        }
	}
}
