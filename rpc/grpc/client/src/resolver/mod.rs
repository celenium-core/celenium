use super::error::Result;
use core::fmt::Debug;
use celenium_grpc_core::{
    ops::CeleniumdPayloadOps,
    protowire::{CeleniumdRequest, CeleniumdResponse},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(&self, op: CeleniumdPayloadOps, request: &CeleniumdRequest) -> CeleniumdResponseReceiver;
    fn handle_response(&self, response: CeleniumdResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type CeleniumdResponseSender = oneshot::Sender<Result<CeleniumdResponse>>;
pub(crate) type CeleniumdResponseReceiver = oneshot::Receiver<Result<CeleniumdResponse>>;
