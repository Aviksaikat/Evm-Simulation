// SPDX-License-Identifier: MIT
pragma solidity ^0.8.16;

interface IERC20 {
    function approve(address spender, uint256 amount) external returns (bool);
}

interface IWETH {
    function deposit() external payable;
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

contract SwapHelper {
    address constant PANCAKESWAP_ROUTER = 0x10ED43C718714eb63d5aA57B78B54704E256024E;

    function swapBNBForTokens(
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external payable {
        require(path[0] == 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c, "First token must be WBNB");

        // Wrap BNB to WBNB
        IWETH wbnb = IWETH(path[0]);
        wbnb.deposit{value: msg.value}();

        // Approve router
        IERC20(path[0]).approve(PANCAKESWAP_ROUTER, msg.value);

        // Perform swap
        IUniswapV2Router(PANCAKESWAP_ROUTER).swapExactTokensForTokens(
            msg.value,
            amountOutMin,
            path,
            to,
            deadline
        );
    }

    // Allow contract to receive BNB
    receive() external payable {}
}
