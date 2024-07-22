import {
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getMaxTickCross,
  getChunkSize,
  getLiquidityTickLimit,
  getMaxPoolKeysReturned,
  getPositionEntriesLimit,
  getMaxPoolPairsReturned
} from 'invariant-vara-wasm'
import { HexString } from './utils.js'

export const enum Network {
  Local = 'ws://127.0.0.1:9944',
  Testnet = 'wss://testnet.vara.network',
  Mainnet = 'wss://rpc.vara.network'
}
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

export const CONCENTRATION_FACTOR = 1.00001526069123

export const TESTNET_INVARIANT_ADDRESS: HexString =
  '0x4421a6dd2d9816e07d27ac9d90763d3802e2e2e55e84edbca99dab2030ec23e2'
export const TESTNET_BTC_ADDRESS: HexString =
  '0xda37bde0057b65433ef238223ecf2925a54c02c6544ca5dcf37dcbe0e8cad6c5'
export const TESTNET_ETH_ADDRESS: HexString =
  '0x56823efb6f0c792497e3f40778bd2d7dac50ecfa7f1e6750967f188ccc6e7ff2'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x8d1226c704360772880b1415a3ae05cde58ea61db04b2fac42fd9b9ade4e2aea'
