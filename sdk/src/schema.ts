import { Liquidity } from '@invariant-labs/vara-sdk-wasm'

export {
  Liquidity,
  PoolKey,
  SqrtPrice,
  TokenAmount,
  CrossTickEvent,
  PositionRemovedEvent,
  PositionCreatedEvent,
  SwapEvent,
  CalculateSwapResult,
  FeeTier,
  Percentage,
  Pool,
  Position,
  Tick,
  Price,
  QuoteResult,
  FeeGrowth,
  SecondsPerLiquidity,
  AmountDeltaResult,
  LiquidityResult,
  LiquidityTick,
  PositionTick,
  SingleTokenLiquidity,
  SwapHop,
  SwapResult,
  TokenAmounts,
  _calculateFeeResult,
  _calculateAmountDeltaResult,
  Tickmap,
  SimulateSwapResult,
  InvariantError,
} from '@invariant-labs/vara-sdk-wasm'

export enum InvariantEvent {
  CrossTickEvent = 'CrossTickEvent',
  SwapEvent = 'SwapEvent',
  PositionCreatedEvent = 'PositionCreatedEvent',
  PositionRemovedEvent = 'PositionRemovedEvent'
}

export interface LiquidityBreakpoint {
  liquidity: Liquidity
  index: bigint
}
