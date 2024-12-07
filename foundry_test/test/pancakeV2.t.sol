// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test, console2} from "forge-std/Test.sol";
import {PancakeV2SwapExamples} from "../src/PancakeV2SwapExamples.sol";
import {IERC20} from "../src/interfaces/IERC20.sol";
import {IWETH} from "../src/interfaces/IWETH.sol";

address constant WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
address constant BUSD = 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56;
address constant USDT = 0x55d398326f99059fF775485246999027B3197955;

contract PancakeV2SwapExamplesTest is Test {
    IWETH private wbnb = IWETH(WBNB);
    IERC20 private usdt = IERC20(USDT);
    IERC20 private busd = IERC20(BUSD);

    PancakeV2SwapExamples private pancake = new PancakeV2SwapExamples();

    function setUp() public {}

    // Swap WETH -> USDT
    function testPancakeSwapSingleHopExactAmountIn() public {
        uint256 wbnbAmount = 1e18;
        wbnb.deposit{value: wbnbAmount}();
        wbnb.approve(address(pancake), wbnbAmount);

        uint256 usdtAmountMin = 1;
        uint256 usdtAmountOut = pancake.swapSingleHopExactAmountIn(wbnbAmount, usdtAmountMin);

        console2.log("USDT", usdtAmountOut / 1e18);
        assertGe(usdtAmountOut, usdtAmountMin, "amount out < min");
    }

    // Swap USDT -> WBNB -> BUSD
    function testPancakeSwapMultiHopExactAmountIn() public {
        // Swap WBNB -> USDT
        uint256 wbnbAmount = 1e18;
        wbnb.deposit{value: wbnbAmount}();
        wbnb.approve(address(pancake), wbnbAmount);

        uint256 usdtAmountMin = 1;
        pancake.swapSingleHopExactAmountIn(wbnbAmount, usdtAmountMin);

        // Swap USDT -> WETH -> BUSD
        uint256 usdtAmountIn = 1e18;
        usdt.approve(address(pancake), usdtAmountIn);

        uint256 usdcAmountOutMin = 1;
        uint256 usdcAmountOut = pancake.swapMultiHopExactAmountIn(usdtAmountIn, usdcAmountOutMin);

        console2.log("BUSD", usdcAmountOut);
        assertGe(usdcAmountOut, usdcAmountOutMin, "amount out < min");
    }

    function testDoSwap() external {
        uint256[] memory AmountsOut = pancake.doSwap{value: 1 ether}();

        // for (uint i = 0; i < AmountsOut.length; i++){
        //     console2.log("Output", AmountsOut[i]);
        // }
        console2.log("Amounts:", AmountsOut[0], AmountsOut[1]);
    }
}
