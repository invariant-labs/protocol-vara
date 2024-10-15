import {
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getMaxSwapStep,
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
  getTokenAmountDenominator,
  getTickSearchRange
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
export const MAX_SWAP_STEPS = getMaxSwapStep()
export const SEARCH_RANGE = getTickSearchRange()
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
  '0x7fcabbac9e6f2bacffd9003cd3ab13516e401b85791db0aa4763d696fc2ffc58'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x1ac44dec403ba8b11175c7daad872793777534d28d33370d3e04cdfeace42a39'
export const TESTNET_ETH_ADDRESS: HexString =
  '0xe237e35656960b740fcc492725fa5f9a5d8c506c92c5ba47782205294b28ef5a'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x917d419347ddea81d76dbbb43ac9f6426809a80c400f346f12b6964e7dfa5ba5'
export const TESTNET_SOL_ADDRESS: HexString =
  '0x0d3b71c84950eb9d33a55af4a52523fffdb821601694653e7d038fb70de7fed6'
export const TESTNET_AZERO_ADDRESS: HexString =
  '0x7d25efa0ec2eb985367f359bd8a213f10aad78cfa0c020f7642f8393cc69d773'

