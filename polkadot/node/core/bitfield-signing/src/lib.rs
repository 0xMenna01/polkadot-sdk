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

//! The bitfield signing subsystem produces `SignedAvailabilityBitfield`s once per block.

#![deny(unused_crate_dependencies)]
#![warn(missing_docs)]
#![recursion_limit = "256"]

use futures::{
	channel::{mpsc, oneshot},
	future,
	lock::Mutex,
	FutureExt,
};
use polkadot_node_subsystem::{
	jaeger,
	messages::{AvailabilityStoreMessage, BitfieldDistributionMessage, ChainApiMessage},
	overseer, ActivatedLeaf, ChainApiError, FromOrchestra, LeafStatus, OverseerSignal, PerLeafSpan,
	SpawnedSubsystem, SubsystemError, SubsystemResult,
};
use polkadot_node_subsystem_util::{
	self as util, availability_chunk_index, request_validators, runtime::request_client_features,
	Validator,
};
use polkadot_primitives::{
	vstaging::ClientFeatures, AvailabilityBitfield, BlockNumber, CoreState, Hash, ValidatorIndex,
};
use sp_keystore::{Error as KeystoreError, KeystorePtr};
use std::{collections::HashMap, iter::FromIterator, time::Duration};
use util::{
	request_availability_cores, request_session_index_for_child, request_session_info,
	runtime::recv_runtime,
};
use wasm_timer::{Delay, Instant};

mod metrics;
use self::metrics::Metrics;

#[cfg(test)]
mod tests;

/// Delay between starting a bitfield signing job and its attempting to create a bitfield.
const SPAWNED_TASK_DELAY: Duration = Duration::from_millis(1500);
const LOG_TARGET: &str = "parachain::bitfield-signing";

// TODO: use `fatality` (https://github.com/paritytech/polkadot/issues/5540).
/// Errors we may encounter in the course of executing the `BitfieldSigningSubsystem`.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
	#[error(transparent)]
	Util(#[from] util::Error),

	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Oneshot(#[from] oneshot::Canceled),

	#[error(transparent)]
	MpscSend(#[from] mpsc::SendError),

	#[error(transparent)]
	Runtime(#[from] util::runtime::Error),

	#[error("Keystore failed: {0:?}")]
	Keystore(KeystoreError),

	#[error("Cannot find block number for given relay parent")]
	BlockNumberNotFound,

	#[error("Oneshot for receiving response from Chain API got cancelled")]
	ChainApiSenderDropped(#[source] oneshot::Canceled),

	#[error("Retrieving response from Chain API unexpectedly failed with error: {0}")]
	ChainApi(#[from] ChainApiError),
}

/// If there is a candidate pending availability, query the Availability Store
/// for whether we have the availability chunk for our validator index.
async fn get_core_availability(
	core: &CoreState,
	n_validators: usize,
	validator_index: ValidatorIndex,
	sender: &Mutex<&mut impl overseer::BitfieldSigningSenderTrait>,
	span: &jaeger::Span,
) -> Result<bool, Error> {
	if let CoreState::Occupied(core) = core {
		let _span = span.child("query-chunk-availability");
		let relay_parent = core.candidate_descriptor.relay_parent;

		let Ok(block_number) = get_block_number(*sender.lock().await, relay_parent).await else {
			gum::warn!(
				target: LOG_TARGET,
				?relay_parent,
				?core.candidate_hash,
				para_id = %core.para_id(),
				"Failed to get block number."
			);

			return Ok(false)
		};

		let maybe_client_features = request_client_features(relay_parent, *sender.lock().await)
			.await
			.map_err(Error::from)?;

		// Init this to all zeros. It won't be used unless
		// `ClientFeatures::AVAILABILITY_CHUNK_SHUFFLING` is enabled. We do this to avoid querying
		// the runtime API for session index and session info unless the feature is enabled.
		let mut babe_randomness = [0; 32];

		if let Some(client_features) = maybe_client_features {
			if client_features.contains(ClientFeatures::AVAILABILITY_CHUNK_SHUFFLING) {
				let session_index = recv_runtime(
					request_session_index_for_child(relay_parent, *sender.lock().await).await,
				)
				.await?;

				let Some(session_info) = recv_runtime(
					request_session_info(relay_parent, session_index, *sender.lock().await).await,
				)
				.await?
				else {
					gum::warn!(
						target: LOG_TARGET,
						?relay_parent,
						session_index,
						?core.candidate_hash,
						para_id = %core.para_id(),
						"Failed to get session info."
					);

					return Ok(false)
				};

				babe_randomness = session_info.random_seed;
			}
		}

		let chunk_index = availability_chunk_index(
			maybe_client_features.as_ref(),
			babe_randomness,
			n_validators,
			block_number,
			core.para_id(),
			validator_index,
		);

		let (tx, rx) = oneshot::channel();
		sender
			.lock()
			.await
			.send_message(AvailabilityStoreMessage::QueryChunkAvailability(
				core.candidate_hash,
				chunk_index,
				tx,
			))
			.await;

		let res = rx.await.map_err(Into::into);

		gum::trace!(
			target: LOG_TARGET,
			para_id = %core.para_id(),
			availability = ?res,
			?core.candidate_hash,
			"Candidate availability",
		);

		res
	} else {
		Ok(false)
	}
}

/// - get the list of core states from the runtime
/// - for each core, concurrently determine chunk availability (see `get_core_availability`)
/// - return the bitfield if there were no errors at any point in this process (otherwise, it's
///   prone to false negatives)
async fn construct_availability_bitfield(
	relay_parent: Hash,
	span: &jaeger::Span,
	n_validators: usize,
	validator_idx: ValidatorIndex,
	sender: &mut impl overseer::BitfieldSigningSenderTrait,
) -> Result<AvailabilityBitfield, Error> {
	// get the set of availability cores from the runtime
	let availability_cores = {
		let _span = span.child("get-availability-cores");
		recv_runtime(request_availability_cores(relay_parent, sender).await).await?
	};

	// Wrap the sender in a Mutex to share it between the futures.
	//
	// We use a `Mutex` here to not `clone` the sender inside the future, because
	// cloning the sender will always increase the capacity of the channel by one.
	// (for the lifetime of the sender)
	let sender = Mutex::new(sender);

	// Handle all cores concurrently
	// `try_join_all` returns all results in the same order as the input futures.
	let results = future::try_join_all(
		availability_cores
			.iter()
			.map(|core| get_core_availability(core, n_validators, validator_idx, &sender, span)),
	)
	.await?;

	let core_bits = FromIterator::from_iter(results.into_iter());
	gum::debug!(
		target: LOG_TARGET,
		?relay_parent,
		"Signing Bitfield for {core_count} cores: {core_bits}",
		core_count = availability_cores.len(),
		core_bits = core_bits,
	);

	Ok(AvailabilityBitfield(core_bits))
}

/// The bitfield signing subsystem.
pub struct BitfieldSigningSubsystem {
	keystore: KeystorePtr,
	metrics: Metrics,
}

impl BitfieldSigningSubsystem {
	/// Create a new instance of the `BitfieldSigningSubsystem`.
	pub fn new(keystore: KeystorePtr, metrics: Metrics) -> Self {
		Self { keystore, metrics }
	}
}

#[overseer::subsystem(BitfieldSigning, error=SubsystemError, prefix=self::overseer)]
impl<Context> BitfieldSigningSubsystem {
	fn start(self, ctx: Context) -> SpawnedSubsystem {
		let future = async move {
			run(ctx, self.keystore, self.metrics)
				.await
				.map_err(|e| SubsystemError::with_origin("bitfield-signing", e))
		}
		.boxed();

		SpawnedSubsystem { name: "bitfield-signing-subsystem", future }
	}
}

#[overseer::contextbounds(BitfieldSigning, prefix = self::overseer)]
async fn run<Context>(
	mut ctx: Context,
	keystore: KeystorePtr,
	metrics: Metrics,
) -> SubsystemResult<()> {
	// Track spawned jobs per active leaf.
	let mut running = HashMap::<Hash, future::AbortHandle>::new();

	loop {
		match ctx.recv().await? {
			FromOrchestra::Signal(OverseerSignal::ActiveLeaves(update)) => {
				// Abort jobs for deactivated leaves.
				for leaf in &update.deactivated {
					if let Some(handle) = running.remove(leaf) {
						handle.abort();
					}
				}

				if let Some(leaf) = update.activated {
					let sender = ctx.sender().clone();
					let leaf_hash = leaf.hash;

					let (fut, handle) = future::abortable(handle_active_leaves_update(
						sender,
						leaf,
						keystore.clone(),
						metrics.clone(),
					));

					running.insert(leaf_hash, handle);

					ctx.spawn("bitfield-signing-job", fut.map(drop).boxed())?;
				}
			},
			FromOrchestra::Signal(OverseerSignal::BlockFinalized(..)) => {},
			FromOrchestra::Signal(OverseerSignal::Conclude) => return Ok(()),
			FromOrchestra::Communication { .. } => {},
		}
	}
}

async fn handle_active_leaves_update<Sender>(
	mut sender: Sender,
	leaf: ActivatedLeaf,
	keystore: KeystorePtr,
	metrics: Metrics,
) -> Result<(), Error>
where
	Sender: overseer::BitfieldSigningSenderTrait,
{
	if let LeafStatus::Stale = leaf.status {
		gum::debug!(
			target: LOG_TARGET,
			relay_parent = ?leaf.hash,
			block_number =  ?leaf.number,
			"Skip bitfield signing for stale leaf"
		);
		return Ok(())
	}

	let span = PerLeafSpan::new(leaf.span, "bitfield-signing");
	let span_delay = span.child("delay");
	let wait_until = Instant::now() + SPAWNED_TASK_DELAY;

	let validators = request_validators(leaf.hash, &mut sender)
		.await
		.await
		.map_err(|e| Error::Runtime(e.into()))?
		.map_err(|e| Error::Runtime(e.into()))?;

	// now do all the work we can before we need to wait for the availability store
	// if we're not a validator, we can just succeed effortlessly
	let validator =
		match Validator::new(&validators, leaf.hash, keystore.clone(), &mut sender).await {
			Ok(validator) => validator,
			Err(util::Error::NotAValidator) => return Ok(()),
			Err(err) => return Err(Error::Util(err)),
		};

	// wait a bit before doing anything else
	Delay::new_at(wait_until).await?;

	// this timer does not appear at the head of the function because we don't want to include
	// SPAWNED_TASK_DELAY each time.
	let _timer = metrics.time_run();

	drop(span_delay);
	let span_availability = span.child("availability");

	let bitfield = match construct_availability_bitfield(
		leaf.hash,
		&span_availability,
		validators.len(),
		validator.index(),
		&mut sender,
	)
	.await
	{
		Err(Error::Runtime(runtime_err)) => {
			// Don't take down the node on runtime API errors.
			gum::warn!(target: LOG_TARGET, err = ?runtime_err, "Encountered a runtime API error");
			return Ok(())
		},
		Err(err) => return Err(err),
		Ok(bitfield) => bitfield,
	};

	drop(span_availability);
	let span_signing = span.child("signing");

	let signed_bitfield =
		match validator.sign(keystore, bitfield).map_err(|e| Error::Keystore(e))? {
			Some(b) => b,
			None => {
				gum::error!(
					target: LOG_TARGET,
					"Key was found at construction, but while signing it could not be found.",
				);
				return Ok(())
			},
		};

	metrics.on_bitfield_signed();

	drop(span_signing);
	let _span_gossip = span.child("gossip");

	sender
		.send_message(BitfieldDistributionMessage::DistributeBitfield(leaf.hash, signed_bitfield))
		.await;

	Ok(())
}

async fn get_block_number<Sender>(
	sender: &mut Sender,
	relay_parent: Hash,
) -> Result<BlockNumber, Error>
where
	Sender: overseer::SubsystemSender<ChainApiMessage>,
{
	let (tx, rx) = oneshot::channel();
	sender.send_message(ChainApiMessage::BlockNumber(relay_parent, tx)).await;

	let block_number = rx.await.map_err(Error::ChainApiSenderDropped)?.map_err(Error::ChainApi)?;

	if let Some(number) = block_number {
		Ok(number)
	} else {
		Err(Error::BlockNumberNotFound)
	}
}
