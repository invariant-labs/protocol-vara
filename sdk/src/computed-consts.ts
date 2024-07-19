import { FeeTier } from './schema.js'
import { calculateFeeTierWithLinearRatio, getConcentrationArray, integerSafeCast } from './utils.js'

export const FEE_TIERS: FeeTier[] = [
  calculateFeeTierWithLinearRatio(1n),
  calculateFeeTierWithLinearRatio(2n),
  calculateFeeTierWithLinearRatio(5n),
  calculateFeeTierWithLinearRatio(10n),
  calculateFeeTierWithLinearRatio(30n),
  calculateFeeTierWithLinearRatio(100n)
]

export const CONCENTRATION_ARRAY: { [key: string]: number[] } = FEE_TIERS.reduce((acc, tier) => {
  acc[integerSafeCast(tier.tickSpacing)] = getConcentrationArray(
    integerSafeCast(tier.tickSpacing),
    2,
    0
  ).sort((a, b) => a - b)
  return acc
}, {} as { [key: string]: number[] })