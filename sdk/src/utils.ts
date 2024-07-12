import { GearApi, GearApiOptions, HexString, ProgramMetadata } from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'
import * as wasmSerializer from './wasm-serializer.js'
import { IKeyringPair } from '@polkadot/types/types'
import {
  _calculateFee,
  _newFeeTier,
  _newPoolKey,
  _calculateAmountDelta,
  _getLiquidityByX,
  _getLiquidityByY,
  _calculateTick,
  _isTokenX,
  getPercentageDenominator,
  getSqrtPriceDenominator,
  _getMinSqrtPrice,
  _getMinTick,
  _getMaxChunk,
  _getMaxSqrtPrice,
  _getMaxTick,
  _toFeeGrowth,
  _toFixedPoint,
  _toLiquidity,
  _toPercentage,
  _toPrice,
  _toSecondsPerLiquidity,
  _toSqrtPrice,
  _toTokenAmount,
  _simulateInvariantSwap,
  tickIndexToPosition,
  _positionToTick
} from 'invariant-vara-wasm'

import { TypeRegistry } from '@polkadot/types'
import {
  TokenAmount,
  Price,
  QuoteResult,
  SqrtPrice,
  CrossTickEvent,
  InvariantEvent,
  SwapEvent,
  PositionCreatedEvent,
  PositionRemovedEvent,
  CalculateSwapResult,
  FeeTier,
  Percentage,
  Pool,
  PoolKey,
  Position,
  Tick,
  LiquidityTick,
  Tickmap,
  _calculateAmountDeltaResult
} from './schema.js'
import { MAX_TICK_CROSS } from './consts.js'
export type Signer = string | IKeyringPair
export type ActorId = Uint8Array | HexString

export const initGearApi = async (gearApiOptions: GearApiOptions | undefined) => {
  const gearApi = await GearApi.create(gearApiOptions)

  const [chain, nodeName, nodeVersion] = await Promise.all([
    gearApi.chain(),
    gearApi.nodeName(),
    gearApi.nodeVersion()
  ])

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`)

  return gearApi
}

// returns usnub function
export const subscribeToNewHeads = async (api: GearApi): Promise<VoidFunction> => {
  return await api.blocks.subscribeNewHeads(header => {
    console.log(
      `New block with number: ${header.number.toNumber()} and hash: ${header.hash.toHex()}`
    )
  })
}

export const getWasm = async (contractName: string): Promise<Buffer> => {
  const __dirname = new URL('.', import.meta.url).pathname

  return readFile(path.join(__dirname, `../contracts/${contractName}/${contractName}.opt.wasm`))
}

export const createTypeByName = (meta: ProgramMetadata, type: string, payload: any) => {
  return meta.createType(meta.getTypeIndexByName(type)!, payload)
}

export const integerSafeCast = (value: bigint): number => {
  if (value > BigInt(Number.MAX_SAFE_INTEGER) || value < BigInt(Number.MIN_SAFE_INTEGER)) {
    throw new Error('Integer value is outside the safe range for Numbers')
  }
  return Number(value)
}

export type Result<T> = { ok: T } | { err: string }
export const unwrapResult = <T>(result: Result<T>): T => {
  if ('ok' in result) {
    return result.ok
  } else if (result.err) {
    throw new Error(result.err)
  } else {
    throw new Error('Invalid Result type')
  }
}

const convertFieldsToBigInt = (returnedObject: any, exclude?: string[]): any => {
  for (const [key, value] of Object.entries(returnedObject)) {
    if (exclude?.includes(key)) {
      continue
    }
    if (typeof value === 'number' || typeof value === 'string') {
      returnedObject[key] = BigInt(value as any)
    }
  }
  return returnedObject
}

export const convertTick = (tick: any): Tick => {
  return convertFieldsToBigInt(tick)
}

export const convertLiquidityTick = (tick: any): LiquidityTick => {
  return convertTick(tick)
}

export const convertFeeTier = (feeTier: any): FeeTier => {
  return convertFieldsToBigInt(feeTier)
}

export const convertPoolKey = (poolKey: any): PoolKey => {
  poolKey.feeTier = convertFeeTier(poolKey.feeTier)
  return poolKey
}

export const convertPool = (pool: any): Pool => {
  return convertFieldsToBigInt(pool, ['currentIndex', 'feeReceiver'])
}

export const convertPosition = (position: any): Position => {
  position = convertFieldsToBigInt(position, ['poolKey'])
  position.poolKey = convertPoolKey(position.poolKey)
  return position as Position
}

export const convertPositionCreatedEvent = (positionEvent: any): PositionCreatedEvent => {
  positionEvent = convertFieldsToBigInt(positionEvent, ['address', 'poolKey'])
  positionEvent.poolKey = convertPoolKey(positionEvent.poolKey)
  return positionEvent as PositionCreatedEvent
}

export const convertPositionRemovedEvent = (positionEvent: any): PositionRemovedEvent => {
  positionEvent = convertFieldsToBigInt(positionEvent, ['address', 'poolKey'])
  positionEvent.poolKey = convertPoolKey(positionEvent.poolKey)
  return positionEvent as PositionRemovedEvent
}

export const convertSwapEvent = (swapEvent: any): SwapEvent => {
  swapEvent = convertFieldsToBigInt(swapEvent, ['address', 'poolKey'])
  swapEvent.poolKey = convertPoolKey(swapEvent.poolKey)

  return swapEvent as SwapEvent
}

export const convertCrossTickEvent = (crossTickEvent: any): CrossTickEvent => {
  crossTickEvent = convertFieldsToBigInt(crossTickEvent, ['address', 'indexes', 'poolKey'])
  crossTickEvent.poolKey = convertPoolKey(crossTickEvent.poolKey)
  crossTickEvent.indexes = crossTickEvent.indexes.map((index: any) => BigInt(index))

  return crossTickEvent as CrossTickEvent
}

export const convertCalculateSwapResult = (calculateSwapResult: any): CalculateSwapResult => {
  calculateSwapResult = convertFieldsToBigInt(calculateSwapResult, ['pool', 'ticks'])
  calculateSwapResult.pool = convertPool(calculateSwapResult.pool)
  calculateSwapResult.ticks = calculateSwapResult.ticks.map(convertTick)

  return calculateSwapResult
}

export const convertQuoteResult = (quoteResult: any): QuoteResult => {
  quoteResult = convertFieldsToBigInt(quoteResult, ['ticks'])
  quoteResult.ticks = quoteResult.ticks.map(convertTick)
  return quoteResult
}

export interface IMethodReturnType<T> {
  msgId: HexString
  blockHash: HexString
  txHash: HexString
  response: () => Promise<T>
}

export interface ITransactionBuilder {
  signAndSend(): Promise<IMethodReturnType<any>>
  withAccount(signer: Signer): this
}

export class TransactionWrapper<U> {
  private txBuilder: ITransactionBuilder
  private decodeCallback: ((t: any) => U) | null = null
  constructor(txBuilder: ITransactionBuilder) {
    this.txBuilder = txBuilder
  }

  async send(): Promise<U> {
    const { response } = await this.txBuilder.signAndSend()
    if (this.decodeCallback) {
      return this.decodeCallback(await response())
    }

    return await response()
  }

  withAccount(signer: Signer): this {
    this.txBuilder.withAccount(signer)
    return this
  }

  withDecode(decodeFn: (t: any) => U): this {
    this.decodeCallback = decodeFn
    return this
  }
}

export type SwapEventCallback = {
  ident: InvariantEvent.SwapEvent
  callback: (event: SwapEvent) => void | Promise<void>
}
export type CrossTickEventCallback = {
  ident: InvariantEvent.CrossTickEvent
  callback: (event: CrossTickEvent) => void | Promise<void>
}
export type PositionRemovedEventCallback = {
  ident: InvariantEvent.PositionRemovedEvent
  callback: (event: PositionRemovedEvent) => void | Promise<void>
}
export type PositionCreatedEventCallback = {
  ident: InvariantEvent.PositionCreatedEvent
  callback: (event: PositionCreatedEvent) => void | Promise<void>
}

export type InvariantEventCallback =
  | SwapEventCallback
  | CrossTickEventCallback
  | PositionRemovedEventCallback
  | PositionCreatedEventCallback
export const decodeEvent = (registry: TypeRegistry, payload: HexString, prefix: string): any => {
  let type: string
  let convertFunction

  switch (prefix as InvariantEvent) {
    case InvariantEvent.PositionCreatedEvent:
      type =
        '(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","liquidityDelta":"Liquidity","lowerTick":"i32","upperTick":"i32","sqrtPrice":"SqrtPrice"})'
      convertFunction = convertPositionCreatedEvent
      break
    case InvariantEvent.PositionRemovedEvent:
      type =
        '(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","liquidityDelta":"Liquidity","lowerTick":"i32","upperTick":"i32","sqrtPrice":"SqrtPrice"})'
      convertFunction = convertPositionRemovedEvent
      break
    case InvariantEvent.CrossTickEvent:
      type =
        '(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","indexes":"Vec<i32>"})'
      convertFunction = convertCrossTickEvent

      break
    case InvariantEvent.SwapEvent:
      type =
        '(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","amountIn":"TokenAmount","amountOut":"TokenAmount","fee":"TokenAmount","startSqrtPrice":"SqrtPrice","targetSqrtPrice":"SqrtPrice","xToY":"bool"})'
      convertFunction = convertSwapEvent
      break
  }
  const event = (registry.createType(type, payload) as any)[2].toJSON() as any

  return convertFunction(event)
}

const sqrt = (value: bigint): bigint => {
  if (value < 0n) {
    throw 'square root of negative numbers is not supported'
  }

  if (value < 2n) {
    return value
  }

  return newtonIteration(value, 1n)
}

const newtonIteration = (n: bigint, x0: bigint): bigint => {
  const x1 = (n / x0 + x0) >> 1n
  if (x0 === x1 || x0 === x1 - 1n) {
    return x0
  }
  return newtonIteration(n, x1)
}
export const sqrtPriceToPrice = (sqrtPrice: SqrtPrice): Price => {
  return ((sqrtPrice * sqrtPrice) / getSqrtPriceDenominator()) as any
}

export const priceToSqrtPrice = (price: Price): SqrtPrice => {
  return sqrt(price * getSqrtPriceDenominator())
}

export const calculateSqrtPriceAfterSlippage = (
  sqrtPrice: SqrtPrice,
  slippage: Percentage,
  up: boolean
): SqrtPrice => {
  if (slippage === 0n) {
    return sqrtPrice
  }

  const percentageDenominator = getPercentageDenominator()
  const multiplier = percentageDenominator + (up ? slippage : -slippage)
  const price = sqrtPriceToPrice(sqrtPrice as any)
  const priceWithSlippage = price * multiplier * percentageDenominator
  const sqrtPriceWithSlippage = priceToSqrtPrice(priceWithSlippage) / percentageDenominator

  return sqrtPriceWithSlippage
}

export function filterTicks<T extends Tick | LiquidityTick>(
  ticks: T[],
  tickIndex: bigint,
  xToY: boolean
): T[] {
  const filteredTicks = new Array(...ticks)
  let tickCount = 0

  for (const [index, tick] of filteredTicks.entries()) {
    if (tickCount >= MAX_TICK_CROSS) {
      break
    }

    if (xToY) {
      if (tick.index > tickIndex) {
        filteredTicks.splice(index, 1)
      }
    } else {
      if (tick.index < tickIndex) {
        filteredTicks.splice(index, 1)
      }
    }
    tickCount++
  }

  return filteredTicks
}

export function filterTickmap(
  tickmap: Tickmap,
  tickSpacing: bigint,
  index: bigint,
  xToY: boolean
): Tickmap {
  const filteredTickmap = new Map(tickmap.bitmap)
  const [currentChunkIndex] = tickIndexToPosition(index, tickSpacing)
  let tickCount = 0
  for (const [chunkIndex] of filteredTickmap) {
    if (tickCount >= MAX_TICK_CROSS) {
      break
    }

    if (xToY) {
      if (chunkIndex > currentChunkIndex) {
        filteredTickmap.delete(chunkIndex)
      }
    } else {
      if (chunkIndex < currentChunkIndex) {
        filteredTickmap.delete(chunkIndex)
      }
    }
    tickCount++
  }

  return { bitmap: filteredTickmap }
}

export const delay = (delayMs: number) => {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}

export const calculateTokenAmounts = (
  pool: Pool,
  position: Position
): _calculateAmountDeltaResult => {
  return _calculateTokenAmounts(pool, position, false)
}

export const _calculateTokenAmounts = (
  pool: Pool,
  position: Position,
  sign: boolean
): _calculateAmountDeltaResult => {
  return wasmSerializer.decodeCalculateAmountDeltaResult(
    _calculateAmountDelta(
      pool.currentTickIndex,
      pool.sqrtPrice,
      wasmSerializer.encodeLiquidity(position.liquidity),
      sign,
      position.upperTickIndex,
      position.lowerTickIndex
    )
  )
}

export const newFeeTier = (fee: Percentage, tickSpacing: bigint): FeeTier => {
  return _newFeeTier(fee, integerSafeCast(tickSpacing))
}

export const newPoolKey = (token0: HexString, token1: HexString, feeTier: FeeTier): PoolKey => {
  return _newPoolKey(token0, token1, feeTier)
}

export const calculateFee = (
  pool: Pool,
  position: Position,
  lowerTick: Tick,
  upperTick: Tick
): [TokenAmount, TokenAmount] => {
  return _calculateFee(
    lowerTick.index,
    lowerTick.feeGrowthOutsideX as any,
    lowerTick.feeGrowthOutsideY as any,
    upperTick.index,
    upperTick.feeGrowthOutsideX as any,
    upperTick.feeGrowthOutsideY as any,
    pool.currentTickIndex,
    pool.feeGrowthGlobalX as any,
    pool.feeGrowthGlobalY as any,
    position.feeGrowthInsideX as any,
    position.feeGrowthInsideY as any,
    wasmSerializer.encodeLiquidity(position.liquidity as any)
  ).map(wasmSerializer.decodeTokenAmount)
}

export const getLiquidityByX = (
  amountX: TokenAmount,
  lowerTick: bigint,
  upperTick: bigint,
  sqrtPrice: SqrtPrice,
  roundingUp: boolean
) => {
  return wasmSerializer.decodeSingleTokenLiquidity(
    _getLiquidityByX(
      wasmSerializer.encodeTokenAmount(amountX),
      lowerTick,
      upperTick,
      sqrtPrice,
      roundingUp
    )
  )
}

export const getLiquidityByY = (
  amountY: TokenAmount,
  lowerTick: bigint,
  upperTick: bigint,
  sqrtPrice: SqrtPrice,
  roundingUp: boolean
) => {
  return wasmSerializer.decodeSingleTokenLiquidity(
    _getLiquidityByY(
      wasmSerializer.encodeTokenAmount(amountY),
      integerSafeCast(lowerTick),
      integerSafeCast(upperTick),
      sqrtPrice,
      roundingUp
    )
  )
}

export const calculateTick = (sqrtPrice: SqrtPrice, tickSpacing: bigint): number => {
  return _calculateTick(sqrtPrice, tickSpacing)
}

export const isTokenX = (token0: HexString, token1: HexString): boolean => {
  return _isTokenX(token0, token1)
}

export const getMinSqrtPrice = (tickSpacing: bigint): SqrtPrice => {
  return _getMinSqrtPrice(tickSpacing) as any
}

export const getMaxSqrtPrice = (tickSpacing: bigint): SqrtPrice => {
  return _getMaxSqrtPrice(tickSpacing) as any
}

export const getMaxChunk = (tickSpacing: bigint): bigint => {
  return BigInt(_getMaxChunk(tickSpacing))
}

export const getMaxTick = (tickSpacing: bigint): bigint => {
  return BigInt(_getMaxTick(tickSpacing))
}

export const getMinTick = (tickSpacing: bigint): bigint => {
  return BigInt(_getMinTick(tickSpacing))
}

export const toFeeGrowth = (val: bigint, scale: bigint): bigint => {
  return _toFeeGrowth(val, integerSafeCast(scale))
}

export const toLiquidity = (val: bigint, scale: bigint): bigint => {
  return _toLiquidity(val, integerSafeCast(scale))
}

export const toFixedPoint = (val: bigint, scale: bigint): bigint => {
  return _toFixedPoint(val, integerSafeCast(scale))
}

export const toPercentage = (val: bigint, scale: bigint): bigint => {
  return _toPercentage(val, integerSafeCast(scale))
}

export const toPrice = (val: bigint, scale: bigint): bigint => {
  return _toPrice(val, integerSafeCast(scale))
}

export const toSecondsPerLiquidity = (val: bigint, scale: bigint): bigint => {
  return _toSecondsPerLiquidity(val, integerSafeCast(scale))
}

export const toSqrtPrice = (val: bigint, scale: bigint): bigint => {
  return _toSqrtPrice(val, integerSafeCast(scale))
}

export const toTokenAmount = (val: bigint, scale: bigint): bigint => {
  return _toTokenAmount(val, integerSafeCast(scale))
}

export const positionToTick = (chunk: bigint, bit: bigint, tickSpacing: bigint): bigint => {
  return BigInt(
    _positionToTick(integerSafeCast(chunk), integerSafeCast(bit), integerSafeCast(tickSpacing))
  )
}

export const simulateInvariantSwap = (
  tickmap: Tickmap,
  feeTier: FeeTier,
  pool: Pool,
  liquidityTicks: LiquidityTick[],
  xToY: boolean,
  amount: bigint,
  byAmountIn: boolean,
  sqrtPriceLimit: bigint
) => {
  return wasmSerializer.decodeSimulateSwapResult(
    _simulateInvariantSwap(
      tickmap,
      feeTier,
      wasmSerializer.encodePool(pool),
      liquidityTicks.map(wasmSerializer.encodeLiquidityTick),
      xToY,
      wasmSerializer.encodeTokenAmount(amount) as any,
      byAmountIn,
      sqrtPriceLimit as any
    )
  )
}
