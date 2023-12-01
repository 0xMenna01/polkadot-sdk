// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Extrinsics implementing the relay chain side of the Coretime interface.
//!
//! <https://github.com/polkadot-fellows/RFCs/blob/main/text/0005-coretime-interface.md>

// #[cfg(test)]
// mod tests;

use frame_support::{pallet_prelude::*, traits::Currency, traits::OriginTrait};
use frame_system::pallet_prelude::*;
use pallet_broker::{CoreAssignment, CoreIndex as BrokerCoreIndex};
use primitives::{CoreIndex, Id as ParaId};
use runtime_parachains::{
	configuration::pallet::Pallet as ConfigurationPallet;
	assigner_bulk::{self, PartsOf57600},
	origin::{ensure_parachain, Origin},
};

use sp_runtime::traits::BadOrigin;
use sp_std::{prelude::*, result};

pub use pallet::*;

pub trait WeightInfo {
	fn request_core_count() -> Weight;
	fn request_revenue_info_at() -> Weight;
	fn credit_account() -> Weight;
	fn assign_core() -> Weight;
}

/// A weight info that is only suitable for testing.
pub struct TestWeightInfo;

impl WeightInfo for TestWeightInfo {
	fn request_core_count() -> Weight {
		Weight::MAX
	}
	fn request_revenue_info_at() -> Weight {
		Weight::MAX
	}
	fn credit_account() -> Weight {
		Weight::MAX
	}
	fn assign_core() -> Weight {
		Weight::MAX
	}
}

/// Shorthand for the Balance type the runtime is using.
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + assigner_bulk::Config {
		type RuntimeOrigin: From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<result::Result<Origin, <Self as Config>::RuntimeOrigin>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The runtime's definition of a Currency.
		type Currency: Currency<Self::AccountId>;
		/// Something that provides the weight of this pallet.
		//type WeightInfo: WeightInfo;
		/// The ParaId of the broker system parachain.
		#[pallet::constant]
		type BrokerId: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The broker chain has asked for revenue information for a specific block.
		RevenueInfoRequested { when: BlockNumberFor<T> },
		/// A core has received a new assignment from the broker chain.
		CoreAssigned { core: CoreIndex },
	}

	#[pallet::origin]
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Origin {
		/// Coretime origin, used when setting `bulk_core_count` in the `HostConfiguration`
		Coretime,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The paraid making the call is not the coretime brokerage system parachain.
		NotBroker,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO Impl me!
		//#[pallet::weight(<T as Config>::WeightInfo::request_core_count())]
		#[pallet::call_index(1)]
		pub fn request_core_count(origin: OriginFor<T>, count: u16) -> DispatchResult {
			// Ignore requests not coming from the broker parachain or root.
			Self::ensure_root_or_para(origin, <T as Config>::BrokerId::get().into())?;
			let coretime_origin = <T as Config>::RuntimeOrigin::from(Origin::Coretime)?;
			ConfigurationPallet::set_bulk_core_count(coretime_origin as <T as Config>::RuntimeOrigin, count)?
			Ok(())
		}

		// TODO Impl me!
		//#[pallet::weight(<T as Config>::WeightInfo::request_revenue_info_at())]
		#[pallet::call_index(2)]
		pub fn request_revenue_info_at(
			origin: OriginFor<T>,
			_when: BlockNumberFor<T>,
		) -> DispatchResult {
			// Ignore requests not coming from the broker parachain or root.
			Self::ensure_root_or_para(origin, <T as Config>::BrokerId::get().into())?;
			Ok(())
		}

		// TODO Impl me!
		//#[pallet::weight(<T as Config>::WeightInfo::credit_account())]
		#[pallet::call_index(3)]
		pub fn credit_account(
			origin: OriginFor<T>,
			_who: T::AccountId,
			_amount: BalanceOf<T>,
		) -> DispatchResult {
			// Ignore requests not coming from the broker parachain or root.
			Self::ensure_root_or_para(origin, <T as Config>::BrokerId::get().into())?;
			Ok(())
		}

		/// Receive instructions from the `ExternalBrokerOrigin`, detailing how a specific core is
		/// to be used.
		///
		/// Parameters:
		/// -`origin`: The `ExternalBrokerOrigin`, assumed to be the Broker system parachain.
		/// -`core`: The core that should be scheduled.
		/// -`begin`: The starting blockheight of the instruction.
		/// -`assignment`: How the blockspace should be utilised.
		/// -`end_hint`: An optional hint as to when this particular set of instructions will end.
		// The broker pallet's `CoreIndex` definition is `u16` but on the relay chain it's `struct
		// CoreIndex(u32)`
		// TODO: Weights!
		//#[pallet::weight(<T as Config>::WeightInfo::assign_core())]
		#[pallet::call_index(4)]
		pub fn assign_core(
			origin: OriginFor<T>,
			core: BrokerCoreIndex,
			begin: BlockNumberFor<T>,
			assignment: Vec<(CoreAssignment, PartsOf57600)>,
			end_hint: Option<BlockNumberFor<T>>,
		) -> DispatchResult {
			// Ignore requests not coming from the broker parachain or root.
			Self::ensure_root_or_para(origin, <T as Config>::BrokerId::get().into())?;

			<assigner_bulk::Pallet<T>>::assign_core(
				// Relay chain `CoreIndex` implements `From` for `u32`
				u32::from(core).into(),
				begin,
				assignment,
				end_hint,
			)
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Ensure the origin is one of Root or the `para` itself.
	fn ensure_root_or_para(
		origin: <T as frame_system::Config>::RuntimeOrigin,
		id: ParaId,
	) -> DispatchResult {
		if let Ok(caller_id) = ensure_parachain(<T as Config>::RuntimeOrigin::from(origin.clone()))
		{
			// Check if matching para id...
			ensure!(caller_id == id, Error::<T>::NotBroker);
		} else {
			// Check if root...
			ensure_root(origin.clone())?;
		}
		Ok(())
	}
}

/// `EnsureOrigin` implementation for `Origin::Coretime`
pub struct EnsureCoretime;
impl<O: OriginTrait + From<Origin>> EnsureOrigin<O> for EnsureCoretime {
	type Success = ();
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			Origin::Coretime => Ok(()),
			r => Err(O::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		Ok(O::from(Origin::Coretime))
	}
}
