/// Represents a tree.
/// Readonly attributes are assigned at tree creation, after which they may not
/// be modified.
///
/// Note: Many APIs within the rest of the code require these objects to
/// be provided. For safety they should be obtained via Admin API calls and
/// not created dynamically.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tree {
    /// ID of the tree.
    /// Readonly.
    #[prost(int64, tag = "1")]
    pub tree_id: i64,
    /// State of the tree.
    /// Trees are ACTIVE after creation. At any point the tree may transition
    /// between ACTIVE, DRAINING and FROZEN states.
    #[prost(enumeration = "TreeState", tag = "2")]
    pub tree_state: i32,
    /// Type of the tree.
    /// Readonly after Tree creation. Exception: Can be switched from
    /// PREORDERED_LOG to LOG if the Tree is and remains in the FROZEN state.
    #[prost(enumeration = "TreeType", tag = "3")]
    pub tree_type: i32,
    /// Display name of the tree.
    /// Optional.
    #[prost(string, tag = "8")]
    pub display_name: ::prost::alloc::string::String,
    /// Description of the tree,
    /// Optional.
    #[prost(string, tag = "9")]
    pub description: ::prost::alloc::string::String,
    /// Storage-specific settings.
    /// Varies according to the storage implementation backing Trillian.
    #[prost(message, optional, tag = "13")]
    pub storage_settings: ::core::option::Option<::prost_types::Any>,
    /// Interval after which a new signed root is produced even if there have been
    /// no submission.  If zero, this behavior is disabled.
    #[prost(message, optional, tag = "15")]
    pub max_root_duration: ::core::option::Option<::prost_types::Duration>,
    /// Time of tree creation.
    /// Readonly.
    #[prost(message, optional, tag = "16")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Time of last tree update.
    /// Readonly (automatically assigned on updates).
    #[prost(message, optional, tag = "17")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// If true, the tree has been deleted.
    /// Deleted trees may be undeleted during a certain time window, after which
    /// they're permanently deleted (and unrecoverable).
    /// Readonly.
    #[prost(bool, tag = "19")]
    pub deleted: bool,
    /// Time of tree deletion, if any.
    /// Readonly.
    #[prost(message, optional, tag = "20")]
    pub delete_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// SignedLogRoot represents a commitment by a Log to a particular tree.
///
/// Note that the signature itself is no-longer provided by Trillian since
/// <https://github.com/google/trillian/pull/2452> .
/// This functionality was intended to support a niche-use case but added
/// significant complexity and was prone to causing confusion and
/// misunderstanding for personality authors.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedLogRoot {
    /// log_root holds the TLS-serialization of the following structure (described
    /// in RFC5246 notation):
    ///
    /// enum { v1(1), (65535)} Version;
    /// struct {
    ///    uint64 tree_size;
    ///    opaque root_hash<0..128>;
    ///    uint64 timestamp_nanos;
    ///    uint64 revision;
    ///    opaque metadata<0..65535>;
    /// } LogRootV1;
    /// struct {
    ///    Version version;
    ///    select(version) {
    ///      case v1: LogRootV1;
    ///    }
    /// } LogRoot;
    ///
    /// A serialized v1 log root will therefore be laid out as:
    ///
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+-....--+
    /// | ver=1 |          tree_size            |len|    root_hash      |
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+-....--+
    ///
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    /// |        timestamp_nanos        |      revision                 |
    /// +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    ///
    /// +---+---+---+---+---+-....---+
    /// |  len  |    metadata        |
    /// +---+---+---+---+---+-....---+
    ///
    /// (with all integers encoded big-endian).
    #[prost(bytes = "vec", tag = "8")]
    pub log_root: ::prost::alloc::vec::Vec<u8>,
}
/// Proof holds a consistency or inclusion proof for a Merkle tree, as returned
/// by the API.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proof {
    /// leaf_index indicates the requested leaf index when this message is used for
    /// a leaf inclusion proof.  This field is set to zero when this message is
    /// used for a consistency proof.
    #[prost(int64, tag = "1")]
    pub leaf_index: i64,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub hashes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// LogRootFormat specifies the fields that are covered by the
/// SignedLogRoot signature, as well as their ordering and formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LogRootFormat {
    Unknown = 0,
    V1 = 1,
}
impl LogRootFormat {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            LogRootFormat::Unknown => "LOG_ROOT_FORMAT_UNKNOWN",
            LogRootFormat::V1 => "LOG_ROOT_FORMAT_V1",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LOG_ROOT_FORMAT_UNKNOWN" => Some(Self::Unknown),
            "LOG_ROOT_FORMAT_V1" => Some(Self::V1),
            _ => None,
        }
    }
}
/// Defines the way empty / node / leaf hashes are constructed incorporating
/// preimage protection, which can be application specific.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum HashStrategy {
    /// Hash strategy cannot be determined. Included to enable detection of
    /// mismatched proto versions being used. Represents an invalid value.
    UnknownHashStrategy = 0,
    /// Certificate Transparency strategy: leaf hash prefix = 0x00, node prefix =
    /// 0x01, empty hash is digest([]byte{}), as defined in the specification.
    Rfc6962Sha256 = 1,
    /// Sparse Merkle Tree strategy:  leaf hash prefix = 0x00, node prefix = 0x01,
    /// empty branch is recursively computed from empty leaf nodes.
    /// NOT secure in a multi tree environment. For testing only.
    TestMapHasher = 2,
    /// Append-only log strategy where leaf nodes are defined as the ObjectHash.
    /// All other properties are equal to RFC6962_SHA256.
    ObjectRfc6962Sha256 = 3,
    /// The CONIKS sparse tree hasher with SHA512_256 as the hash algorithm.
    ConiksSha512256 = 4,
    /// The CONIKS sparse tree hasher with SHA256 as the hash algorithm.
    ConiksSha256 = 5,
}
impl HashStrategy {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            HashStrategy::UnknownHashStrategy => "UNKNOWN_HASH_STRATEGY",
            HashStrategy::Rfc6962Sha256 => "RFC6962_SHA256",
            HashStrategy::TestMapHasher => "TEST_MAP_HASHER",
            HashStrategy::ObjectRfc6962Sha256 => "OBJECT_RFC6962_SHA256",
            HashStrategy::ConiksSha512256 => "CONIKS_SHA512_256",
            HashStrategy::ConiksSha256 => "CONIKS_SHA256",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN_HASH_STRATEGY" => Some(Self::UnknownHashStrategy),
            "RFC6962_SHA256" => Some(Self::Rfc6962Sha256),
            "TEST_MAP_HASHER" => Some(Self::TestMapHasher),
            "OBJECT_RFC6962_SHA256" => Some(Self::ObjectRfc6962Sha256),
            "CONIKS_SHA512_256" => Some(Self::ConiksSha512256),
            "CONIKS_SHA256" => Some(Self::ConiksSha256),
            _ => None,
        }
    }
}
/// State of the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TreeState {
    /// Tree state cannot be determined. Included to enable detection of
    /// mismatched proto versions being used. Represents an invalid value.
    UnknownTreeState = 0,
    /// Active trees are able to respond to both read and write requests.
    Active = 1,
    /// Frozen trees are only able to respond to read requests, writing to a frozen
    /// tree is forbidden. Trees should not be frozen when there are entries
    /// in the queue that have not yet been integrated. See the DRAINING
    /// state for this case.
    Frozen = 2,
    /// Deprecated: now tracked in Tree.deleted.
    DeprecatedSoftDeleted = 3,
    /// Deprecated: now tracked in Tree.deleted.
    DeprecatedHardDeleted = 4,
    /// A tree that is draining will continue to integrate queued entries.
    /// No new entries should be accepted.
    Draining = 5,
}
impl TreeState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TreeState::UnknownTreeState => "UNKNOWN_TREE_STATE",
            TreeState::Active => "ACTIVE",
            TreeState::Frozen => "FROZEN",
            TreeState::DeprecatedSoftDeleted => "DEPRECATED_SOFT_DELETED",
            TreeState::DeprecatedHardDeleted => "DEPRECATED_HARD_DELETED",
            TreeState::Draining => "DRAINING",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN_TREE_STATE" => Some(Self::UnknownTreeState),
            "ACTIVE" => Some(Self::Active),
            "FROZEN" => Some(Self::Frozen),
            "DEPRECATED_SOFT_DELETED" => Some(Self::DeprecatedSoftDeleted),
            "DEPRECATED_HARD_DELETED" => Some(Self::DeprecatedHardDeleted),
            "DRAINING" => Some(Self::Draining),
            _ => None,
        }
    }
}
/// Type of the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TreeType {
    /// Tree type cannot be determined. Included to enable detection of mismatched
    /// proto versions being used. Represents an invalid value.
    UnknownTreeType = 0,
    /// Tree represents a verifiable log.
    Log = 1,
    /// Tree represents a verifiable pre-ordered log, i.e., a log whose entries are
    /// placed according to sequence numbers assigned outside of Trillian.
    PreorderedLog = 3,
}
impl TreeType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TreeType::UnknownTreeType => "UNKNOWN_TREE_TYPE",
            TreeType::Log => "LOG",
            TreeType::PreorderedLog => "PREORDERED_LOG",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN_TREE_TYPE" => Some(Self::UnknownTreeType),
            "LOG" => Some(Self::Log),
            "PREORDERED_LOG" => Some(Self::PreorderedLog),
            _ => None,
        }
    }
}
/// ListTrees request.
/// No filters or pagination options are provided.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTreesRequest {
    /// If true, deleted trees are included in the response.
    #[prost(bool, tag = "1")]
    pub show_deleted: bool,
}
/// ListTrees response.
/// No pagination is provided, all trees the requester has access to are
/// returned.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTreesResponse {
    /// Trees matching the list request filters.
    #[prost(message, repeated, tag = "1")]
    pub tree: ::prost::alloc::vec::Vec<Tree>,
}
/// GetTree request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTreeRequest {
    /// ID of the tree to retrieve.
    #[prost(int64, tag = "1")]
    pub tree_id: i64,
}
/// CreateTree request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTreeRequest {
    /// Tree to be created. See Tree and CreateTree for more details.
    #[prost(message, optional, tag = "1")]
    pub tree: ::core::option::Option<Tree>,
}
/// UpdateTree request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateTreeRequest {
    /// Tree to be updated.
    #[prost(message, optional, tag = "1")]
    pub tree: ::core::option::Option<Tree>,
    /// Fields modified by the update request.
    /// For example: "tree_state", "display_name", "description".
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// DeleteTree request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTreeRequest {
    /// ID of the tree to delete.
    #[prost(int64, tag = "1")]
    pub tree_id: i64,
}
/// UndeleteTree request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndeleteTreeRequest {
    /// ID of the tree to undelete.
    #[prost(int64, tag = "1")]
    pub tree_id: i64,
}
/// Generated client implementations.
pub mod trillian_admin_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Trillian Administrative interface.
    /// Allows creation and management of Trillian trees.
    #[derive(Debug, Clone)]
    pub struct TrillianAdminClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TrillianAdminClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TrillianAdminClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TrillianAdminClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            TrillianAdminClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Lists all trees the requester has access to.
        pub async fn list_trees(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTreesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTreesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/ListTrees",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "ListTrees"));
            self.inner.unary(req, path, codec).await
        }
        /// Retrieves a tree by ID.
        pub async fn get_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTreeRequest>,
        ) -> std::result::Result<tonic::Response<super::Tree>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/GetTree",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "GetTree"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates a new tree.
        /// System-generated fields are not required and will be ignored if present,
        /// e.g.: tree_id, create_time and update_time.
        /// Returns the created tree, with all system-generated fields assigned.
        pub async fn create_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTreeRequest>,
        ) -> std::result::Result<tonic::Response<super::Tree>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/CreateTree",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "CreateTree"));
            self.inner.unary(req, path, codec).await
        }
        /// Updates a tree.
        /// See Tree for details. Readonly fields cannot be updated.
        pub async fn update_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateTreeRequest>,
        ) -> std::result::Result<tonic::Response<super::Tree>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/UpdateTree",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "UpdateTree"));
            self.inner.unary(req, path, codec).await
        }
        /// Soft-deletes a tree.
        /// A soft-deleted tree may be undeleted for a certain period, after which
        /// it'll be permanently deleted.
        pub async fn delete_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTreeRequest>,
        ) -> std::result::Result<tonic::Response<super::Tree>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/DeleteTree",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "DeleteTree"));
            self.inner.unary(req, path, codec).await
        }
        /// Undeletes a soft-deleted a tree.
        /// A soft-deleted tree may be undeleted for a certain period, after which
        /// it'll be permanently deleted.
        pub async fn undelete_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::UndeleteTreeRequest>,
        ) -> std::result::Result<tonic::Response<super::Tree>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianAdmin/UndeleteTree",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianAdmin", "UndeleteTree"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// ChargeTo describes the user(s) associated with the request whose quota should
/// be checked and charged.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChargeTo {
    /// user is a list of personality-defined strings.
    /// Trillian will treat them as /User/%{user}/... keys when checking and
    /// charging quota.
    /// If one or more of the specified users has insufficient quota, the
    /// request will be denied.
    ///
    /// As an example, a Certificate Transparency frontend might set the following
    /// user strings when sending a QueueLeaf request to the Trillian log:
    ///    - The requesting IP address.
    ///      This would limit the number of requests per IP.
    ///    - The "intermediate-<hash>" for each of the intermediate certificates in
    ///      the submitted chain.
    ///      This would have the effect of limiting the rate of submissions under
    ///      a given intermediate/root.
    #[prost(string, repeated, tag = "1")]
    pub user: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueLeafRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(message, optional, tag = "2")]
    pub leaf: ::core::option::Option<LogLeaf>,
    #[prost(message, optional, tag = "3")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueueLeafResponse {
    /// queued_leaf describes the leaf which is or will be incorporated into the
    /// Log.  If the submitted leaf was already present in the Log (as indicated by
    /// its leaf identity hash), then the returned leaf will be the pre-existing
    /// leaf entry rather than the submitted leaf.
    #[prost(message, optional, tag = "2")]
    pub queued_leaf: ::core::option::Option<QueuedLogLeaf>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInclusionProofRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, tag = "2")]
    pub leaf_index: i64,
    #[prost(int64, tag = "3")]
    pub tree_size: i64,
    #[prost(message, optional, tag = "4")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInclusionProofResponse {
    /// The proof field may be empty if the requested tree_size was larger
    /// than that available at the server (e.g. because there is skew between
    /// server instances, and an earlier client request was processed by a
    /// more up-to-date instance).  In this case, the signed_log_root
    /// field will indicate the tree size that the server is aware of, and
    /// the proof field will be empty.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<Proof>,
    #[prost(message, optional, tag = "3")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInclusionProofByHashRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    /// The leaf hash field provides the Merkle tree hash of the leaf entry
    /// to be retrieved.
    #[prost(bytes = "vec", tag = "2")]
    pub leaf_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "3")]
    pub tree_size: i64,
    #[prost(bool, tag = "4")]
    pub order_by_sequence: bool,
    #[prost(message, optional, tag = "5")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInclusionProofByHashResponse {
    /// Logs can potentially contain leaves with duplicate hashes so it's possible
    /// for this to return multiple proofs.  If the leaf index for a particular
    /// instance of the requested Merkle leaf hash is beyond the requested tree
    /// size, the corresponding proof entry will be missing.
    #[prost(message, repeated, tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<Proof>,
    #[prost(message, optional, tag = "3")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConsistencyProofRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, tag = "2")]
    pub first_tree_size: i64,
    #[prost(int64, tag = "3")]
    pub second_tree_size: i64,
    #[prost(message, optional, tag = "4")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConsistencyProofResponse {
    /// The proof field may be empty if the requested tree_size was larger
    /// than that available at the server (e.g. because there is skew between
    /// server instances, and an earlier client request was processed by a
    /// more up-to-date instance).  In this case, the signed_log_root
    /// field will indicate the tree size that the server is aware of, and
    /// the proof field will be empty.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<Proof>,
    #[prost(message, optional, tag = "3")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestSignedLogRootRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(message, optional, tag = "2")]
    pub charge_to: ::core::option::Option<ChargeTo>,
    /// If first_tree_size is non-zero, the response will include a consistency
    /// proof between first_tree_size and the new tree size (if not smaller).
    #[prost(int64, tag = "3")]
    pub first_tree_size: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestSignedLogRootResponse {
    #[prost(message, optional, tag = "2")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
    /// proof is filled in with a consistency proof if first_tree_size in
    /// GetLatestSignedLogRootRequest is non-zero (and within the tree size
    /// available at the server).
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<Proof>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetEntryAndProofRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, tag = "2")]
    pub leaf_index: i64,
    #[prost(int64, tag = "3")]
    pub tree_size: i64,
    #[prost(message, optional, tag = "4")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetEntryAndProofResponse {
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<Proof>,
    #[prost(message, optional, tag = "3")]
    pub leaf: ::core::option::Option<LogLeaf>,
    #[prost(message, optional, tag = "4")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(message, optional, tag = "2")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitLogResponse {
    #[prost(message, optional, tag = "1")]
    pub created: ::core::option::Option<SignedLogRoot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddSequencedLeavesRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(message, repeated, tag = "2")]
    pub leaves: ::prost::alloc::vec::Vec<LogLeaf>,
    #[prost(message, optional, tag = "4")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddSequencedLeavesResponse {
    /// Same number and order as in the corresponding request.
    #[prost(message, repeated, tag = "2")]
    pub results: ::prost::alloc::vec::Vec<QueuedLogLeaf>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLeavesByRangeRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, tag = "2")]
    pub start_index: i64,
    #[prost(int64, tag = "3")]
    pub count: i64,
    #[prost(message, optional, tag = "4")]
    pub charge_to: ::core::option::Option<ChargeTo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLeavesByRangeResponse {
    /// Returned log leaves starting from the `start_index` of the request, in
    /// order. There may be fewer than `request.count` leaves returned, if the
    /// requested range extended beyond the size of the tree or if the server opted
    /// to return fewer leaves than requested.
    #[prost(message, repeated, tag = "1")]
    pub leaves: ::prost::alloc::vec::Vec<LogLeaf>,
    #[prost(message, optional, tag = "2")]
    pub signed_log_root: ::core::option::Option<SignedLogRoot>,
}
/// QueuedLogLeaf provides the result of submitting an entry to the log.
/// TODO(pavelkalinnikov): Consider renaming it to AddLogLeafResult or the like.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueuedLogLeaf {
    /// The leaf as it was stored by Trillian. Empty unless `status.code` is:
    ///   - `google.rpc.OK`: the `leaf` data is the same as in the request.
    ///   - `google.rpc.ALREADY_EXISTS` or 'google.rpc.FAILED_PRECONDITION`: the
    ///     `leaf` is the conflicting one already in the log.
    #[prost(message, optional, tag = "1")]
    pub leaf: ::core::option::Option<LogLeaf>,
    /// The status of adding the leaf.
    ///   - `google.rpc.OK`: successfully added.
    ///   - `google.rpc.ALREADY_EXISTS`: the leaf is a duplicate of an already
    ///     existing one. Either `leaf_identity_hash` is the same in the `LOG`
    ///     mode, or `leaf_index` in the `PREORDERED_LOG`.
    ///   - `google.rpc.FAILED_PRECONDITION`: A conflicting entry is already
    ///     present in the log, e.g., same `leaf_index` but different `leaf_data`.
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<super::google::rpc::Status>,
}
/// LogLeaf describes a leaf in the Log's Merkle tree, corresponding to a single log entry.
/// Each leaf has a unique leaf index in the scope of this tree.  Clients submitting new
/// leaf entries should only set the following fields:
///    - leaf_value
///    - extra_data (optionally)
///    - leaf_identity_hash (optionally)
///    - leaf_index (iff the log is a PREORDERED_LOG)
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogLeaf {
    /// merkle_leaf_hash holds the Merkle leaf hash over leaf_value.  This is
    /// calculated by the Trillian server when leaves are added to the tree, using
    /// the defined hashing algorithm and strategy for the tree; as such, the client
    /// does not need to set it on leaf submissions.
    #[prost(bytes = "vec", tag = "1")]
    pub merkle_leaf_hash: ::prost::alloc::vec::Vec<u8>,
    /// leaf_value holds the data that forms the value of the Merkle tree leaf.
    /// The client should set this field on all leaf submissions, and is
    /// responsible for ensuring its validity (the Trillian server treats it as an
    /// opaque blob).
    #[prost(bytes = "vec", tag = "2")]
    pub leaf_value: ::prost::alloc::vec::Vec<u8>,
    /// extra_data holds additional data associated with the Merkle tree leaf.
    /// The client may set this data on leaf submissions, and the Trillian server
    /// will return it on subsequent read operations. However, the contents of
    /// this field are not covered by and do not affect the Merkle tree hash
    /// calculations.
    #[prost(bytes = "vec", tag = "3")]
    pub extra_data: ::prost::alloc::vec::Vec<u8>,
    /// leaf_index indicates the index of this leaf in the Merkle tree.
    /// This field is returned on all read operations, but should only be
    /// set for leaf submissions in PREORDERED_LOG mode (for a normal log
    /// the leaf index is assigned by Trillian when the submitted leaf is
    /// integrated into the Merkle tree).
    #[prost(int64, tag = "4")]
    pub leaf_index: i64,
    /// leaf_identity_hash provides a hash value that indicates the client's
    /// concept of which leaf entries should be considered identical.
    ///
    /// This mechanism allows the client personality to indicate that two leaves
    /// should be considered "duplicates" even though their `leaf_value`s differ.
    ///
    /// If this is not set on leaf submissions, the Trillian server will take its
    /// value to be the same as merkle_leaf_hash (and thus only leaves with
    /// identical leaf_value contents will be considered identical).
    ///
    /// For example, in Certificate Transparency each certificate submission is
    /// associated with a submission timestamp, but subsequent submissions of the
    /// same certificate should be considered identical.  This is achieved
    /// by setting the leaf identity hash to a hash over (just) the certificate,
    /// whereas the Merkle leaf hash encompasses both the certificate and its
    /// submission time -- allowing duplicate certificates to be detected.
    ///
    ///
    /// Continuing the CT example, for a CT mirror personality (which must allow
    /// dupes since the source log could contain them), the part of the
    /// personality which fetches and submits the entries might set
    /// `leaf_identity_hash` to `H(leaf_index||cert)`.
    ///
    /// TODO(pavelkalinnikov): Consider instead using `H(cert)` and allowing
    /// identity hash dupes in `PREORDERED_LOG` mode, for it can later be
    /// upgraded to `LOG` which will need to correctly detect duplicates with
    /// older entries when new ones get queued.
    #[prost(bytes = "vec", tag = "5")]
    pub leaf_identity_hash: ::prost::alloc::vec::Vec<u8>,
    /// queue_timestamp holds the time at which this leaf was queued for
    /// inclusion in the Log, or zero if the entry was submitted without
    /// queuing. Clients should not set this field on submissions.
    #[prost(message, optional, tag = "6")]
    pub queue_timestamp: ::core::option::Option<::prost_types::Timestamp>,
    /// integrate_timestamp holds the time at which this leaf was integrated into
    /// the tree.  Clients should not set this field on submissions.
    #[prost(message, optional, tag = "7")]
    pub integrate_timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
/// Generated client implementations.
pub mod trillian_log_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// The TrillianLog service provides access to an append-only Log data structure
    /// as described in the [Verifiable Data
    /// Structures](docs/papers/VerifiableDataStructures.pdf) paper.
    ///
    /// The API supports adding new entries to the Merkle tree for a specific Log
    /// instance (identified by its log_id) in two modes:
    ///  - For a normal log, new leaf entries are queued up for subsequent
    ///    inclusion in the log, and the leaves are assigned consecutive leaf_index
    ///    values as part of that integration process.
    ///  - For a 'pre-ordered log', new entries have an already-defined leaf
    ///    ordering, and leaves are only integrated into the Merkle tree when a
    ///    contiguous range of leaves is available.
    ///
    /// The API also supports read operations to retrieve leaf contents, and to
    /// provide cryptographic proofs of leaf inclusion and of the append-only nature
    /// of the Log.
    ///
    /// Each API request also includes a charge_to field, which allows API users
    /// to provide quota identifiers that should be "charged" for each API request
    /// (and potentially rejected with codes.ResourceExhausted).
    ///
    /// Various operations on the API also allows for 'server skew', which can occur
    /// when different API requests happen to be handled by different server instances
    /// that may not all be up to date.  An API request that is relative to a specific
    /// tree size may reach a server instance that is not yet aware of this tree size;
    /// in this case the server will typically return an OK response that contains:
    ///  - a signed log root that indicates the tree size that it is aware of
    ///  - an empty response otherwise.
    #[derive(Debug, Clone)]
    pub struct TrillianLogClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TrillianLogClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TrillianLogClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TrillianLogClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            TrillianLogClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// QueueLeaf adds a single leaf to the queue of pending leaves for a normal
        /// log.
        pub async fn queue_leaf(
            &mut self,
            request: impl tonic::IntoRequest<super::QueueLeafRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueueLeafResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/QueueLeaf",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "QueueLeaf"));
            self.inner.unary(req, path, codec).await
        }
        /// GetInclusionProof returns an inclusion proof for a leaf with a given index
        /// in a particular tree.
        ///
        /// If the requested tree_size is larger than the server is aware of, the
        /// response will include the latest known log root and an empty proof.
        pub async fn get_inclusion_proof(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInclusionProofRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetInclusionProofResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetInclusionProof",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "GetInclusionProof"));
            self.inner.unary(req, path, codec).await
        }
        /// GetInclusionProofByHash returns an inclusion proof for any leaves that have
        /// the given Merkle hash in a particular tree.
        ///
        /// If any of the leaves that match the given Merkle has have a leaf index that
        /// is beyond the requested tree size, the corresponding proof entry will be empty.
        pub async fn get_inclusion_proof_by_hash(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInclusionProofByHashRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetInclusionProofByHashResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetInclusionProofByHash",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("trillian.TrillianLog", "GetInclusionProofByHash"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// GetConsistencyProof returns a consistency proof between different sizes of
        /// a particular tree.
        ///
        /// If the requested tree size is larger than the server is aware of,
        /// the response will include the latest known log root and an empty proof.
        pub async fn get_consistency_proof(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConsistencyProofRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetConsistencyProofResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetConsistencyProof",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "GetConsistencyProof"));
            self.inner.unary(req, path, codec).await
        }
        /// GetLatestSignedLogRoot returns the latest log root for a given tree,
        /// and optionally also includes a consistency proof from an earlier tree size
        /// to the new size of the tree.
        ///
        /// If the earlier tree size is larger than the server is aware of,
        /// an InvalidArgument error is returned.
        pub async fn get_latest_signed_log_root(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLatestSignedLogRootRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLatestSignedLogRootResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetLatestSignedLogRoot",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("trillian.TrillianLog", "GetLatestSignedLogRoot"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// GetEntryAndProof returns a log leaf and the corresponding inclusion proof
        /// to a specified tree size, for a given leaf index in a particular tree.
        ///
        /// If the requested tree size is unavailable but the leaf is
        /// in scope for the current tree, the returned proof will be for the
        /// current tree size rather than the requested tree size.
        pub async fn get_entry_and_proof(
            &mut self,
            request: impl tonic::IntoRequest<super::GetEntryAndProofRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetEntryAndProofResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetEntryAndProof",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "GetEntryAndProof"));
            self.inner.unary(req, path, codec).await
        }
        /// InitLog initializes a particular tree, creating the initial signed log
        /// root (which will be of size 0).
        pub async fn init_log(
            &mut self,
            request: impl tonic::IntoRequest<super::InitLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitLogResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/InitLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "InitLog"));
            self.inner.unary(req, path, codec).await
        }
        /// AddSequencedLeaves adds a batch of leaves with assigned sequence numbers
        /// to a pre-ordered log.  The indices of the provided leaves must be contiguous.
        pub async fn add_sequenced_leaves(
            &mut self,
            request: impl tonic::IntoRequest<super::AddSequencedLeavesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AddSequencedLeavesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/AddSequencedLeaves",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "AddSequencedLeaves"));
            self.inner.unary(req, path, codec).await
        }
        /// GetLeavesByRange returns a batch of leaves whose leaf indices are in a
        /// sequential range.
        pub async fn get_leaves_by_range(
            &mut self,
            request: impl tonic::IntoRequest<super::GetLeavesByRangeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLeavesByRangeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/trillian.TrillianLog/GetLeavesByRange",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("trillian.TrillianLog", "GetLeavesByRange"));
            self.inner.unary(req, path, codec).await
        }
    }
}
