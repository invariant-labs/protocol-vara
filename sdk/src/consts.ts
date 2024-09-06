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

export const enum Network {
  Local = 'Local',
  Testnet = 'Testnet',
  Mainnet = 'Mainnet'
}

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
  '0x7109e59e1d7259375c83d67315c2aadf2980ffbb6a9e7c7c0777616470308641'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x3e32c934bbac50a24735423f71d6953feff6ec203c7778d06d0cba37fec8a0cd'
export const TESTNET_ETH_ADDRESS: HexString =
  '0xcd0eb80c3bf278c3518a29797bd39d2e2a1f9ff187faf559f0847ae7434c7724'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x890f9ce7863c0d891bdd2c2bc7c8d00f898bd24b623f5f696b94db55ba5c5b8b'
