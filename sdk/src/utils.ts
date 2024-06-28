import { GearApi, GearApiOptions, HexString, ProgramMetadata } from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'
import { IKeyringPair } from '@polkadot/types/types'
import {
  _calculateFee,
  _newFeeTier,
  _newPoolKey,
  calculateAmountDelta,
  calculateAmountDeltaResult,
  getPercentageDenominator,
  getSqrtPriceDenominator
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
  Tick
} from './schema.js'
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

export const newFeeTier = (fee: Percentage, tickSpacing: bigint): FeeTier => {
  return convertFeeTier(_newFeeTier(fee, integerSafeCast(tickSpacing)))
}

export const newPoolKey = (token0: HexString, token1: HexString, feeTier: FeeTier): PoolKey => {
  return convertPoolKey(_newPoolKey(token0, token1, feeTier))
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

export const convertFeeTier = (feeTier: any): FeeTier => {
  return convertFieldsToBigInt(feeTier, ['tickSpacing'])
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

export const calculateTokenAmounts = (
  pool: Pool,
  position: Position
): calculateAmountDeltaResult => {
  return _calculateTokenAmounts(pool, position, false)
}

export const _calculateTokenAmounts = (
  pool: Pool,
  position: Position,
  sign: boolean
): calculateAmountDeltaResult => {
  return calculateAmountDelta(
    pool.currentTickIndex,
    pool.sqrtPrice,
    position.liquidity,
    sign,
    position.upperTickIndex,
    position.lowerTickIndex
  )
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
  return (sqrtPrice * sqrtPrice) / getSqrtPriceDenominator()
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

  const multiplier = getPercentageDenominator() + (up ? slippage : -slippage)
  const price = sqrtPriceToPrice(sqrtPrice)
  const priceWithSlippage = price * multiplier * getPercentageDenominator()
  const sqrtPriceWithSlippage = priceToSqrtPrice(priceWithSlippage) / getPercentageDenominator()

  return sqrtPriceWithSlippage
}

export const delay = (delayMs: number) => {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}

export const calculateFee = (
  pool: Pool,
  position: Position,
  lowerTick: Tick,
  upperTick: Tick
): [TokenAmount, TokenAmount] => {
  return _calculateFee(
    lowerTick.index,
    lowerTick.feeGrowthOutsideX,
    lowerTick.feeGrowthOutsideY,
    upperTick.index,
    upperTick.feeGrowthOutsideX,
    upperTick.feeGrowthOutsideY,
    pool.currentTickIndex,
    pool.feeGrowthGlobalX,
    pool.feeGrowthGlobalY,
    position.feeGrowthInsideX,
    position.feeGrowthInsideY,
    position.liquidity
  )
}