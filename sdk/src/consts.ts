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
  '0xfa8d8c38cd06205b54b5b0e7c6cc6584252f001865c6bf7604d94a241cb523ed'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x5117d1886af96c070a9c29093a432e6924d839b20d6e04042d40dc4723a59390'
export const TESTNET_ETH_ADDRESS: HexString =
  '0x321d733690439b32922b58ec0d148844e448ea1d8636fea435f61871061fa04a'
export const TESTNET_USDC_ADDRESS: HexString =
  '0xd5e714be0ac00dea0e33168af0ab99964c3338fea6a3569fb00a416704976f3b'
