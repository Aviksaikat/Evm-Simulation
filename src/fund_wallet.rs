// Alloy latest version 0.7.3
use alloy::network::{Ethereum, EthereumWallet};
use alloy::node_bindings::Anvil;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{address, keccak256, Address, U256};
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::layers::AnvilProvider;
use alloy::providers::{Identity, ReqwestProvider, RootProvider};
use alloy::signers::local::{coins_bip39::English, MnemonicBuilder, PrivateKeySigner};
use alloy::sol_types::SolValue;
use alloy::transports::http::{Client, Http};
use alloy::transports::BoxTransport;
use alloy::{
    providers::{ext::AnvilApi, ProviderBuilder},
    sol,
};
use anyhow::Result;
use std::env;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address target) returns (uint256);
    }
);

pub static WBNB_ADDRESS: Address = address!("bb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");

pub async fn setup_n_fund_wallet() -> Result<(
    FillProvider<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        ReqwestProvider,
        Http<Client>,
        Ethereum,
    >,
    Address,
    PrivateKeySigner,
)> {
    let wss_url = env::var("WSS_URL").expect("WSS_URL must be set");
    let mnemonic = "work man father plunge mystery proud hollow address reunion sauce theory bonus";
    let signer = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .build()?;

    let wallet = EthereumWallet::from(signer.clone());
    let wallet_address = signer.address();
    // let anvil = Anvil::new().fork(wss_url).try_spawn()?;
    // let provider = ProviderBuilder::new().with_recommended_fillers().on_http(anvil.endpoint_url());
    // let info = provider.anvil_node_info().await?;

    // let provider = ProviderBuilder::new()
    //     .with_recommended_fillers()
    //     .wallet(wallet.clone())
    //     .on_anvil_with_config(|anvil| anvil.fork(wss_url));

    let anvil = Anvil::new().fork(wss_url).try_spawn()?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(anvil.endpoint_url());

    // Create an instance of the WETH contract.
    let wbnb = IERC20::new(WBNB_ADDRESS, provider.clone());
    let balance_before = wbnb.balanceOf(wallet_address).call().await?._0;
    println!("WBNB balance before: {}", balance_before);

    // Mock WBNB balance using the Anvil API.
    let hashed_slot = keccak256((wallet_address, U256::from(3)).abi_encode());
    let mocked_balance: U256 = parse_units("15.0", "ether")?.into();
    provider
        .anvil_set_storage_at(WBNB_ADDRESS, hashed_slot.into(), mocked_balance.into())
        .await?;

    // Get the WETH balance of the target account after mocking.
    let balance_after = wbnb.balanceOf(wallet_address).call().await?._0;
    println!("WBNB balance after: {}", balance_after);

    Ok((provider, wallet_address, signer))
}
