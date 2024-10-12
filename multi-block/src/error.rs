use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Failed to parse log directive: `{0}´")]
	LogParse(#[from] tracing_subscriber::filter::ParseError),
	#[error("I/O error: `{0}`")]
	Io(#[from] std::io::Error),
	#[error("subxt error: `{0}`")]
	Subxt(#[from] subxt::Error),
	#[error("Invalid chain: `{0}`, staking-miner supports only polkadot, kusama and westend")]
	InvalidChain(String),
	#[error("Invalid metadata: {0}")]
	InvalidMetadata(String),
	#[error("Codec error: `{0}`")]
	Codec(#[from] codec::Error),
	#[error("The account does not exist")]
	AccountDoesNotExists,
	#[error("Crypto error: `{0:?}`")]
	Crypto(sp_core::crypto::SecretStringError),
	#[error("Empty snapshot")]
	EmptySnapshot,
	#[error("Miner error: `{0}`")]
	Miner(String),
	#[error("Dynamic transaction error: {0}")]
	DynamicTransaction(String),
	#[error("Other error: `{0}`")]
	Other(String),
}
