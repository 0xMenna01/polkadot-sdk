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

//! The `Error` and `Result` types used by the subsystem.

use futures::channel::oneshot;
use polkadot_node_subsystem::ChainApiError;
use thiserror::Error;

/// Error type used by the Availability Recovery subsystem.
#[derive(Debug, Error)]
// TODO: add fatality
pub enum Error {
	#[error(transparent)]
	Subsystem(#[from] polkadot_node_subsystem::SubsystemError),

	#[error("failed to query full data from store")]
	CanceledQueryFullData(#[source] oneshot::Canceled),

	#[error("failed to query session info")]
	CanceledSessionInfo(#[source] oneshot::Canceled),

	#[error("failed to query client features from runtime")]
	RequestClientFeatures(#[source] polkadot_node_subsystem_util::runtime::Error),

	#[error("failed to send response")]
	CanceledResponseSender,

	#[error(transparent)]
	Runtime(#[from] polkadot_node_subsystem::errors::RuntimeApiError),

	#[error(transparent)]
	Erasure(#[from] polkadot_erasure_coding::Error),

	#[error(transparent)]
	Util(#[from] polkadot_node_subsystem_util::Error),

	#[error("Oneshot for receiving response from Chain API got cancelled")]
	ChainApiSenderDropped(#[source] oneshot::Canceled),

	#[error("Retrieving response from Chain API unexpectedly failed with error: {0}")]
	ChainApi(#[from] ChainApiError),

	#[error("Cannot find block number for given relay parent")]
	BlockNumberNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
