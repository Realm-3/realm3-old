use hex_literal::hex;
use node_primitives::*;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use realm3_runtime::{
	constants::currency::*, opaque::SessionKeys, wasm_binary_unwrap, BabeConfig, BalancesConfig,
	CouncilConfig, DemocracyConfig, ElectionsConfig, FaucetsConfig, GenesisConfig, GrandpaConfig,
	ImOnlineConfig, MaxNominations, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, TechnicalCommitteeConfig, BABE_GENESIS_EPOCH_CONFIG,
};
use sc_chain_spec::Properties;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

const SS58_PREFIX: u32 = 5821;
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

fn session_keys(babe: BabeId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online }
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let properties = Some(get_properties("R3C", 12, 42));

	Ok(ChainSpec::from_genesis(
		// Name
		"Realm3 Development",
		// ID
		"realm3_dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				vec![],
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					100_800,               // period
					1_000_000_000_000_000, // period_limit
					100_000_000_000,       // drip_limit
				)],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		properties,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let properties = Some(get_properties("R3C", 12, SS58_PREFIX));
	Ok(ChainSpec::from_genesis(
		// Name
		"Realm3 Local Testnet",
		// ID
		"realm3_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					100_800,               // period
					1_000_000_000_000_000, // period_limit
					100_000_000_000,       // drip_limit
				)],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		properties,
		// Extensions
		None,
	))
}

pub fn staging_network_config() -> ChainSpec {
	let boot_nodes = vec![];
	let properties = Some(get_properties("R3C", 12, SS58_PREFIX));

	ChainSpec::from_genesis(
		"Realm3",
		"realm3",
		ChainType::Live,
		staging_network_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		properties,
		Default::default(),
	)
}

fn staging_network_config_genesis() -> GenesisConfig {
	// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	let initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)> = vec![
		(
			// 5G9Ricxk4NuwhUzsifzeHUCPWn65j2mgzQXZFs5b24nJMnon
			hex!["b48ad53b2d3b7842f52417ae44658240dad9a85c0bbbe52bff9a0f9dab8dd00e"].into(),
			// 5DP7aavT96p5zFiGLXfWHWC65GehxwFP9PBuG3HMfPtWPvkc
			hex!["3a46822b8993d49bd74116dae5a7402b1c742f932b44d3878b2240e0d4810c3d"].into(),
			// 5DDE3ptBNGFEkZEVR4KArD3evwTr69jG5YLU1qjj5MEayBLM
			hex!["32bbd5517ea65ced4b4d1b18797d42dacf8c4caa8e1e0a5e883a39a62b01154a"]
				.unchecked_into(),
			// 5EoQhxogEzHDfJo5dVnVBUwwjyZJfWf9Gxwv8mzeotgDHS1u
			hex!["790a497308b55518b2977dc5fd73439e1f8c4ffcb6863bf7002b076606e87900"]
				.unchecked_into(),
			// 5CRy8QK7tMsysghJYzGY8G97G5tj2tXdhGyMZrcNPFPPtEEL
			hex!["10378faa125b7b9ce9e9f7e1cfc38658a6a069e9d0330d9d38984c82f89df34c"]
				.unchecked_into(),
		),
		(
			// 5FLUGF5HPTLWvDTXebqCpyiRJHB8t8mfxsUTq6ggfX8ZbHQm
			hex!["02b0a01d3fd5ece4db1b45eadcc20ddd97167963ef2ac1210f0498de03c51b06"].into(),
			// 5CJp2sfu7YZd8oBnZcetBF8Ws8BMBVBQPfXBv1PTC7awHRtx
			hex!["e63da98a27c08e2ead42faf5298624f9b518056685dc247749d6f121b4008fb5"].into(),
			// 5HeLRA1pibg2KzkrBjEaBHPMc72LKpuqhxGZ8PTenWoDx6XU
			hex!["f6d3568573fd8a6f3627ef43d279cae66a955adf378e09d786dde341c825414a"]
				.unchecked_into(),
			// 5DCiFAhTcWahtZHirVYGDpQSPc5ypYzYadikHMqaqkPmzbTp
			hex!["325780a64b5172f8398d9b28aec784332f42fa8508f73d006a4b609cd6cf253b"]
				.unchecked_into(),
			// 5GZaiKD4cZDx26KviF7gPSM4aPpPurhZySmjeHoYbBWsvhyf
			hex!["c6f700672cdc93a23b1b134be663136eb1b6af33c0a1168ef905ce3990027f35"]
				.unchecked_into(),
		),
		(
			// 5FCY2LLS25NHCyDUY7KL4o7RCB7HiYtcrdLPgxuhwYk4YbTP
			hex!["8aad8fc320c88ce1ddf85cb542e3d3d06aefceedcf9cbd7daa55e595fd2e5b78"].into(),
			// 5EHuP7CfsC81uaBiMLxpq4LakmW2wfdcWWt2ojJnH1KVYkEp
			hex!["628974006e2df3654ecf8011aa3add1e8cae365b326d5fe694be58b7530dc045"].into(),
			// 5EqgeGGenmJNSc2ByUz2nahrcokbKpXrrDaG3cLQYDaK1NYk
			hex!["7ac66c80c4d722851a38e0e0fda03a81a6b1375c731494ff4eaaca5723841b52"]
				.unchecked_into(),
			// 5GMJYpYD76CZhDChtTKJUWyruGoFHEpHM4MTa1Bsbd3af6st
			hex!["bd99a5167cdaa2b6465d974818125ee432672f3362abe2f181552ddb06997295"]
				.unchecked_into(),
			// 5DPYBD9FcksMkamGiZ1RBTG8s96egUZw4Jb9SHB2qrbC7v9Q
			hex!["3a994f2c7a5f970f1cb8c9be2daefa935da54e676e801b0641d1f71ec0d34336"]
				.unchecked_into(),
		),
		(
			// 5CXry4exXj7MAds8NAJ4QFFXXzZykXXg1Hp46Z45BSsFu6LE
			hex!["14b64b3593749c8e600ff87c0bd36360fdad49be3248c2a8dce4e359221cf15a"].into(),
			// 5DZgahg66K1zB4jhQp7wp7ChsXA8KTtWYdTbEz6bdhyp5vz7
			hex!["42560ea357a94d3576fd21b5510c98cd9916c63a37ebd44e9c7af0a01e908c1c"].into(),
			// 5HihBpuT5ETL6HJfVJdpCyM4N2B7QMS1ZTdw3KjZAzpkrDAJ
			hex!["fa263cfe3d21931aee0f157b9dd04819492ca29b9f652e64d831f3bfaad74f47"]
				.unchecked_into(),
			// 5GHpoMBtRSriuZGnBGkQRrtYLbYzi76jQ1qBTStga55eZPQm
			hex!["baf27ed81a311b06af8fcd042e5874ce8a26c7fa689e4f852e5d2144871cd2d6"]
				.unchecked_into(),
			// 5FtBt5o2zS9kqniJLwukYW48UupSpm7bMHLr7wyWPMwJjBkE
			hex!["29b190372b53d161213c71f3db6918b26fb942bb0656a13de43297e69e4001e6"]
				.unchecked_into(),
		),
	];

	let root_key: AccountId = hex![
		// 5EvmQMfNq6x8k5kuL5TvHGMzJ84HwAsPZEmdxgeDQhRJFH67
		"7ea6acda0f98b8819949972a2900e4998b9b629d5dba68b50fd195f2cee4f15a"
	]
	.into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], vec![], root_key, endowed_accounts)
}

pub fn default_config() -> ChainSpec {
	let boot_nodes = vec![];
	let properties = Some(get_properties("R3", 12, SS58_PREFIX));

	ChainSpec::from_genesis(
		"Realm3",
		"realm3",
		ChainType::Live,
		default_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		properties,
		Default::default(),
	)
}

fn default_genesis() -> GenesisConfig {
	let root_key: AccountId = hex![
		// 5FemZuvaJ7wVy4S49X7Y9mj7FyTR4caQD5mZo2rL7MXQoXMi
		"9eaf896d76b55e04616ff1e1dce7fc5e4a417967c17264728b3fd8fee3b12f3c"
	]
	.into();
	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(vec![], vec![], vec![], root_key, endowed_accounts)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	initial_nominators: Vec<AccountId>,
	initial_faucets: Vec<(AccountId, BlockNumber, Balance, Balance)>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary_unwrap().to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone()))
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			// TODO: ForceEra::ForceNone
			..Default::default()
		},
		babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: GrandpaConfig { authorities: vec![] },
		im_online: ImOnlineConfig { keys: vec![] },
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		treasury: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		faucets: FaucetsConfig { initial_faucets },
	}
}

pub fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), symbol.into());
	properties.insert("tokenDecimals".into(), decimals.into());
	properties.insert("ss58Format".into(), ss58format.into());

	properties
}
