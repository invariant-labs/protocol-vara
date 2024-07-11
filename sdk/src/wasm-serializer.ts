import {
  Pool,
  Position,
  QuoteResult,
  Tick,
  calculateAmountDeltaResult,
  _calculateFeeResult,
  TokenAmounts,
  SwapResult,
  SingleTokenLiquidity,
  LiquidityTick,
  LiquidityResult,
  AmountDeltaResult,
  CalculateSwapResult
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

export const encodeLiquidity = encodeU256ToU64Array;
export const encodeTokenAmount = encodeU256ToU64Array;

export const decodeLiquidity = decodeU256FromU64Array;
export const decodeTokenAmount = decodeU256FromU64Array;

export const encodeTick = (value: Tick) => {
  value.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  value.liquidityGross = encodeLiquidity(value.liquidityGross) as any
  return value
}

export const encodeLiquidityTick = (value: LiquidityTick) => {
  value.liquidityChange = encodeLiquidity(value.liquidityChange) as any
  return value
}

export const encodePool = (value: Pool) => {
  value.liquidity = encodeLiquidity(value.liquidity) as any
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

export const encodeQuoteResult = (value: QuoteResult) => {
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
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
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.feeAmount = encodeTokenAmount(value.feeAmount) as any
  return value
}

export const encodeCalculateSwapResult = (value: CalculateSwapResult) => {
  value.amountIn = encodeTokenAmount(value.amountIn) as any
  value.amountOut = encodeTokenAmount(value.amountOut) as any
  value.fee = encodeTokenAmount(value.fee) as any
  value.pool = encodePool(value.pool) as any
  value.ticks = value.ticks.map(encodeTick)

  return value
}

export const encodeCalculateAmountDeltaResult = (value: calculateAmountDeltaResult) => {
  value[0] = encodeTokenAmount(value[0]) as any
  value[1] = encodeTokenAmount(value[1]) as any

  return value
}

export const encodePosition = (value: Position) => {
  value.liquidity = encodeLiquidity(value.liquidity) as any
  value.tokensOwedX = encodeTokenAmount(value.tokensOwedX) as any
  value.tokensOwedY = encodeTokenAmount(value.tokensOwedY) as any
  return value
}

export const decodeTick = (value: Tick) => {
  value.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  value.liquidityGross = decodeLiquidity(value.liquidityGross as any)
  return value
}

export const decodeLiquidityTick = (value: LiquidityTick) => {
  value.liquidityChange = decodeLiquidity(value.liquidityChange as any)
  return value

}

export const decodePool = (value: Pool) => {
  value.liquidity = decodeLiquidity(value.liquidity as any)
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

export const decodeQuoteResult = (value: QuoteResult) => {
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
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
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.feeAmount = decodeTokenAmount(value.feeAmount as any)
  return value
}

export const decodeCalculateSwapResult = (value: CalculateSwapResult) => {
  value.amountIn = decodeTokenAmount(value.amountIn as any)
  value.amountOut = decodeTokenAmount(value.amountOut as any)
  value.fee = decodeTokenAmount(value.fee as any)
  value.pool = decodePool(value.pool as any)
  value.ticks = value.ticks.map(decodeTick)
  return value
}

export const decodeCalculateAmountDeltaResult = (value: calculateAmountDeltaResult) => {
  value[0] = decodeTokenAmount(value[0] as any)
  value[1] = decodeTokenAmount(value[1] as any)
  return value
}

export const decodePosition = (value: Position) => {
  value.liquidity = decodeLiquidity(value.liquidity as any)
  value.tokensOwedX = decodeTokenAmount(value.tokensOwedX as any)
  value.tokensOwedY = decodeTokenAmount(value.tokensOwedY as any)
  return value
}
