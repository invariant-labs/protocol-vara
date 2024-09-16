import {
  MessageQueuedData,
  UserMessageSent,
  GearApi,
  HexString,
  ProgramMetadata
} from '@gear-js/api'
import * as wasmSerializer from './wasm-serializer.js'
import { ISubmittableResult, IKeyringPair } from '@polkadot/types/types'
import { SignerOptions, SubmittableExtrinsic } from '@polkadot/api/types'
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
  _positionToTick,
  _alignTickToSpacing,
  _calculateSqrtPrice
} from '@invariant-labs/vara-sdk-wasm'

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
  PositionTick,
  Tickmap,
  _calculateAmountDeltaResult,
  InvariantError,
  LiquidityBreakpoint,
  Liquidity
} from './schema.js'
import { Network } from './network.js'
import { CONCENTRATION_FACTOR, LOCAL, MAINNET, MAX_TICK_CROSS, TESTNET } from './consts.js'
export { HexString } from '@gear-js/api'

export type Signer = string | IKeyringPair
export type ActorId = Uint8Array | HexString

export const initGearApi = async (network: Network) => {
  let address
  switch (network) {
    case Network.Local:
      address = LOCAL
      break
    case Network.Testnet:
      address = TESTNET
      break
    case Network.Mainnet:
      address = MAINNET
      break
    default:
      throw new Error('Network unknown')
  }

  const gearApi = await GearApi.create({ providerAddress: address })

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
let nodeModules: typeof import('./node.js')

// This is necessary to avoid import issues on the fronted
const loadNodeModules = async () => {
  if (typeof window !== 'undefined') {
    throw new Error('cannot load node modules in a browser environment')
  }

  await import('./node.js')
    .then(node => {
      nodeModules = node
    })
    .catch(error => {
      console.error('error while loading node modules:', error)
    })
}

export const getWasm = async (contractName: string): Promise<any> => {
  await loadNodeModules()
  const __dirname = new URL('.', import.meta.url).pathname

  return nodeModules.readFile(
    nodeModules.join(__dirname, `../contracts/${contractName}/${contractName}.opt.wasm`)
  )
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

export const convertPositionTick = (tick: any): PositionTick => {
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

export const convertPositions = (positions: any): [Pool, [Position, number][]][] => {
  positions = positions.map(([pool, positions]: any[]) => {
    pool = convertPool(pool)
    positions = positions.map(([position, index]: [Position, number]) => {
      return [convertPosition(position), index]
    })
    return [pool, positions]
  })
  return positions
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
  private validateCallback: ((t: UserMessageSent) => string | null) | null = null
  constructor(txBuilder: ITransactionBuilder) {
    this.txBuilder = txBuilder
  }

  async signAndSend(): Promise<U> {
    try {
      const { response } = await this.txBuilder.signAndSend()
      if (this.decodeCallback) {
        return this.decodeCallback(await response())
      }

      return await response()
    } catch (e: any) {
      const message = e.message
      if (message) {
        throw e
      } else {
        throw JSON.stringify(e)
      }
    }
  }

  withAccount(signer: Signer): this {
    this.txBuilder.withAccount(signer)
    return this
  }

  withDecode(decodeFn: (t: any) => U): this {
    this.decodeCallback = decodeFn
    return this
  }

  withValidate(validateFn: (t: UserMessageSent) => string | null): this {
    this.validateCallback = validateFn
    return this
  }

  public get extrinsic(): SubmittableExtrinsic<'promise', ISubmittableResult> {
    return (this.txBuilder as any)._tx
  }

  public get validation(): ((t: UserMessageSent) => string | null) | null {
    return this.validateCallback
  }
}

export const validateFungibleTokenResponse = (message: UserMessageSent) => {
  const registry = new TypeRegistry()
  const json = registry.createType('(String, String, bool', message.data.message.payload)
  return json[2].isTrue ? null : 'Token response invalid'
}

const validateInvariantSingleTransfer = (message: UserMessageSent) => {
  try {
    const registry = new TypeRegistry()
    registry.createType('(String, String, U256)', message.data.message.payload)
  } catch (e) {
    // this may happen if the gas runs out during reply handling
    return 'Deposit response invalid'
  }
  return null
}
export const validateInvariantSingleDeposit = validateInvariantSingleTransfer
export const validateInvariantSingleWithdraw = validateInvariantSingleTransfer
export const validateInvariantVaraDeposit = validateInvariantSingleTransfer
export const validateInvariantVaraWithdraw = validateInvariantSingleTransfer

const validateInvariantPairTransfer = (message: UserMessageSent) => {
  try {
    const registry = new TypeRegistry()
    registry.createType('(String, String, (U256, U256))', message.data.message.payload)
  } catch (e) {
    // this may happen if the gas runs out during reply handling
    return 'Deposit response invalid'
  }
  return null
}

export const validateInvariantPairDeposit = validateInvariantPairTransfer
export const validateInvariantPairWithdraw = validateInvariantPairTransfer

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

export const calculatePriceImpact = (
  startingSqrtPrice: SqrtPrice,
  endingSqrtPrice: SqrtPrice
): Percentage => {
  const startingPrice = startingSqrtPrice * startingSqrtPrice
  const endingPrice = endingSqrtPrice * endingSqrtPrice
  const diff = startingPrice - endingPrice

  const nominator = diff > 0n ? diff : -diff
  const denominator = startingPrice > endingPrice ? startingPrice : endingPrice

  return (nominator * getPercentageDenominator()) / denominator
}

export const sqrtPriceToPrice = (sqrtPrice: SqrtPrice): Price => {
  return ((sqrtPrice * sqrtPrice) / getSqrtPriceDenominator()) as any
}

export const priceToSqrtPrice = (price: Price): SqrtPrice => {
  return sqrt(price * getSqrtPriceDenominator())
}

export const calculateLiquidityBreakpoints = (
  ticks: (Tick | LiquidityTick)[]
): LiquidityBreakpoint[] => {
  let currentLiquidity = 0n

  return ticks.map(tick => {
    currentLiquidity = currentLiquidity + tick.liquidityChange * (tick.sign ? 1n : -1n)
    return {
      liquidity: currentLiquidity,
      index: tick.index
    }
  })
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
  return convertFeeTier(_newFeeTier(fee, tickSpacing))
}

export const newPoolKey = (token0: HexString, token1: HexString, feeTier: FeeTier): PoolKey => {
  return convertPoolKey(_newPoolKey(token0, token1, feeTier))
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

export const calculateTick = (sqrtPrice: SqrtPrice, tickSpacing: bigint): bigint => {
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

export const calculateSqrtPrice = (tickIndex: bigint): bigint => {
  return _calculateSqrtPrice(tickIndex)
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

export const calculateFeeTierWithLinearRatio = (tickCount: bigint): FeeTier => {
  return newFeeTier(tickCount * toPercentage(1n, 4n), tickCount)
}

export const calculateConcentration = (tickSpacing: number, minimumRange: number, n: number) => {
  const concentration = 1 / (1 - Math.pow(1.0001, (-tickSpacing * (minimumRange + 2 * n)) / 4))
  return concentration / CONCENTRATION_FACTOR
}

export const calculateTickDelta = (
  tickSpacing: number,
  minimumRange: number,
  concentration: number
) => {
  const base = Math.pow(1.0001, -(tickSpacing / 4))
  const logArg =
    (1 - 1 / (concentration * CONCENTRATION_FACTOR)) /
    Math.pow(1.0001, (-tickSpacing * minimumRange) / 4)

  return Math.ceil(Math.log(logArg) / Math.log(base) / 2)
}

export const getConcentrationArray = (
  tickSpacing: number,
  minimumRange: number,
  currentTick: number
): number[] => {
  const concentrations: number[] = []
  let counter = 0
  let concentration = 0
  let lastConcentration = calculateConcentration(tickSpacing, minimumRange, counter) + 1
  let concentrationDelta = 1

  while (concentrationDelta >= 1) {
    concentration = calculateConcentration(tickSpacing, minimumRange, counter)
    concentrations.push(concentration)
    concentrationDelta = lastConcentration - concentration
    lastConcentration = concentration
    counter++
  }
  concentration = Math.ceil(concentrations[concentrations.length - 1])

  while (concentration > 1) {
    concentrations.push(concentration)
    concentration--
  }
  const maxTick = integerSafeCast(_alignTickToSpacing(getMaxTick(1n), tickSpacing))
  if ((minimumRange / 2) * tickSpacing > maxTick - Math.abs(currentTick)) {
    throw new Error(String(InvariantError.TickLimitReached))
  }
  const limitIndex =
    (maxTick - Math.abs(currentTick) - (minimumRange / 2) * tickSpacing) / tickSpacing

  return concentrations.slice(0, limitIndex)
}

export const calculateAmountDelta = (
  currentTickIndex: bigint,
  currentSqrtPrice: bigint,
  liquidity: bigint,
  roundingUp: boolean,
  upperTickIndex: bigint,
  lowerTickIndex: bigint
) => {
  const encodedLiquidity = wasmSerializer.encodeLiquidity(liquidity)

  const [x, y] = _calculateAmountDelta(
    currentTickIndex,
    currentSqrtPrice,
    encodedLiquidity,
    roundingUp,
    upperTickIndex,
    lowerTickIndex
  )

  return [wasmSerializer.decodeTokenAmount(x), wasmSerializer.decodeTokenAmount(y)]
}

export const calculateTokenAmountsWithSlippage = (
  tickSpacing: bigint,
  currentSqrtPrice: SqrtPrice,
  liquidity: Liquidity,
  lowerTickIndex: bigint,
  upperTickIndex: bigint,
  slippage: Percentage,
  roundingUp: boolean
): [bigint, bigint] => {
  const lowerBound = calculateSqrtPriceAfterSlippage(currentSqrtPrice, slippage, false)
  const upperBound = calculateSqrtPriceAfterSlippage(currentSqrtPrice, slippage, true)

  const currentTickIndex = calculateTick(currentSqrtPrice, tickSpacing)

  const [lowerX, lowerY] = calculateAmountDelta(
    currentTickIndex,
    lowerBound,
    liquidity,
    roundingUp,
    upperTickIndex,
    lowerTickIndex
  )
  const [upperX, upperY] = calculateAmountDelta(
    currentTickIndex,
    upperBound,
    liquidity,
    roundingUp,
    upperTickIndex,
    lowerTickIndex
  )

  const x = lowerX > upperX ? lowerX : upperX
  const y = lowerY > upperY ? lowerY : upperY
  return [x, y]
}

type MsgId = HexString
type BlockHash = any
type ProgramId = HexString
export class BatchError extends Error {
  failedTxs: Map<number, string>
  constructor(failedTxs: Map<number, string>) {
    let message = 'Batch error occurred'
    failedTxs.forEach(function (err, nr) {
      message = message + `\nRequest number ${nr} failed: ${err}`
    })

    super(message)
    this.failedTxs = failedTxs
  }
}
export const batchTxs = async (
  api: GearApi,
  account: Signer,
  transactions: TransactionWrapper<any>[],
  options: Partial<SignerOptions> = {}
) => {
  const methods = transactions.map(val => val.extrinsic)
  const validationCallbacks = transactions.map(val => val.validation)

  const tx = api.tx.utility.batchAll([...methods])

  await tx.signAsync(account, options)
  const res = await new Promise<[MsgId, BlockHash, ProgramId][]>((resolve, reject) =>
    tx
      .send(({ events, status }) => {
        if (status.isInBlock) {
          const msgData: [MsgId, BlockHash, ProgramId][] = []

          events.forEach(({ event }) => {
            const { method, section, data } = event
            if (method === 'MessageQueued' && section === 'gear') {
              const { id, destination } = data as MessageQueuedData
              msgData.push([id.toHex(), status.asInBlock.toHex(), destination.toHex()])
            } else if (method === 'ExtrinsicSuccess') {
              resolve(msgData)
            } else if (method === 'ExtrinsicFailed') {
              reject(api.getExtrinsicFailedError(event))
            }
          })
        }
      })
      .catch(error => {
        reject(error.message)
      })
  )

  const messages = await res

  const responsePromises: Promise<UserMessageSent>[] = messages.map(
    ([msgId, blockHash, programId]) => {
      return new Promise(async resolve => {
        const res = await api.message.getReplyEvent(programId, msgId as any, blockHash)
        resolve(res)
      })
    }
  )

  const responses: UserMessageSent[] = await Promise.all(responsePromises)
  const errors: Map<number, string> = new Map()
  for (let i = 0; i < responses.length; i++) {
    const response = responses[i]
    const message = response.data.message

    const validationCallback = validationCallbacks[i]

    if (!message.details.unwrap().code.isSuccess) {
      errors.set(i, api.registry.createType('String', message.payload).toString())
      continue
    }

    if (validationCallback) {
      const errorFromAdditionalCheck = validationCallback(response)
      if (errorFromAdditionalCheck) {
        errors.set(i, errorFromAdditionalCheck)
      }
    }
  }

  if (errors.size) {
    throw new BatchError(errors)
  }

  return responses
}
