// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IERC20} from "./interfaces/IERC20.sol";
import {IWETH} from "./interfaces/IWETH.sol";
import {IUniswapV2Router} from "./interfaces/IUniswapV2Router.sol";

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

    receive() external payable {}
}
