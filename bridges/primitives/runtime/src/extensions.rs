// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Primitives that may be used for creating signed extensions for indirect runtimes.

use codec::{Compact, Decode, Encode};
use impl_trait_for_tuples::impl_for_tuples;
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_runtime::{
	traits::{DispatchInfoOf, TransactionExtension},
	transaction_validity::{TransactionValidityError, ValidTransaction},
};
use sp_std::{fmt::Debug, marker::PhantomData};

/// Trait that describes some properties of a `TransactionExtension` that are needed in order to
/// send a transaction to the chain.
pub trait TransactionExtensionSchema:
	Encode + Decode + Debug + Eq + Clone + StaticTypeInfo
{
	/// A type of the data encoded as part of the transaction.
	type Payload: Encode + Decode + Debug + Eq + Clone + StaticTypeInfo;
	/// Parameters which are part of the payload used to produce transaction signature,
	/// but don't end up in the transaction itself (i.e. inherent part of the runtime).
	type AdditionalSigned: Encode + Debug + Eq + Clone + StaticTypeInfo;
}

impl TransactionExtensionSchema for () {
	type Payload = ();
	type AdditionalSigned = ();
}

/// An implementation of `TransactionExtensionSchema` using generic params.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo)]
pub struct GenericTransactionExtensionSchema<P, S>(PhantomData<(P, S)>);

impl<P, S> TransactionExtensionSchema for GenericTransactionExtensionSchema<P, S>
where
	P: Encode + Decode + Debug + Eq + Clone + StaticTypeInfo,
	S: Encode + Debug + Eq + Clone + StaticTypeInfo,
{
	type Payload = P;
	type AdditionalSigned = S;
}

/// The `TransactionExtensionSchema` for `frame_system::CheckNonZeroSender`.
pub type CheckNonZeroSender = GenericTransactionExtensionSchema<(), ()>;

/// The `TransactionExtensionSchema` for `frame_system::CheckSpecVersion`.
pub type CheckSpecVersion = GenericTransactionExtensionSchema<(), u32>;

/// The `TransactionExtensionSchema` for `frame_system::CheckTxVersion`.
pub type CheckTxVersion = GenericTransactionExtensionSchema<(), u32>;

/// The `TransactionExtensionSchema` for `frame_system::CheckGenesis`.
pub type CheckGenesis<Hash> = GenericTransactionExtensionSchema<(), Hash>;

/// The `TransactionExtensionSchema` for `frame_system::CheckEra`.
pub type CheckEra<Hash> = GenericTransactionExtensionSchema<sp_runtime::generic::Era, Hash>;

/// The `TransactionExtensionSchema` for `frame_system::CheckNonce`.
pub type CheckNonce<TxNonce> = GenericTransactionExtensionSchema<Compact<TxNonce>, ()>;

/// The `TransactionExtensionSchema` for `frame_system::CheckWeight`.
pub type CheckWeight = GenericTransactionExtensionSchema<(), ()>;

/// The `TransactionExtensionSchema` for `pallet_transaction_payment::ChargeTransactionPayment`.
pub type ChargeTransactionPayment<Balance> =
	GenericTransactionExtensionSchema<Compact<Balance>, ()>;

/// The `TransactionExtensionSchema` for `polkadot-runtime-common::PrevalidateAttests`.
pub type PrevalidateAttests = GenericTransactionExtensionSchema<(), ()>;

/// The `TransactionExtensionSchema` for `BridgeRejectObsoleteHeadersAndMessages`.
pub type BridgeRejectObsoleteHeadersAndMessages = GenericTransactionExtensionSchema<(), ()>;

/// The `TransactionExtensionSchema` for `RefundBridgedParachainMessages`.
/// This schema is dedicated for `RefundBridgedParachainMessages` transaction extension as
/// wildcard/placeholder, which relies on the scale encoding for `()` or `((), ())`, or `((), (),
/// ())` is the same. So runtime can contains any kind of tuple:
/// `(BridgeRefundBridgeHubRococoMessages)`
/// `(BridgeRefundBridgeHubRococoMessages, BridgeRefundBridgeHubWococoMessages)`
/// `(BridgeRefundParachainMessages1, ..., BridgeRefundParachainMessagesN)`
pub type RefundBridgedParachainMessagesSchema = GenericTransactionExtensionSchema<(), ()>;

#[impl_for_tuples(1, 12)]
impl TransactionExtensionSchema for Tuple {
	for_tuples!( type Payload = ( #( Tuple::Payload ),* ); );
	for_tuples!( type AdditionalSigned = ( #( Tuple::AdditionalSigned ),* ); );
}

/// A simplified version of transaction extensions meant for producing transaction transactions
/// and signed payloads in the client code.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct GenericTransactionExtension<S: TransactionExtensionSchema> {
	pub payload: S::Payload,
	#[codec(skip)]
	// It may be set to `None` if extensions are decoded. We are never reconstructing transactions
	// (and it makes no sense to do that) => decoded version of `TransactionExtensions` is only
	// used to read fields of the `payload`. And when resigning transaction, we're reconstructing
	// `TransactionExtensions` from scratch.
	additional_signed: Option<S::AdditionalSigned>,
}

impl<S: TransactionExtensionSchema> GenericTransactionExtension<S> {
	pub fn new(payload: S::Payload, additional_signed: Option<S::AdditionalSigned>) -> Self {
		Self { payload, additional_signed }
	}
}

impl<S> TransactionExtension for GenericTransactionExtension<S>
where
	S: TransactionExtensionSchema,
	S::Payload: Send + Sync,
	S::AdditionalSigned: Send + Sync,
{
	const IDENTIFIER: &'static str = "Not needed.";
	type Call = ();
	type AdditionalSigned = S::AdditionalSigned;
	type Val = ();
	type Pre = ();

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		// we shall not ever see this error in relay, because we are never signing decoded
		// transactions. Instead we're constructing and signing new transactions. So the error code
		// is kinda random here
		self.additional_signed.clone().ok_or(
			frame_support::unsigned::TransactionValidityError::Unknown(
				frame_support::unsigned::UnknownTransaction::Custom(0xFF),
			),
		)
	}

	fn validate(
		&self,
		_origin: (),
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<(ValidTransaction, (), ()), TransactionValidityError> {
		Ok((Default::default(), (), ()))
	}

	fn prepare(
		self,
		_val: (),
		_origin: &(),
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		Ok(())
	}
}
