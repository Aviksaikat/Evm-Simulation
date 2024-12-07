// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface IWETH {
    function deposit() external payable;
    function approve(address spender, uint256 amount) external returns (bool);
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

interface IERC20 {
    function approve(address spender, uint256 amount) external returns (bool);
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

interface IUniswapV2Router {
    function swapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external returns (uint256[] memory amounts);
}

contract PancakeSwapV2SwapExamples {
    address private constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
    address private constant DAI = 0x55d398326f99059fF775485246999027B3197955;
    address private constant USDT = 0x55d398326f99059fF775485246999027B3197955;

    // pancakeswap
    IUniswapV2Router private constant router = IUniswapV2Router(0x10ED43C718714eb63d5aA57B78B54704E256024E);
    IERC20 private constant dai = IERC20(DAI);
    IERC20 private constant usdt = IERC20(USDT);
    IWETH private constant wbnb = IWETH(WBNB);

    // Event to log the swap result
    event SwapResult(uint256[] amountOut);

    function swapMultiHopExactAmountIn(uint256 amountIn, uint256 amountOutMin)
    external
    returns (uint256[] memory result)
    {
        dai.transferFrom(msg.sender, address(this), amountIn);
        dai.approve(address(router), amountIn);

        address[] memory path = new address[](3);
        path[0] = DAI;
        path[1] = WBNB;
        path[2] = USDT;

        uint256[] memory amounts = router.swapExactTokensForTokens(
            amountIn, amountOutMin, path, msg.sender, block.timestamp
        );

        return amounts;
    }

    // Test function that combines both swaps
    function testSwapMultiHopExactAmountIn() external payable {
        require(msg.value >= 1 ether, "Send at least 1 ETH");

        // Step 1: Wrap ETH to WETH
        wbnb.deposit{value: msg.value}();

        // Step 2: Approve WETH to UniswapV2SwapExamples
        wbnb.approve(address(this), msg.value);

        // Step 3: Swap WETH -> DAI (using a simple helper function)
        uint256 daiAmount = _swapWBNBToDAI(msg.value);

        // Step 4: Approve DAI to this contract
        dai.approve(address(this), daiAmount);

        // Step 5: Execute the multi-hop swap (DAI -> WETH -> USDC)
        uint256 usdcAmountOutMin = 1;
        uint256[] memory usdcAmountOut = this.swapMultiHopExactAmountIn(daiAmount, usdcAmountOutMin);

        emit SwapResult(usdcAmountOut);
    }

    // Helper function to swap WETH to DAI
    function _swapWBNBToDAI(uint256 wethAmount) internal returns (uint256) {
        wbnb.approve(address(router), wethAmount);

        address[] memory path = new address[](2);
        path[0] = WBNB;
        path[1] = DAI;

        uint256[] memory amounts = router.swapExactTokensForTokens(
            wethAmount,
            1, // min amount
            path,
            address(this),
            block.timestamp
        );

        return amounts[1];
    }

    // Helper functions to check balances
    function getDAIBalance(address account) external view returns (uint256) {
        return dai.balanceOf(account);
    }

    function getUSDTBalance(address account) external view returns (uint256) {
        return usdt.balanceOf(account);
    }

    function getWBNBBalance(address account) external view returns (uint256) {
        return wbnb.balanceOf(account);
    }

    // Allow the contract to receive ETH
    receive() external payable {}
}