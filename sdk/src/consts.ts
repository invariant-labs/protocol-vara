import {
  getLiquidityTickLimit,
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getMaxTickCross,
  getChunkSize
} from 'invariant-vara-wasm'
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
