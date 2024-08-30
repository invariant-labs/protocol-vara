import {
  Invariant,
  GearKeyring,
  Network,
  FungibleToken,
  PoolKey,
  SwapEvent,
  TESTNET_INVARIANT_ADDRESS,
  calculateFee,
  initGearApi,
  positionToTick,
  simulateInvariantSwap
} from '@invariant-labs/vara-sdk'
import { CHUNK_SIZE } from '@invariant-labs/vara-sdk/target/consts.js'
import assert from 'assert'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initGearApi(network)

  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)

  const POSITION_ID = 0n
  const SWAP_AMOUNT = 1000000n

  const invariant = await Invariant.load(api, TESTNET_INVARIANT_ADDRESS)
  const positionBefore = await invariant.getPosition(account.addressRaw, POSITION_ID)
  const grc20 = await FungibleToken.load(api)
  grc20.setAdmin(account)

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await grc20.mint(account.addressRaw, SWAP_AMOUNT, positionBefore.poolKey.tokenX)
  await grc20.approve(
    account,
    TESTNET_INVARIANT_ADDRESS,
    SWAP_AMOUNT,
    positionBefore.poolKey.tokenX
  )

  await grc20.mint(account.addressRaw, SWAP_AMOUNT, positionBefore.poolKey.tokenY)
  await grc20.approve(
    account,
    TESTNET_INVARIANT_ADDRESS,
    SWAP_AMOUNT,
    positionBefore.poolKey.tokenY
  )
  await invariant.depositSingleToken(account, positionBefore.poolKey.tokenX, SWAP_AMOUNT)
  const {
    tickmap: tickmapBeforeFirstSwap,
    ticks: ticksBeforeFirstSwap,
    pool: poolBeforeFirstSwap
  } = await getPoolState(invariant, positionBefore.poolKey)

  const firstSimulation = simulateInvariantSwap(
    tickmapBeforeFirstSwap,
    positionBefore.poolKey.feeTier,
    poolBeforeFirstSwap,
    ticksBeforeFirstSwap,
    true,
    SWAP_AMOUNT,
    true,
    0n
  )
  const firstSwapResult = await invariant.swap(
    account,
    positionBefore.poolKey,
    true,
    SWAP_AMOUNT,
    true,
    0n
  )

  await invariant.withdrawSingleToken(account, positionBefore.poolKey.tokenY, null)

  firstSwapResult
  assert(firstSimulation.globalInsufficientLiquidity === false)
  assert(firstSimulation.maxTicksCrossed === false)
  assert(firstSimulation.stateOutdated === false)
  assert(firstSimulation.amountIn == firstSwapResult.amountIn)
  assert(firstSimulation.amountOut == firstSwapResult.amountOut)
  assert(firstSimulation.crossedTicks.length === 0)
  assert(firstSimulation.startSqrtPrice === firstSwapResult.startSqrtPrice)
  assert(firstSimulation.targetSqrtPrice === firstSwapResult.targetSqrtPrice)

  const {
    tickmap: tickmapBeforeSecondSwap,
    ticks: ticksBeforeSecondSwap,
    pool: poolBeforeSecondSwap
  } = await getPoolState(invariant, positionBefore.poolKey)

  const secondSimulation = simulateInvariantSwap(
    tickmapBeforeSecondSwap,
    positionBefore.poolKey.feeTier,
    poolBeforeSecondSwap,
    ticksBeforeSecondSwap,
    false,
    SWAP_AMOUNT,
    true,
    2n ** 128n - 1n
  )
  await invariant.depositSingleToken(account, positionBefore.poolKey.tokenY, SWAP_AMOUNT)
  const secondSwapResult = await invariant.swap(
    account,
    positionBefore.poolKey,
    false,
    SWAP_AMOUNT,
    true,
    2n ** 128n - 1n
  )
  await invariant.withdrawSingleToken(account, positionBefore.poolKey.tokenX, null)

  assert(secondSimulation.globalInsufficientLiquidity === false)
  assert(secondSimulation.maxTicksCrossed === false)
  assert(secondSimulation.stateOutdated === false)
  assert(secondSimulation.amountIn === secondSwapResult.amountIn)
  assert(secondSimulation.amountOut === secondSwapResult.amountOut)
  assert(secondSimulation.crossedTicks.length === 0)
  assert(secondSimulation.startSqrtPrice === secondSwapResult.startSqrtPrice)
  assert(secondSimulation.targetSqrtPrice === secondSwapResult.targetSqrtPrice)

  const pool = await invariant.getPool(
    positionBefore.poolKey.tokenX,
    positionBefore.poolKey.tokenY,
    positionBefore.poolKey.feeTier
  )
  console.log('Pool:', pool)
  const positionAfter = await invariant.getPosition(account.addressRaw, POSITION_ID)
  const lowerTick = await invariant.getTick(positionBefore.poolKey, positionBefore.lowerTickIndex)
  const upperTick = await invariant.getTick(positionBefore.poolKey, positionBefore.upperTickIndex)
  console.log('Fees:', calculateFee(pool, positionAfter, lowerTick, upperTick))

  process.exit(0)
}
const getPoolState = async (invariant: Invariant, poolKey: PoolKey) => {
  console.log(poolKey)
  const tickmap = await invariant.getTickmap(poolKey)
  const promises = []
  for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
    for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
      const checkedBit = chunk & (1n << bit)
      if (checkedBit) {
        const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
        promises.push(invariant.getTick(poolKey, tickIndex))
      }
    }
  }

  const ticks = await Promise.all(promises)

  const pool = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
  return { tickmap, ticks, pool }
}
main()
