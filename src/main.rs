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
use alloy::sol_types::{SolCall, SolValue};
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

// PANCAKESWAP V2 Router ABI and address
const PANCAKESWAP_V2_ROUTER_ADDRESS: Address = address!("10ED43C718714eb63d5aA57B78B54704E256024E");
sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
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

// sol!(
//     #[sol(abi=true,rpc)]
//     #[derive(Debug, PartialEq, Eq)]
//     interface IWETH {
//         event Approval(address indexed owner, address indexed spender, uint value);
//         event Transfer(address indexed from, address indexed to, uint value);
//
//         function name() external view returns (string memory);
//         function symbol() external view returns (string memory);
//         function decimals() external view returns (uint8);
//         function totalSupply() external view returns (uint);
//         function balanceOf(address owner) external view returns (uint);
//         function allowance(address owner, address spender) external view returns (uint);
//
//         function approve(address spender, uint value) external returns (bool);
//         function transfer(address to, uint value) external returns (bool);
//         function transferFrom(address from, address to, uint value) external returns (bool);
//     }
// );

// WBNB and USDT addresses on BSC
const WBNB_ADDRESS: Address = address!("bb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
const USDT_ADDRESS: Address = address!("55d398326f99059fF775485246999027B3197955");

sol! {
    #[sol(abi=true,rpc, bytecode="6080604052348015600e575f5ffd5b506105108061001c5f395ff3fe608060405260043610610020575f3560e01c80636c7836371461002b575f5ffd5b3661002757005b5f5ffd5b61003e6100393660046102ab565b610040565b005b83835f8181106100525761005261033d565b90506020020160208101906100679190610351565b6001600160a01b031673bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c6001600160a01b0316146100df5760405162461bcd60e51b815260206004820152601860248201527f466972737420746f6b656e206d7573742062652057424e420000000000000000604482015260640160405180910390fd5b5f84845f8181106100f2576100f261033d565b90506020020160208101906101079190610351565b9050806001600160a01b031663d0e30db0346040518263ffffffff1660e01b81526004015f604051808303818588803b158015610142575f5ffd5b505af1158015610154573d5f5f3e3d5ffd5b505050505084845f81811061016b5761016b61033d565b90506020020160208101906101809190610351565b60405163095ea7b360e01b81527310ed43c718714eb63d5aa57b78b54704e256024e60048201523460248201526001600160a01b03919091169063095ea7b3906044016020604051808303815f875af11580156101df573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906102039190610371565b506040516338ed173960e01b81527310ed43c718714eb63d5aa57b78b54704e256024e906338ed1739906102459034908a908a908a908a908a90600401610390565b5f604051808303815f875af1158015610260573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f191682016040526102879190810190610412565b50505050505050565b80356001600160a01b03811681146102a6575f5ffd5b919050565b5f5f5f5f5f608086880312156102bf575f5ffd5b85359450602086013567ffffffffffffffff8111156102dc575f5ffd5b8601601f810188136102ec575f5ffd5b803567ffffffffffffffff811115610302575f5ffd5b8860208260051b8401011115610316575f5ffd5b6020919091019450925061032c60408701610290565b949793965091946060013592915050565b634e487b7160e01b5f52603260045260245ffd5b5f60208284031215610361575f5ffd5b61036a82610290565b9392505050565b5f60208284031215610381575f5ffd5b8151801515811461036a575f5ffd5b8681526020810186905260a06040820181905281018490525f8560c08301825b878110156103de576001600160a01b036103c984610290565b168252602092830192909101906001016103b0565b506001600160a01b03959095166060840152505060800152949350505050565b634e487b7160e01b5f52604160045260245ffd5b5f60208284031215610422575f5ffd5b815167ffffffffffffffff811115610438575f5ffd5b8201601f81018413610448575f5ffd5b805167ffffffffffffffff811115610462576104626103fe565b8060051b604051601f19603f830116810181811067ffffffffffffffff8211171561048f5761048f6103fe565b6040529182526020818401810192908101878411156104ac575f5ffd5b6020850194505b838510156104cf578451808252602095860195909350016104b3565b50969550505050505056fea2646970667358221220c819c425b214a3c9c3e07649f5c045cc1c14b00951717c0bc27e322eedecc46264736f6c634300081c0033")]
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    contract SwapHelper {
        address constant PANCAKESWAP_ROUTER = 0x10ED43C718714eb63d5aA57B78B54704E256024E;
        address private constant WBNB = address("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
        address private constant DAI = address("0x55d398326f99059fF775485246999027B3197955");

        IUniswapV2Router private router = IUniswapV2Router(PANCAKESWAP_ROUTER);
        function swapBNBForTokens(
            address[] calldata path
        ) external payable returns (uint256[] memory result) {
            require(path[0] == WBNB, "First token must be WBNB");
            uint amountIn = 1e18;
            // Wrap BNB to WBNB
            IWETH wbnb = IWETH(WBNB);
            wbnb.transferFrom(msg.sender, address(this), amountIn);
            wbnb.approve(address(this), type(uint).max);


            // Perform swap
            uint256[] memory amounts = router.swapExactTokensForTokens(
                amountIn,
                0, // amountOutMin
                path,
                address(this),
                block.timestamp + 200
            );
            return amounts;
        }

        // Allow contract to receive BNB
        receive() external payable {}
    }
}

// sol!("contract/SwapHelper.sol");

pub fn build_tx(to: Address, from: Address, calldata: Bytes, value: U256, base_fee: u128) -> TransactionRequest {
    TransactionRequest::default()
        .to(to)
        .from(from)
        .with_input(calldata)
        .with_value(value)
        .nonce(0)
        .gas_limit(1000000)
        .max_fee_per_gas(base_fee)
        .max_priority_fee_per_gas(0)
        .build_unsigned()
        .unwrap()
        .into()
}

pub fn decode_quote_response(response: Bytes) -> Result<U256> {
    let (amount_in, amount_out) = <(U256, U256)>::abi_decode(&response, false)?;
    Ok(amount_out)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("WSS_URL must be set");

    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);
    let provider = Arc::new(provider);

    let anvil = Anvil::new()
        .fork(rpc_url)
        .block_time(1_u64)
        .spawn();
    let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse()?);
    let anvil_provider = Arc::new(anvil_provider);

    // Create two users, Alice and Bob.
    let accounts = anvil_provider.get_accounts().await?;
    let alice = accounts[0];
    let bob = accounts[1];

    anvil_provider.anvil_set_nonce(alice, U256::from(1123)).await?;
    // let from = address!("0000000000000000000000000000000000000001");

    // Fund the address with BNB
    // let value = U256::from_str(&*parse_units("1000", 18).unwrap().to_string())?;
    // anvil_provider.anvil_set_balance(from, value).await?;

    let contract = SwapHelper::deploy(anvil_provider.clone()).await?;
    println!("Helper contract deployed at: {}", contract.address());

    // Prepare swap parameters
    let amount_in = U256::from(1000000000000000000u128);
    let path = vec![WBNB_ADDRESS, USDT_ADDRESS];
    // let block = provider
    //     .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
    //     .await?
    //     .unwrap();

    // Create the helper contract call
    let helper_call = SwapHelper::swapBNBForTokensCall::new((path,));

    let data = SwapHelper::swapBNBForTokensCall::abi_encode(&helper_call).into();
    // let gas_price = anvil_provider.get_gas_price().await?;
    // let send_eth_tx = build_tx(*contract.address(), alice, "".parse()?, amount_in,gas_price + 1);
    // anvil_provider.anvil_set_nonce(*contract.address(), U256::from(1123)).await?;
    // let pending_tx = anvil_provider.send_transaction(send_eth_tx).await?;
    // println!("Pending transaction... {}", pending_tx.tx_hash());
    //
    // let tx = contract.swapBNBForTokens(path).call().await?;
    // println!("Tx sent: {tx:?}");

    // let decoded = swapExactTokensForTokensCall::abi_decode(&tx, false)?;
    // println!("{decoded:?}");

    let gas_price = anvil_provider.get_gas_price().await?;
    // println!("Gas price {gas_price}");
    let tx = build_tx(*contract.address(), alice, data, amount_in,gas_price + 1);

    let block = BlockId::Number(BlockNumberOrTag::Latest);
    let trace_options = GethDebugTracingCallOptions::default();
    // let result = anvil_provider
    //     .debug_trace_call(tx.clone(), block, trace_options)
    //     .await?;
    let result = anvil_provider.call(&tx).await?;
    println!("Swap simulation result: {:?}", result);
    let amount_out = decode_quote_response(result)?;
    println!("Amounts Out: {amount_out}");

    Ok(())
}
