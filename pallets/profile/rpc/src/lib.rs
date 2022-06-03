use codec::{Decode, Encode};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_profile::rpc::ProProfile;
pub use profile_runtime_api::ProfileApi as ProfileStorageRuntimeApi;
use rp_profile::{Area, Profession};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::scale_info::TypeInfo;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ProfileStorageApi<BlockHash, AccountId, Hash> {
	#[rpc(name = "profile_isUsernameExist")]
	fn is_username_exist(&self, at: Option<BlockHash>, username: Hash) -> Result<bool>;
	#[rpc(name = "profile_getProProfiles")]
	fn get_pros(
		&self,
		at: Option<BlockHash>,
		profession: Profession,
		area: Area,
		offset: u64,
		limit: u16,
	) -> Result<Vec<ProProfile<AccountId>>>;
}

pub struct ProfileStorage<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> ProfileStorage<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, Hash> ProfileStorageApi<<Block as BlockT>::Hash, AccountId, Hash>
	for ProfileStorage<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: ProfileStorageRuntimeApi<Block, AccountId, Hash>,
	AccountId: Encode + Decode + Clone + PartialEq + TypeInfo,
	Hash: Encode + Decode + Default + Clone + PartialEq + TypeInfo,
{
	fn is_username_exist(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		username: Hash,
	) -> Result<bool> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.is_username_exist(&at, username);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(2201), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_pros(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		profession: Profession,
		area: Area,
		offset: u64,
		limit: u16,
	) -> Result<Vec<ProProfile<AccountId>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_pros(&at, profession, area, offset, limit);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(2202), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
