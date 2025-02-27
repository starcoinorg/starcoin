// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Implementation of libp2p's `NetworkBehaviour` trait that opens a single substream with the
//! remote and then allows any communication with them.
//!
//! The `Protocol` struct uses `GenericProto` in order to open substreams with the rest of the
//! network, then performs the Substrate protocol handling on top.

pub use self::behaviour::{GenericProto, GenericProtoOut};
pub use self::handler::{NotificationsSink, NotifsHandlerError, Ready};
#[allow(clippy::blocks_in_conditions)]
mod behaviour;
mod handler;
mod tests;
mod upgrade;
