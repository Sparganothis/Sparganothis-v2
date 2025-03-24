use std::sync::Arc;

use anyhow::Context;
use matchbox_socket::WebRtcSocket;
use n0_future::task::AbortOnDropHandle;
use n0_future::task::spawn;

use super::signaller::IrohGossipSignallerBuilder;

