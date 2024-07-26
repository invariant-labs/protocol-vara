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
  '0xcfefd7f97acb9ac09c2b9e9ad5e01bedb68abaddd69345a3562d018e831bd02c'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x6bfed5e399cb1d63c31289027a19704acfa30ef35f691ea8d2d1e931e05baa64'
export const TESTNET_ETH_ADDRESS: HexString =
  '0x9ad6750601e79d0d8de510c81334ef88835447c17d277a4edd74657c51edaae1'
export const TESTNET_USDC_ADDRESS: HexString =
  '0xdcfc1c8508562ef21ae01e2693716d0631e86322497312560f0f6eae9ef88307'
