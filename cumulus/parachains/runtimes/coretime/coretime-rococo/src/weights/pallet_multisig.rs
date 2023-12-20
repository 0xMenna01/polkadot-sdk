// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-12-07, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `dagda.local`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("coretime-rococo-dev")`, DB CACHE: 1024

// Executed Command:
// target/release/polkadot-parachain
// benchmark
// pallet
// --chain=coretime-rococo-dev
// --wasm-execution=compiled
// --pallet=pallet_multisig
// --extrinsic=*
// --steps=2
// --repeat=1
// --json
// --header=./cumulus/file_header.txt
// --output=./cumulus/parachains/runtimes/coretime/coretime-rococo/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(_z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 13_000_000 picoseconds.
		Weight::from_parts(16_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(_s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `172 + s * (3 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 35_000_000 picoseconds.
		Weight::from_parts(36_530_612, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 259
			.saturating_add(Weight::from_parts(1_650, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[3, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `282`
		//  Estimated: `6811`
		// Minimum execution time: 21_000_000 picoseconds.
		Weight::from_parts(18_422_680, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 8_928
			.saturating_add(Weight::from_parts(25_773, 0).saturating_mul(s.into()))
			// Standard Error: 86
			.saturating_add(Weight::from_parts(1_250, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(_s: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `315 + s * (34 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 53_000_000 picoseconds.
		Weight::from_parts(56_571_428, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			// Standard Error: 86
			.saturating_add(Weight::from_parts(150, 0).saturating_mul(z.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_create(_s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `172 + s * (3 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(35_000_000, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn approve_as_multi_approve(_s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `282`
		//  Estimated: `6811`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(21_000_000, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Multisig::Multisigs` (r:1 w:1)
	/// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3346), added: 5821, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[2, 100]`.
	fn cancel_as_multi(_s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `379 + s * (2 ±0)`
		//  Estimated: `6811`
		// Minimum execution time: 31_000_000 picoseconds.
		Weight::from_parts(40_000_000, 0)
			.saturating_add(Weight::from_parts(0, 6811))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
