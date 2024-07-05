import {
  FeeTier,
  Pool,
  PoolKey,
  Position,
  PositionTick,
  QuoteResult,
  Tick,
  _calculateAmountDeltaResult,
  _calculateFeeResult,
  TokenAmounts,
  SwapResult,
  SwapHop,
  SingleTokenLiquidity,
  LiquidityTick,
  LiquidityResult,
  AmountDeltaResult,
  CalculateSwapResult,
  SimulateSwapResult
} from './schema.js'

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

export const encodeFeeGrowth = encodeU128ToU64Array;
export const encodeFixedPoint = encodeU128ToU64Array;
export const encodePercentage = encodeU128ToU64Array;
export const encodePrice = encodeU128ToU64Array;
export const encodeSecondsPerLiquidity = encodeU128ToU64Array;
export const encodeSqrtPrice = encodeU128ToU64Array;

export const encodeLiquidity = encodeU256ToU64Array;
export const encodeTokenAmount = encodeU256ToU64Array;

export const decodeFeeGrowth = decodeU128FromU64Array;
export const decodeFixedPoint = decodeU128FromU64Array;
export const decodePercentage = decodeU128FromU64Array;
export const decodePrice = decodeU128FromU64Array;
export const decodeSecondsPerLiquidity = decodeU128FromU64Array;
export const decodeSqrtPrice = decodeU128FromU64Array;

export const decodeLiquidity = decodeU256FromU64Array;
export const decodeTokenAmount = decodeU256FromU64Array;

export const encodeTick = (value: Tick) => {
  value.feeGrowthOutsideX = encodeFeeGrowth(value.feeGrowthOutsideX) as any
  value.feeGrowthOutsideY = encodeFeeGrowth(value.feeGrowthOutsideY) as any
  value.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  value.liquidityGross = encodeLiquidity(value.liquidityGross) as any
  value.sqrtPrice = encodeSqrtPrice(value.sqrtPrice) as any
  return value
}

export const encodePositionTick = (value: PositionTick) => {
  value.feeGrowthOutsideX = encodeFeeGrowth(value.feeGrowthOutsideX) as any
  value.feeGrowthOutsideY = encodeFeeGrowth(value.feeGrowthOutsideY) as any
  return value

}

export const encodeLiquidityTick = (value: LiquidityTick) => {
  value.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  return value
}

export const encodePool = (value: Pool) => {
  value.liquidity = encodeLiquidity(value.liquidity) as any
  value.sqrtPrice = encodeSqrtPrice(value.sqrtPrice) as any
  value.feeGrowthGlobalX = encodeFeeGrowth(value.feeGrowthGlobalX) as any
  value.feeGrowthGlobalY = encodeFeeGrowth(value.feeGrowthGlobalY) as any
  value.feeProtocolTokenX = encodeTokenAmount(value.feeProtocolTokenX) as any
  value.feeProtocolTokenY = encodeTokenAmount(value.feeProtocolTokenY) as any
  return value
}

export const encodeLiquidityResult = (value: LiquidityResult) => {
  value.x = encodeTokenAmount(value.x) as any
  value.y = encodeTokenAmount(value.y) as any
  value.l = encodeTokenAmount(value.l) as any
  return value

}

export const encodeSingleTokenLiquidity = (value: SingleTokenLiquidity) => {
  value.amount = encodeTokenAmount(value.amount) as any
  value.l = encodeLiquidity(value.l) as any
  return value

}

export const encodeSingleSwapHop = (value: SwapHop) => {
  value.poolKey = encodePoolKey(value.poolKey) as any
  return value
}

export const encodeQuoteResult = (value: QuoteResult) => {
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.targetSqrtPrice = encodeSqrtPrice(value.targetSqrtPrice) as any
  value.ticks = value.ticks.map(encodeTick)
  return value
}

export const encodeTokenAmounts = (value: TokenAmounts) => {
  value.x = encodeTokenAmount(value.x) as any
  value.y = encodeTokenAmount(value.y) as any
  return value
}

export const encodeCalculateFeeResult = (value: _calculateFeeResult) => {
  value[0] = encodeTokenAmount(value[0]) as any
  value[1] = encodeTokenAmount(value[1]) as any
  return value
}

export const encodeAmountDeltaResult = (value: AmountDeltaResult) => {
  value.x = encodeTokenAmount(value.x) as any
  value.y = encodeTokenAmount(value.y) as any
  return value
}

export const encodeSwapResult = (value: SwapResult) => {
  value.nextSqrtPrice = encodeSqrtPrice(value.nextSqrtPrice) as any
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.feeAmount = encodeTokenAmount(value.feeAmount) as any
  return value
}

export const encodeCalculateSwapResult = (value: CalculateSwapResult) => {
  value.startSqrtPrice = encodeSqrtPrice(value.startSqrtPrice) as any
  value.targetSqrtPrice = encodeSqrtPrice(value.targetSqrtPrice) as any
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.fee = encodeTokenAmount(value.fee) as any
  value.pool = encodePool(value.pool) as any
  value.ticks = value.ticks.map(encodeTick)

  return value
}

export const encodeCalculateAmountDeltaResult = (value: _calculateAmountDeltaResult) => {
  value[0] = encodeTokenAmount(value[0]) as any
  value[1] = encodeTokenAmount(value[1]) as any

  return value
}

export const encodePoolKey = (value: PoolKey) => {
  value.feeTier = encodeFeeTier(value.feeTier) as any

  return value
}

export const encodePosition = (value: Position) => {
  value.poolKey = encodePoolKey(value.poolKey) as any
  value.liquidity = encodeLiquidity(value.liquidity) as any
  value.feeGrowthInsideX = encodeFeeGrowth(value.feeGrowthInsideX) as any
  value.feeGrowthInsideY = encodeFeeGrowth(value.feeGrowthInsideY) as any
  value.tokensOwedX = encodeTokenAmount(value.tokensOwedX) as any
  value.tokensOwedY = encodeTokenAmount(value.tokensOwedY) as any
  return value
}

export const encodeFeeTier = (value: FeeTier) => {
  value.fee = encodePercentage(value.fee) as any
  return value
}

export const encodeSimulateSwapResult = (value: SimulateSwapResult) => {
  value.startSqrtPrice = encodeSqrtPrice(value.startSqrtPrice)  as any
  value.targetSqrtPrice = encodeSqrtPrice(value.targetSqrtPrice)  as any
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.fee = encodeTokenAmount(value.fee) as any
  value.crossedTicks = value.crossedTicks.map(encodeLiquidityTick)
  return value
}

export const decodeTick = (value: Tick) => {
  value.feeGrowthOutsideX = decodeFeeGrowth(value.feeGrowthOutsideX as any)
  value.feeGrowthOutsideY = decodeFeeGrowth(value.feeGrowthOutsideY as any)
  value.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  value.liquidityGross = decodeLiquidity(value.liquidityGross as any)
  value.sqrtPrice = decodeSqrtPrice(value.sqrtPrice as any)
  return value
}

export const decodePositionTick = (value: PositionTick) => {
  value.feeGrowthOutsideX = decodeFeeGrowth(value.feeGrowthOutsideX as any)
  value.feeGrowthOutsideY = decodeFeeGrowth(value.feeGrowthOutsideY as any)
  return value

}

export const decodeLiquidityTick = (value: LiquidityTick) => {
  value.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  return value

}

export const decodePool = (value: Pool) => {
  value.liquidity = decodeLiquidity(value.liquidity as any)
  value.sqrtPrice = decodeSqrtPrice(value.sqrtPrice as any)
  value.feeGrowthGlobalX = decodeFeeGrowth(value.feeGrowthGlobalX as any)
  value.feeGrowthGlobalY = decodeFeeGrowth(value.feeGrowthGlobalY as any)
  value.feeProtocolTokenX = decodeTokenAmount(value.feeProtocolTokenX as any)
  value.feeProtocolTokenY = decodeTokenAmount(value.feeProtocolTokenY as any)
  return value

}

export const decodeLiquidityResult = (value: LiquidityResult) => {
  value.x = decodeTokenAmount(value.x as any)
  value.y = decodeTokenAmount(value.y as any)
  value.l = decodeTokenAmount(value.l as any)
  return value

}

export const decodeSingleTokenLiquidity = (value: SingleTokenLiquidity) => {
  value.amount = decodeTokenAmount(value.amount as any)
  value.l = decodeLiquidity(value.l as any)
  return value

}

export const decodeSingleSwapHop = (value: SwapHop) => {
  value.poolKey = decodePoolKey(value.poolKey as any)
  return value

}

export const decodeQuoteResult = (value: QuoteResult) => {
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.targetSqrtPrice = decodeSqrtPrice(value.targetSqrtPrice as any)
  value.ticks = value.ticks.map(decodeTick)
  return value
}

export const decodeTokenAmounts = (value: TokenAmounts) => {
  value.x = decodeTokenAmount(value.x as any)
  value.y = decodeTokenAmount(value.y as any)
  return value
}

export const decodeCalculateFeeResult = (value: _calculateFeeResult) => {
  value[0] = decodeTokenAmount(value[0] as any)
  value[1] = decodeTokenAmount(value[1] as any)
  return value
}

export const decodeAmountDeltaResult = (value: AmountDeltaResult) => {
  value.x = decodeTokenAmount(value.x as any)
  value.y = decodeTokenAmount(value.y as any)
  return value
}

export const decodeSwapResult = (value: SwapResult) => {
  value.nextSqrtPrice = decodeSqrtPrice(value.nextSqrtPrice as any)
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.feeAmount = decodeTokenAmount(value.feeAmount as any)
  return value
}

export const decodeCalculateSwapResult = (value: CalculateSwapResult) => {
  value.startSqrtPrice = decodeSqrtPrice(value.startSqrtPrice as any)
  value.targetSqrtPrice = decodeSqrtPrice(value.targetSqrtPrice as any)
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.fee = decodeTokenAmount(value.fee as any)
  value.pool = decodePool(value.pool as any)
  value.ticks = value.ticks.map(decodeTick)
  return value
}

export const decodeCalculateAmountDeltaResult = (value: _calculateAmountDeltaResult) => {
  value[0] = decodeTokenAmount(value[0] as any)
  value[1] = decodeTokenAmount(value[1] as any)
  return value
}

export const decodePoolKey = (value: PoolKey) => {
  value.feeTier = decodeFeeTier(value.feeTier as any)
  return value
}

export const decodePosition = (value: Position) => {
  value.poolKey = decodePoolKey(value.poolKey as any)
  value.liquidity = decodeLiquidity(value.liquidity as any)
  value.feeGrowthInsideX = decodeFeeGrowth(value.feeGrowthInsideX as any)
  value.feeGrowthInsideY = decodeFeeGrowth(value.feeGrowthInsideY as any)
  value.tokensOwedX = decodeTokenAmount(value.tokensOwedX as any)
  value.tokensOwedY = decodeTokenAmount(value.tokensOwedY as any)
  return value
}

export const decodeFeeTier = (value: FeeTier) => {
  value.fee = decodePercentage(value.fee as any)
  value.tickSpacing = BigInt(value.tickSpacing)
  return value
}

export const decodeSimulateSwapResult = (value: SimulateSwapResult) => {
  value.startSqrtPrice = decodeSqrtPrice(value.startSqrtPrice as any)
  value.targetSqrtPrice = decodeSqrtPrice(value.targetSqrtPrice as any)
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.fee = decodeTokenAmount(value.fee as any)
  value.crossedTicks = value.crossedTicks.map(decodeLiquidityTick)
  return value
}