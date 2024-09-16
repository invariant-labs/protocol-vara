import {
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getMaxTickCross,
  getChunkSize,
  getLiquidityTickLimit,
  getMaxPoolKeysReturned,
  getPositionEntriesLimit,
  getMaxPoolPairsReturned,
  getPriceScale,
  getPercentageScale,
  getLiquidityScale,
  getFixedPointScale,
  getFeeGrowthScale,
  getSecondsPerLiquidityScale,
  getSqrtPriceScale,
  getTokenAmountScale,
  getFeeGrowthDenominator,
  getFixedPointDenominator,
  getLiquidityDenominator,
  getPercentageDenominator,
  getPriceDenominator,
  getSecondsPerLiquidityDenominator,
  getSqrtPriceDenominator,
  getTokenAmountDenominator
} from '@invariant-labs/vara-sdk-wasm'
import { HexString } from './utils.js'

export const LOCAL = 'ws://127.0.0.1:9944'
export const TESTNET = 'wss://testnet.vara.network'
export const MAINNET = 'wss://rpc.vara.network'

export const FUNGIBLE_TOKEN_GAS_LIMIT = 750_000_000_000n
export const INVARIANT_GAS_LIMIT = 750_000_000_000n
export const DEFAULT_ADDRESS = '5F3sa2TJAWMqDhXG6jhV4N8ko9SxwGy8TpaNS1repo5EYjQX'

export const MAX_SQRT_PRICE = getGlobalMaxSqrtPrice()
export const MIN_SQRT_PRICE = getGlobalMinSqrtPrice()
export const MAX_TICK_CROSS = getMaxTickCross()
export const CHUNK_SIZE = getChunkSize()
export const LIQUIDITY_TICKS_LIMIT = getLiquidityTickLimit()
export const MAX_POOL_PAIRS_RETURNED = getMaxPoolPairsReturned()
export const MAX_POOL_KEYS_RETURNED = getMaxPoolKeysReturned()
export const POSITIONS_ENTRIES_LIMIT = getPositionEntriesLimit()

export const FEE_GROWTH_DENOMINATOR = getFeeGrowthDenominator()
export const FIXED_POINT_DENOMINATOR = getFixedPointDenominator()
export const LIQUIDITY_DENOMINATOR = getLiquidityDenominator()
export const PERCENTAGE_DENOMINATOR = getPercentageDenominator()
export const PRICE_DENOMINATOR = getPriceDenominator()
export const SECONDS_PER_LIQUIDITY_DENOMINATOR = getSecondsPerLiquidityDenominator()
export const SQRT_PRICE_DENOMINATOR = getSqrtPriceDenominator()
export const TOKEN_AMOUNT_DENOMINATOR = getTokenAmountDenominator()

export const FEE_GROWTH_SCALE = getFeeGrowthScale()
export const FIXED_POINT_SCALE = getFixedPointScale()
export const LIQUIDITY_SCALE = getLiquidityScale()
export const PERCENTAGE_SCALE = getPercentageScale()
export const PRICE_SCALE = getPriceScale()
export const SECONDS_PER_LIQUIDITY_SCALE = getSecondsPerLiquidityScale()
export const SQRT_PRICE_SCALE = getSqrtPriceScale()
export const TOKEN_AMOUNT_SCALE = getTokenAmountScale()

export const CONCENTRATION_FACTOR = 1.00001526069123

export const VARA_ADDRESS: HexString =
  '0x0000000000000000000000000000000000000000000000000000000000000000'
export const TESTNET_INVARIANT_ADDRESS: HexString =
  '0xbe6b7d437dafbbdd983a153ca594ae32abebcd42450b4914a01fcbd6d7a59264'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x3d35d6eee42ee2b89490295e591d205ac94f01446b4a27a8ba21522f474caffc'
export const TESTNET_ETH_ADDRESS: HexString =
  '0xbbef6ca66fd7ec80bd7800b0c1b5b67dbef041931c5e76a0f318ef3025868711'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x79973b6c7cec41ea24101d604d2d33d9aa3d9482581dd57701a50773c8822cfc'
export const TESTNET_SOL_ADDRESS: HexString =
  '0x41c273ed8b4eb1d559bd709fba24608730f83b0225d58b55ee6bd067fc3b40aa'
export const TESTNET_AZERO_ADDRESS: HexString =
  '0x7e0bf68153c2444c762b9cd9fec0e8ff4f8f48c733db64dbdf9336354af6ffa4'

