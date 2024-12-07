use alloy::contract::{ContractInstance, Interface};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::network::primitives::BlockTransactionsKind;
use alloy::network::TransactionBuilder;
use alloy::node_bindings::Anvil;
use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::ext::{AnvilApi, DebugApi, TraceApi};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::rpc::types::trace::geth::{GethDebugTracingCallOptions, GethDebugTracingOptions};
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::{SolCall, SolValue};
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use crate::IUniswapV2Router::swapExactTokensForTokensCall;

// PANCAKESWAP V2 Router ABI and address
const PANCAKESWAP_V2_ROUTER_ADDRESS: Address = address!("10ED43C718714eb63d5aA57B78B54704E256024E");
// sol!(
//     #[allow(missing_docs)]
//     #[derive(Debug, PartialEq, Eq)]
//     #[sol(rpc)]
//     interface IUniswapV2Router {
//         function swapExactTokensForTokens(
//             uint256 amountIn,
//             uint256 amountOutMin,
//             address[] calldata path,
//             address to,
//             uint256 deadline
//         ) external returns (uint256[] memory amounts);
//     }
// );
//
// sol!(
//     #[sol(abi=true,rpc)]
//     #[derive(Debug, PartialEq, Eq)]
//     interface IERC20 {
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
//
//     interface IWETH is IERC20 {
//         function deposit() external payable;
//         function withdraw(uint256 amount) external;
//     }
// );

// WBNB and USDT addresses on BSC
const WBNB_ADDRESS: Address = address!("bb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");
const USDT_ADDRESS: Address = address!("55d398326f99059fF775485246999027B3197955");
const BUSD_ADDRESS: Address = address!("e9e7CEA3DedcA5984780Bafc599bD69ADd087D56");

sol! {

    // interface IWETH {
    //     function deposit() external payable;
    //     function approve(address spender, uint256 amount) external returns (bool);
    //     function transfer(address to, uint256 amount) external returns (bool);
    //     function balanceOf(address account) external view returns (uint256);
    // }
    //
    // interface IERC20 {
    //     function approve(address spender, uint256 amount) external returns (bool);
    //     function transfer(address to, uint256 amount) external returns (bool);
    //     function transferFrom(address from, address to, uint256 amount) external returns (bool);
    //     function balanceOf(address account) external view returns (uint256);
    // }

    // interface IUniswapV2Router {
    //     function swapExactTokensForTokens(
    //         uint256 amountIn,
    //         uint256 amountOutMin,
    //         address[] calldata path,
    //         address to,
    //         uint256 deadline
    //     ) external returns (uint256[] memory amounts);
    // }

    #[sol(abi=true,rpc, bytecode="60806040525f80546001600160a01b03199081167310ed43c718714eb63d5aa57b78b54704e256024e1790915560018054821673bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c1790556002805482167355d398326f99059ff775485246999027b31979551790556003805490911673e9e7cea3dedca5984780bafc599bd69add087d561790553480156091575f5ffd5b50610a2a8061009f5f395ff3fe608060405260043610610036575f3560e01c8063bdf7259914610041578063da8458df1461005f578063e47677771461008c575f5ffd5b3661003d57005b5f5ffd5b6100496100ab565b60405161005691906107fe565b60405180910390f35b34801561006a575f5ffd5b5061007e610079366004610840565b61036c565b604051908152602001610056565b348015610097575f5ffd5b5061007e6100a6366004610840565b6105e3565b6060670de0b6b3a76400003410156100fe5760405162461bcd60e51b815260206004820152601260248201527153656e64203120424e42206f72206d6f726560701b604482015260640160405180910390fd5b60015460408051630d0e30db60e41b81529051670de0b6b3a7640000926001600160a01b03169163d0e30db09184916004808201925f9290919082900301818588803b15801561014c575f5ffd5b505af115801561015e573d5f5f3e3d5ffd5b505060015460405163095ea7b360e01b8152306004820152602481018690526001600160a01b03909116935063095ea7b3925060440190506020604051808303815f875af11580156101b2573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906101d69190610860565b506001545f5460405163095ea7b360e01b81526001600160a01b0391821660048201526024810184905291169063095ea7b3906044016020604051808303815f875af1158015610228573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061024c9190610860565b50604080516002808252606080830184529260208301908036833701905050905073bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c815f815181106102945761029461089a565b60200260200101906001600160a01b031690816001600160a01b0316815250507355d398326f99059ff775485246999027b3197955816001815181106102dc576102dc61089a565b6001600160a01b0392831660209182029290920101525f80546040516338ed173960e01b8152919216906338ed17399061032290869085908790339042906004016108f1565b5f604051808303815f875af115801561033d573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f19168201604052610364919081019061092c565b949350505050565b6002546040516323b872dd60e01b8152336004820152306024820152604481018490525f916001600160a01b0316906323b872dd906064016020604051808303815f875af11580156103c0573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906103e49190610860565b506002545f5460405163095ea7b360e01b81526001600160a01b0391821660048201526024810186905291169063095ea7b3906044016020604051808303815f875af1158015610436573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061045a9190610860565b50604080516003808252608082019092526060916020820183803683370190505090507355d398326f99059ff775485246999027b3197955815f815181106104a4576104a461089a565b60200260200101906001600160a01b031690816001600160a01b03168152505073bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c816001815181106104ec576104ec61089a565b60200260200101906001600160a01b031690816001600160a01b03168152505073e9e7cea3dedca5984780bafc599bd69add087d56816002815181106105345761053461089a565b6001600160a01b0392831660209182029290920101525f80546040516338ed173960e01b8152919216906338ed17399061057a90889088908790339042906004016108f1565b5f604051808303815f875af1158015610595573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f191682016040526105bc919081019061092c565b9050806002815181106105d1576105d161089a565b60200260200101519250505092915050565b6001546040516323b872dd60e01b8152336004820152306024820152604481018490525f916001600160a01b0316906323b872dd906064016020604051808303815f875af1158015610637573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061065b9190610860565b506001545f5460405163095ea7b360e01b81526001600160a01b0391821660048201526024810186905291169063095ea7b3906044016020604051808303815f875af11580156106ad573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906106d19190610860565b50604080516002808252606080830184529260208301908036833701905050905073bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c815f815181106107195761071961089a565b60200260200101906001600160a01b031690816001600160a01b0316815250507355d398326f99059ff775485246999027b3197955816001815181106107615761076161089a565b6001600160a01b0392831660209182029290920101525f80546040516338ed173960e01b8152919216906338ed1739906107a790889088908790339042906004016108f1565b5f604051808303815f875af11580156107c2573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f191682016040526107e9919081019061092c565b9050806001815181106105d1576105d161089a565b602080825282518282018190525f918401906040840190835b81811015610835578351835260209384019390920191600101610817565b509095945050505050565b5f5f60408385031215610851575f5ffd5b50508035926020909101359150565b5f60208284031215610870575f5ffd5b8151801515811461087f575f5ffd5b9392505050565b634e487b7160e01b5f52604160045260245ffd5b634e487b7160e01b5f52603260045260245ffd5b5f8151808452602084019350602083015f5b828110156108e75781516001600160a01b03168652602095860195909101906001016108c0565b5093949350505050565b85815284602082015260a060408201525f61090f60a08301866108ae565b6001600160a01b0394909416606083015250608001529392505050565b5f6020828403121561093c575f5ffd5b815167ffffffffffffffff811115610952575f5ffd5b8201601f81018413610962575f5ffd5b805167ffffffffffffffff81111561097c5761097c610886565b8060051b604051601f19603f830116810181811067ffffffffffffffff821117156109a9576109a9610886565b6040529182526020818401810192908101878411156109c6575f5ffd5b6020850194505b838510156109e9578451808252602095860195909350016109cd565b50969550505050505056fea2646970667358221220805118a8c09110550cc714e34e4ff355e1c57cdd49a5af5e4ecfdbc8f3be464a64736f6c634300081c0033")]
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    contract PancakeV2SwapExamples {
        address private constant PANCAKESWAP_V2_ROUTER = 0x10ED43C718714eb63d5aA57B78B54704E256024E;

        address private constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
        address private constant BUSD = 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56;
        address private constant USDT = 0x55d398326f99059fF775485246999027B3197955;

        IUniswapV2Router private router = IUniswapV2Router(PANCAKESWAP_V2_ROUTER);
        IWETH private wbnb = IWETH(WBNB);
        IERC20 private usdt = IERC20(USDT);
        IERC20 private busd = IERC20(BUSD);

        // Swap WBNB to USDT
        function swapSingleHopExactAmountIn(uint256 amountIn, uint256 amountOutMin) external returns (uint256 amountOut) {
            wbnb.transferFrom(msg.sender, address(this), amountIn);
            wbnb.approve(address(router), amountIn);

            address[] memory path;
            path = new address[](2);
            path[0] = WBNB;
            path[1] = USDT;

            uint256[] memory amounts =
                router.swapExactTokensForTokens(amountIn, amountOutMin, path, msg.sender, block.timestamp);

            // amounts[0] = WBNB amount, amounts[1] = USDT amount
            return amounts[1];
        }

        // Swap USDT -> WBNB -> USDC
        function swapMultiHopExactAmountIn(uint256 amountIn, uint256 amountOutMin) external returns (uint256 amountOut) {
            usdt.transferFrom(msg.sender, address(this), amountIn);
            usdt.approve(address(router), amountIn);

            address[] memory path;
            path = new address[](3);
            path[0] = USDT;
            path[1] = WBNB;
            path[2] = BUSD;

            uint256[] memory amounts =
                router.swapExactTokensForTokens(amountIn, amountOutMin, path, msg.sender, block.timestamp);

            // amounts[0] = USDT amount
            // amounts[1] = WBNB amount
            // amounts[2] = USDC amount
            return amounts[2];
        }

        function doSwap() external payable returns (uint256[] memory) {
            require(msg.value >= 1e18, "Send 1 BNB or more");
            uint256 wbnbAmount = 1e18;
            wbnb.deposit{value: wbnbAmount}();
            wbnb.approve(address(this), wbnbAmount);

            // wbnb.transferFrom(msg.sender, address(this), wbnbAmount);
            wbnb.approve(address(router), wbnbAmount);

            address[] memory path;
            path = new address[](2);
            path[0] = WBNB;
            path[1] = USDT;

            uint256[] memory amounts = router.swapExactTokensForTokens(wbnbAmount, 0, path, msg.sender, block.timestamp);
            return amounts;
        }
    }
    receive() external payable {}
}

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    IERC20,
    "foundry_test/out/IERC20.sol/IERC20.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    IWETH,
    "foundry_test/out/IWETH.sol/IWETH.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    IUniswapV2Router,
    "foundry_test/out/IUniswapV2Router.sol/IUniswapV2Router.json"
);

// sol!("contract/SwapHelper.sol");

pub fn build_tx(
    to: Address,
    from: Address,
    calldata: Bytes,
    value: U256,
    base_fee: u128,
) -> TransactionRequest {
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

    // let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);
    // let provider = Arc::new(provider);

    let anvil = Anvil::new().fork(rpc_url).block_time(1_u64).spawn();
    let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint().parse()?);
    let anvil_provider = Arc::new(anvil_provider);

    // Create two users, Alice and Bob.
    let accounts = anvil_provider.get_accounts().await?;
    let alice = accounts[0];
    let bob = accounts[1];

    anvil_provider
        .anvil_set_nonce(alice, U256::from(1123))
        .await?;
    // let from = address!("0000000000000000000000000000000000000001");

    // Fund the address with BNB
    // let value = U256::from_str(&*parse_units("1000", 18).unwrap().to_string())?;
    // anvil_provider.anvil_set_balance(from, value).await?;

    // let contract = PancakeV2SwapExamples::deploy(anvil_provider.clone()).await?;
    // println!("Helper contract deployed at: {}", contract.address());

    // Prepare swap parameters
    // 1 bnb
    let amount_in = U256::from(10000000000000000000u128);
    anvil_provider
        .anvil_set_balance(alice, amount_in * U256::from(2000))
        .await?;
    let path = vec![WBNB_ADDRESS, USDT_ADDRESS];
    // let block = provider
    //     .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
    //     .await?
    //     .unwrap();

    let wbnb = IWETH::new(WBNB_ADDRESS, anvil_provider.clone());
    println!("WBNB deployed at: {}", wbnb.address());
    let usdt = IERC20::new(USDT_ADDRESS, anvil_provider.clone());
    println!("USDT deployed at: {}", usdt.address());
    let router = IUniswapV2Router::new(PANCAKESWAP_V2_ROUTER_ADDRESS, anvil_provider.clone());
    println!("Router deployed at: {}", router.address());

    let deposit_tx = wbnb
        .deposit()
        .value(amount_in * U256::from(20))
        .from(alice)
        .send()
        .await?;
    println!("Deposit tx sent: {:?}", deposit_tx.tx_hash());
    let receipt = deposit_tx.get_receipt().await?;
    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );
    let wbnb_balance = wbnb.balanceOf(alice).call().await?._0;
    println!("WBNB balance of alice: {wbnb_balance}");

    let approve_tx = wbnb
        .approve(*router.address(), amount_in * U256::from(20))
        .from(alice)
        .send()
        .await?;
    println!("Approve tx sent: {:?}", approve_tx.tx_hash());

    let receipt = approve_tx.get_receipt().await?;
    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );
    let block = anvil_provider
        .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
        .await?
        .unwrap();
    let deadline = U256::from(block.header.timestamp + 60);
    let swap_tx = router
        .swapExactTokensForTokens(
            amount_in * U256::from(20),
            U256::ZERO,
            path.clone(),
            alice,
            deadline,
        )
        .from(alice)
        .send()
        .await?;
    println!("Swap tx sent: {:?}", swap_tx.tx_hash());

    // Store the hash before getting the receipt since get_receipt consumes swap_tx
    let tx_hash = *swap_tx.tx_hash();

    let receipt = swap_tx.get_receipt().await?;
    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );

    let wbnb_balance = wbnb.balanceOf(alice).call().await?._0;
    println!("WBNB balance of Alice: {wbnb_balance}");
    let usdt_balance = usdt.balanceOf(alice).call().await?._0;
    println!("USDT balance of Alice: {usdt_balance}");

    // Use the stored hash instead of trying to access swap_tx again
    let default_options = GethDebugTracingOptions::default();
    let result = anvil_provider
        .debug_trace_transaction(tx_hash, default_options)
        .await?;

    println!("DEFAULT_TRACE: {result:?}");

    // let deposit_tx = wbnb.transfer(*contract.address(), amount_in * U256::from(20)).from(alice).send().await?;
    // println!("Deposited to contract tx sent: {:?}", deposit_tx.tx_hash());
    //
    // let receipt = deposit_tx.get_receipt().await?;
    // println!(
    //     "Transaction included in block {}",
    //     receipt.block_number.expect("Failed to get block number")
    // );
    //
    // let wbnb_balance = wbnb.balanceOf(*contract.address()).call().await?._0;
    // println!("WBNB balance of Contract: {wbnb_balance}");


    // Create the helper contract call
    // let helper_call = IUniswapV2Router::swapExactTokensForTokensCall::new((amount_in * U256::from(20), U256::ZERO, path.clone(), alice, U256::from(100)));
    //
    // let call_data = swapExactTokensForTokensCall::abi_encode(&helper_call).into();
    //
    // let gas_price = anvil_provider.get_gas_price().await?;
    // // println!("Gas price {gas_price}");
    // let tx = build_tx(*router.address(), alice, call_data, amount_in, gas_price + 1);
    //
    // let block = BlockId::Number(BlockNumberOrTag::Latest);
    // let trace_options = GethDebugTracingCallOptions::default();
    // let result = anvil_provider
    //     .debug_trace_call(tx.clone(), block, trace_options)
    //     .await?;
    // // let result = anvil_provider.call(&tx).await?;
    // println!("Swap simulation result: {:?}", result);
    // let amount_out = decode_quote_response(result)?;
    // println!("Amounts Out: {amount_out}");

    Ok(())
}
