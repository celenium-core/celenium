use crate::protowire::{CeleniumdRequest, CeleniumdResponse, celeniumd_request};

impl From<celeniumd_request::Payload> for CeleniumdRequest {
    fn from(item: celeniumd_request::Payload) -> Self {
        CeleniumdRequest { id: 0, payload: Some(item) }
    }
}

impl AsRef<CeleniumdRequest> for CeleniumdRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<CeleniumdResponse> for CeleniumdResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod celeniumd_request_convert {
    use crate::protowire::*;
    use celenium_rpc_core::{RpcError, RpcResult};

    impl_into_celeniumd_request!(Shutdown);
    impl_into_celeniumd_request!(SubmitBlock);
    impl_into_celeniumd_request!(GetBlockTemplate);
    impl_into_celeniumd_request!(GetBlock);
    impl_into_celeniumd_request!(GetInfo);

    impl_into_celeniumd_request!(GetCurrentNetwork);
    impl_into_celeniumd_request!(GetPeerAddresses);
    impl_into_celeniumd_request!(GetSink);
    impl_into_celeniumd_request!(GetMempoolEntry);
    impl_into_celeniumd_request!(GetMempoolEntries);
    impl_into_celeniumd_request!(GetConnectedPeerInfo);
    impl_into_celeniumd_request!(AddPeer);
    impl_into_celeniumd_request!(SubmitTransaction);
    impl_into_celeniumd_request!(SubmitTransactionReplacement);
    impl_into_celeniumd_request!(GetSubnetwork);
    impl_into_celeniumd_request!(GetVirtualChainFromBlock);
    impl_into_celeniumd_request!(GetBlocks);
    impl_into_celeniumd_request!(GetBlockCount);
    impl_into_celeniumd_request!(GetBlockDagInfo);
    impl_into_celeniumd_request!(ResolveFinalityConflict);
    impl_into_celeniumd_request!(GetHeaders);
    impl_into_celeniumd_request!(GetUtxosByAddresses);
    impl_into_celeniumd_request!(GetBalanceByAddress);
    impl_into_celeniumd_request!(GetBalancesByAddresses);
    impl_into_celeniumd_request!(GetSinkBlueScore);
    impl_into_celeniumd_request!(Ban);
    impl_into_celeniumd_request!(Unban);
    impl_into_celeniumd_request!(EstimateNetworkHashesPerSecond);
    impl_into_celeniumd_request!(GetMempoolEntriesByAddresses);
    impl_into_celeniumd_request!(GetCoinSupply);
    impl_into_celeniumd_request!(Ping);
    impl_into_celeniumd_request!(GetMetrics);
    impl_into_celeniumd_request!(GetConnections);
    impl_into_celeniumd_request!(GetSystemInfo);
    impl_into_celeniumd_request!(GetServerInfo);
    impl_into_celeniumd_request!(GetSyncStatus);
    impl_into_celeniumd_request!(GetDaaScoreTimestampEstimate);
    impl_into_celeniumd_request!(GetFeeEstimate);
    impl_into_celeniumd_request!(GetFeeEstimateExperimental);
    impl_into_celeniumd_request!(GetCurrentBlockColor);
    impl_into_celeniumd_request!(GetUtxoReturnAddress);
    impl_into_celeniumd_request!(GetVirtualChainFromBlockV2);
    impl_into_celeniumd_request!(GetBlockRewardInfo);
    impl_into_celeniumd_request!(GetSeqCommitLaneProof);
    impl_into_celeniumd_request!(GetUtxosByAddressesV2);

    impl_into_celeniumd_request!(NotifyBlockAdded);
    impl_into_celeniumd_request!(NotifyNewBlockTemplate);
    impl_into_celeniumd_request!(NotifyUtxosChanged);
    impl_into_celeniumd_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_celeniumd_request!(NotifyFinalityConflict);
    impl_into_celeniumd_request!(NotifyVirtualDaaScoreChanged);
    impl_into_celeniumd_request!(NotifyVirtualChainChanged);
    impl_into_celeniumd_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_celeniumd_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_celeniumd_request_ex!(celenium_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_celeniumd_request;

    macro_rules! impl_into_celeniumd_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for celeniumd_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for CeleniumdRequest {
                fn from(item: &$core_struct) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<$core_struct> for celeniumd_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for CeleniumdRequest {
                fn from(item: $core_struct) -> Self {
                    Self { id: 0, payload: Some((&item).into()) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&celeniumd_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &celeniumd_request::Payload) -> RpcResult<Self> {
                    if let celeniumd_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&CeleniumdRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &CeleniumdRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("CeleniumRequest".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for CeleniumdRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(celeniumd_request::Payload::$variant(item)) }
                }
            }

            impl From<$protowire_struct> for celeniumd_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    celeniumd_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_celeniumd_request_ex;
}

pub mod celeniumd_response_convert {
    use crate::protowire::*;
    use celenium_rpc_core::{RpcError, RpcResult};

    impl_into_celeniumd_response!(Shutdown);
    impl_into_celeniumd_response!(SubmitBlock);
    impl_into_celeniumd_response!(GetBlockTemplate);
    impl_into_celeniumd_response!(GetBlock);
    impl_into_celeniumd_response!(GetInfo);
    impl_into_celeniumd_response!(GetCurrentNetwork);

    impl_into_celeniumd_response!(GetPeerAddresses);
    impl_into_celeniumd_response!(GetSink);
    impl_into_celeniumd_response!(GetMempoolEntry);
    impl_into_celeniumd_response!(GetMempoolEntries);
    impl_into_celeniumd_response!(GetConnectedPeerInfo);
    impl_into_celeniumd_response!(AddPeer);
    impl_into_celeniumd_response!(SubmitTransaction);
    impl_into_celeniumd_response!(SubmitTransactionReplacement);
    impl_into_celeniumd_response!(GetSubnetwork);
    impl_into_celeniumd_response!(GetVirtualChainFromBlock);
    impl_into_celeniumd_response!(GetBlocks);
    impl_into_celeniumd_response!(GetBlockCount);
    impl_into_celeniumd_response!(GetBlockDagInfo);
    impl_into_celeniumd_response!(ResolveFinalityConflict);
    impl_into_celeniumd_response!(GetHeaders);
    impl_into_celeniumd_response!(GetUtxosByAddresses);
    impl_into_celeniumd_response!(GetBalanceByAddress);
    impl_into_celeniumd_response!(GetBalancesByAddresses);
    impl_into_celeniumd_response!(GetSinkBlueScore);
    impl_into_celeniumd_response!(Ban);
    impl_into_celeniumd_response!(Unban);
    impl_into_celeniumd_response!(EstimateNetworkHashesPerSecond);
    impl_into_celeniumd_response!(GetMempoolEntriesByAddresses);
    impl_into_celeniumd_response!(GetCoinSupply);
    impl_into_celeniumd_response!(Ping);
    impl_into_celeniumd_response!(GetMetrics);
    impl_into_celeniumd_response!(GetConnections);
    impl_into_celeniumd_response!(GetSystemInfo);
    impl_into_celeniumd_response!(GetServerInfo);
    impl_into_celeniumd_response!(GetSyncStatus);
    impl_into_celeniumd_response!(GetDaaScoreTimestampEstimate);
    impl_into_celeniumd_response!(GetFeeEstimate);
    impl_into_celeniumd_response!(GetFeeEstimateExperimental);
    impl_into_celeniumd_response!(GetCurrentBlockColor);
    impl_into_celeniumd_response!(GetUtxoReturnAddress);
    impl_into_celeniumd_response!(GetVirtualChainFromBlockV2);
    impl_into_celeniumd_response!(GetBlockRewardInfo);
    impl_into_celeniumd_response!(GetSeqCommitLaneProof);
    impl_into_celeniumd_response!(GetUtxosByAddressesV2);

    impl_into_celeniumd_notify_response!(NotifyBlockAdded);
    impl_into_celeniumd_notify_response!(NotifyNewBlockTemplate);
    impl_into_celeniumd_notify_response!(NotifyUtxosChanged);
    impl_into_celeniumd_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_celeniumd_notify_response!(NotifyFinalityConflict);
    impl_into_celeniumd_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_celeniumd_notify_response!(NotifyVirtualChainChanged);
    impl_into_celeniumd_notify_response!(NotifySinkBlueScoreChanged);

    impl_into_celeniumd_notify_response!(NotifyUtxosChanged, StopNotifyingUtxosChanged);
    impl_into_celeniumd_notify_response!(NotifyPruningPointUtxoSetOverride, StopNotifyingPruningPointUtxoSetOverride);

    macro_rules! impl_into_celeniumd_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_celeniumd_response_ex!(celenium_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            paste::paste! {
                impl_into_celeniumd_response_base!(celenium_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>],[<$protowire_name Response>]);
            }
        };
    }
    use impl_into_celeniumd_response;

    macro_rules! impl_into_celeniumd_response_base {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for celeniumd_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    celeniumd_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for CeleniumdResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(celeniumd_response::Payload::$variant(item)) }
                }
            }
        };
    }
    use impl_into_celeniumd_response_base;

    macro_rules! impl_into_celeniumd_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for celeniumd_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    celeniumd_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for CeleniumdResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for celeniumd_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    celeniumd_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for CeleniumdResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl_into_celeniumd_response_base!($core_struct, $protowire_struct, $variant);

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&celeniumd_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &celeniumd_response::Payload) -> RpcResult<Self> {
                    if let celeniumd_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&CeleniumdResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &CeleniumdResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("CeleniumResponse".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_celeniumd_response_ex;

    macro_rules! impl_into_celeniumd_notify_response {
        ($name:tt) => {
            impl_into_celeniumd_response!($name);

            paste::paste! {
                impl_into_celeniumd_notify_response_ex!(celenium_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            impl_into_celeniumd_response!($core_name, $protowire_name);

            paste::paste! {
                impl_into_celeniumd_notify_response_ex!(celenium_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>]);
            }
        };
    }
    use impl_into_celeniumd_notify_response;

    macro_rules! impl_into_celeniumd_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_celeniumd_notify_response_ex;
}
