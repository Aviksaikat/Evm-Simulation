use crate::UniswapV2Router::swapExactTokensForTokensCall;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::network::TransactionBuilder;
use alloy::node_bindings::Anvil;
use alloy::primitives::utils::parse_units;
use alloy::primitives::{address, keccak256, Address, Bytes, U256};
use alloy::providers::ext::{AnvilApi, DebugApi, TraceApi};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::rpc::types::trace::geth::GethDebugTracingCallOptions;
use alloy::rpc::types::{BlockTransactionsKind, TransactionInput, TransactionRequest};
use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;

// PANCAKESWAP V2 Router ABI and address
const PANCAKESWAP_V2_ROUTER_ADDRESS: Address = address!("10ED43C718714eb63d5aA57B78B54704E256024E");
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract UniswapV2Router {
        function swapExactTokensForTokens(
            uint256 amountIn,
            uint256 amountOutMin,
            address[] path,
            address to,
            uint256 deadline
        ) external returns (uint256[] memory amounts);
    }
);

sol!(
    #[sol(abi=true,rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface IWETH {
        event Approval(address indexed owner, address indexed spender, uint value);
        event Transfer(address indexed from, address indexed to, uint value);

        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint);
        function balanceOf(address owner) external view returns (uint);
        function allowance(address owner, address spender) external view returns (uint);

        function approve(address spender, uint value) external returns (bool);
        function transfer(address to, uint value) external returns (bool);
        function transferFrom(address from, address to, uint value) external returns (bool);
    }
);

// WBNB and USDT addresses on BSC
const WBNB_ADDRESS: Address = address!("bb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
const USDT_ADDRESS: Address = address!("55d398326f99059fF775485246999027B3197955");

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // let (provider, wallet_address, signer) = setup_n_fund_wallet().await?;
    // let _info = provider.anvil_node_info().await?;
    // println!("Node info: {:#?}", info);

    let wss_url = env::var("WSS_URL").expect("WSS_URL must be set");

    let anvil = Anvil::new().fork(wss_url).try_spawn()?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(anvil.endpoint_url());
    // let provider = ProviderBuilder::new().on_anvil_with_wallet();
    // let from = provider.default_signer_address();
    let from = address!("0000000000000000000000000000000000000001");

    // TODO: Do uniswap Swap

    let amount_in = U256::from(1000000000000000000u128);
    let amount_out_min = U256::from(0);

    let path = vec![WBNB_ADDRESS, USDT_ADDRESS];
    let block = provider
        .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
        .await?
        .unwrap();
    let deadline = U256::from(block.header.timestamp + 60);
    let swap_data =
        swapExactTokensForTokensCall::new((amount_in, amount_out_min, path, from, deadline));

    let data = TransactionInput::new(swapExactTokensForTokensCall::abi_encode(&swap_data).into());

    let gas_price = provider.get_gas_price().await?;
    let tx = TransactionRequest::default()
        .with_from(from)
        .with_to(PANCAKESWAP_V2_ROUTER_ADDRESS)
        .input(data)
        .with_value(amount_in)
        .max_fee_per_gas(gas_price + 1)
        .max_priority_fee_per_gas(gas_price + 1);

    let block = BlockId::Number(BlockNumberOrTag::Latest);
    let trace_options = GethDebugTracingCallOptions::default();
    let result = provider
        .debug_trace_call(tx.clone(), block, trace_options)
        .await?;
    println!("Simulate result: {:#?}", result);

    // let res = provider.call(&tx).await?;
    // println!("Call result: {:?}", res);

    Ok(())
}
