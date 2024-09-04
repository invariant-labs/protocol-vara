import {
  Pool,
  Position,
  QuoteResult,
  Tick,
  _calculateAmountDeltaResult,
  _calculateFeeResult,
  TokenAmounts,
  SwapResult,
  SingleTokenLiquidity,
  LiquidityTick,
  LiquidityResult,
  AmountDeltaResult,
  CalculateSwapResult,
  SimulateSwapResult
} from './schema.js'

function copyObject(obj: any): any {
  if (obj === null || typeof obj !== 'object') {
    return obj
  }

  if (Array.isArray(obj)) {
    return obj.map(copyObject)
  }

  const copiedObject = {} as any
  for (const key in obj) {
    copiedObject[key] = copyObject(obj[key])
  }

  return copiedObject
}

export const decodeU256FromU64Array = (value: string): bigint => {
  return BigInt(value)
}

export const decodeU128FromU64Array = (value: string): bigint => {
  return BigInt(value)
}

export const encodeU256ToU64Array = (value: bigint): string => {
  return value.toString()
}

export const encodeU128ToU64Array = (value: bigint): string => {
  return value.toString()
}

export const encodeLiquidity = encodeU256ToU64Array
export const encodeTokenAmount = encodeU256ToU64Array

export const decodeLiquidity = decodeU256FromU64Array
export const decodeTokenAmount = decodeU256FromU64Array

export const encodeTick = (value: Tick) => {
  const newVal = copyObject(value)
  newVal.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  newVal.liquidityGross = encodeLiquidity(value.liquidityGross) as any
  return newVal
}

export const encodeLiquidityTick = (value: LiquidityTick) => {
  const newVal = copyObject(value)
  newVal.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  return newVal
}

export const encodePool = (value: Pool) => {
  const newVal = copyObject(value)
  newVal.liquidity = encodeLiquidity(value.liquidity) as any
  newVal.feeProtocolTokenX = encodeTokenAmount(value.feeProtocolTokenX) as any
  newVal.feeProtocolTokenY = encodeTokenAmount(value.feeProtocolTokenY) as any
  return newVal
}

export const encodeLiquidityResult = (value: LiquidityResult) => {
  const newVal = copyObject(value)
  newVal.x = encodeTokenAmount(value.x) as any
  newVal.y = encodeTokenAmount(value.y) as any
  newVal.l = encodeTokenAmount(value.l) as any
  return newVal
}

export const encodeSingleTokenLiquidity = (value: SingleTokenLiquidity) => {
  const newVal = copyObject(value)
  newVal.amount = encodeTokenAmount(value.amount) as any
  newVal.l = encodeLiquidity(value.l) as any
  return newVal
}

export const encodeQuoteResult = (value: QuoteResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = encodeTokenAmount(value.amountIn) as any
  newVal.amountOut = encodeTokenAmount(value.amountOut) as any
  newVal.ticks = value.ticks.map(encodeTick)
  return newVal
}

export const encodeTokenAmounts = (value: TokenAmounts) => {
  const newVal = copyObject(value)
  newVal.x = encodeTokenAmount(value.x) as any
  newVal.y = encodeTokenAmount(value.y) as any
  return newVal
}

export const encodeCalculateFeeResult = (value: _calculateFeeResult) => {
  const newVal = copyObject(value)
  newVal[0] = encodeTokenAmount(value[0]) as any
  newVal[1] = encodeTokenAmount(value[1]) as any
  return newVal
}

export const encodeAmountDeltaResult = (value: AmountDeltaResult) => {
  const newVal = copyObject(value)
  newVal.x = encodeTokenAmount(value.x) as any
  newVal.y = encodeTokenAmount(value.y) as any
  return newVal
}

export const encodeSwapResult = (value: SwapResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = encodeTokenAmount(value.amountIn) as any
  newVal.amountOut = encodeTokenAmount(value.amountOut) as any
  newVal.feeAmount = encodeTokenAmount(value.feeAmount) as any
  return newVal
}

export const encodeCalculateSwapResult = (value: CalculateSwapResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = encodeTokenAmount(value.amountIn) as any
  newVal.amountOut = encodeTokenAmount(value.amountOut) as any
  newVal.fee = encodeTokenAmount(value.fee) as any
  newVal.pool = encodePool(value.pool) as any
  newVal.ticks = value.ticks.map(encodeTick)
  return newVal
}

export const encodeCalculateAmountDeltaResult = (value: _calculateAmountDeltaResult) => {
  const newVal = copyObject(value)
  newVal[0] = encodeTokenAmount(value[0]) as any
  newVal[1] = encodeTokenAmount(value[1]) as any

  return newVal
}

export const encodePosition = (value: Position) => {
  const newVal = copyObject(value)
  newVal.liquidity = encodeLiquidity(value.liquidity) as any
  newVal.tokensOwedX = encodeTokenAmount(value.tokensOwedX) as any
  newVal.tokensOwedY = encodeTokenAmount(value.tokensOwedY) as any
  return newVal
}

export const decodeTick = (value: Tick) => {
  const newVal = copyObject(value)
  newVal.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  newVal.liquidityGross = decodeLiquidity(value.liquidityGross as any)
  return newVal
}

export const decodeLiquidityTick = (value: LiquidityTick) => {
  const newVal = copyObject(value)
  newVal.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  return newVal
}

export const decodePool = (value: Pool) => {
  const newVal = copyObject(value)
  newVal.liquidity = decodeLiquidity(value.liquidity as any)
  newVal.feeProtocolTokenX = decodeTokenAmount(value.feeProtocolTokenX as any)
  newVal.feeProtocolTokenY = decodeTokenAmount(value.feeProtocolTokenY as any)
  return newVal
}

export const decodeLiquidityResult = (value: LiquidityResult) => {
  const newVal = copyObject(value)
  newVal.x = decodeTokenAmount(value.x as any)
  newVal.y = decodeTokenAmount(value.y as any)
  newVal.l = decodeTokenAmount(value.l as any)
  return newVal
}

export const decodeSingleTokenLiquidity = (value: SingleTokenLiquidity) => {
  const newVal = copyObject(value)
  newVal.amount = decodeTokenAmount(value.amount as any)
  newVal.l = decodeLiquidity(value.l as any)
  return newVal
}

export const decodeQuoteResult = (value: QuoteResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = decodeTokenAmount(value.amountIn as any)
  newVal.amountOut = decodeTokenAmount(value.amountOut as any)
  newVal.ticks = value.ticks.map(decodeTick)
  return newVal
}

export const decodeTokenAmounts = (value: TokenAmounts) => {
  const newVal = copyObject(value)
  newVal.x = decodeTokenAmount(value.x as any)
  newVal.y = decodeTokenAmount(value.y as any)
  return newVal
}

export const decodeCalculateFeeResult = (value: _calculateFeeResult) => {
  const newVal = copyObject(value)
  newVal[0] = decodeTokenAmount(value[0] as any)
  newVal[1] = decodeTokenAmount(value[1] as any)
  return newVal
}

export const decodeAmountDeltaResult = (value: AmountDeltaResult) => {
  const newVal = copyObject(value)

  newVal.x = decodeTokenAmount(value.x as any)
  newVal.y = decodeTokenAmount(value.y as any)
  return newVal
}

export const decodeSwapResult = (value: SwapResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = decodeTokenAmount(value.amountIn as any)
  newVal.amountOut = decodeTokenAmount(value.amountOut as any)
  newVal.feeAmount = decodeTokenAmount(value.feeAmount as any)
  return newVal
}

export const decodeCalculateSwapResult = (value: CalculateSwapResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = decodeTokenAmount(value.amountIn as any)
  newVal.amountOut = decodeTokenAmount(value.amountOut as any)
  newVal.fee = decodeTokenAmount(value.fee as any)
  newVal.pool = decodePool(value.pool as any)
  newVal.ticks = value.ticks.map(decodeTick)
  return newVal
}

export const decodeCalculateAmountDeltaResult = (value: _calculateAmountDeltaResult) => {
  const newVal = copyObject(value)
  newVal[0] = decodeTokenAmount(value[0] as any)
  newVal[1] = decodeTokenAmount(value[1] as any)
  return newVal
}

export const decodePosition = (value: Position) => {
  const newVal = copyObject(value)
  newVal.liquidity = decodeLiquidity(value.liquidity as any)
  newVal.tokensOwedX = decodeTokenAmount(value.tokensOwedX as any)
  newVal.tokensOwedY = decodeTokenAmount(value.tokensOwedY as any)
  return newVal
}

export const decodeSimulateSwapResult = (value: SimulateSwapResult) => {
  const newVal = copyObject(value)
  newVal.amountIn = decodeTokenAmount(value.amountIn as any)
  newVal.amountOut = decodeTokenAmount(value.amountOut as any)
  newVal.fee = decodeTokenAmount(value.fee as any)
  newVal.crossedTicks = value.crossedTicks.map(decodeLiquidityTick)
  return newVal
}
