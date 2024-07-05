import { delay, getMaxSqrtPrice, getMinSqrtPrice, integerSafeCast } from '../src/utils'
import { Keyring } from '@polkadot/api'
import { expect } from 'chai'
import { Network, MAX_SQRT_PRICE, MIN_SQRT_PRICE, MAX_TICK_CROSS } from '../src/consts'
import { Invariant } from '../src/invariant'
import { FungibleToken } from '../src/erc20'
import { assertThrowsAsync } from '../src/test-utils'
import {
  filterTickmap,
  filterTicks,
  initGearApi,
  newFeeTier,
  newPoolKey,
  simulateInvariantSwap
} from '../src/utils'
import { describe, it } from 'mocha'
import { HexString } from '@gear-js/api'

const api = await initGearApi({ providerAddress: Network.Local })
import { subscribeToNewHeads } from '../src/utils'
const keyring = new Keyring({ type: 'sr25519' })
const admin = await keyring.addFromUri('//Alice')

const protocolFee = 10000000000n
let unsub: Promise<VoidFunction> = null as any
let invariant: Invariant = null as any
let token0Address: HexString = null as any
let token1Address: HexString = null as any
const GRC20 = await FungibleToken.load(api)
GRC20.setAdmin(admin)

const maxTokenAmount = 2n ** 256n - 1n;
const feeTier = newFeeTier(10000000000n, 1n)

describe('simulateInvariantSwap', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
  beforeEach(async function () {
    this.timeout(80000)

    invariant = await Invariant.deploy(api, admin, protocolFee)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await GRC20.mint(admin.addressRaw, 10000000000000n, token0Address)
    await GRC20.mint(admin.addressRaw, 10000000000000n, token1Address)
    await GRC20.approve(admin, invariant.programId(), 10000000000000n, token0Address)
    await GRC20.approve(admin, invariant.programId(), 10000000000000n, token1Address)
    await delay(1000) // response fails to parse occasionally without the timeout
    await invariant.depositTokenPair  (
      admin,
      [token0Address, 10000000000000n],
      [token1Address, 10000000000000n]
    )

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      1000000000000000000000000n,
      0n
    )
  })
  context('reaches price limit', async function () {
    it('X to Y by amount in', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)

      const amountIn = 6000n
      const byAmountIn = true
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('Y to X by amount in', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('Y to X', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 5000n
      const byAmountIn = false
      const xToY = false
      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('X to Y', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
      const amountIn = 5000n
      const byAmountIn = false
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })
  })

  context('matches the price', async function () {
    it('X to Y by amount in', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4999n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swap = await invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      const swapResult = swap

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(swapResult.ticks.length)
    })

    it('Y to X by amount in', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4999n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swapResult = await invariant.swap(
        admin,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(swapResult.ticks.length)
    })

    it('Y to X', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4888n
      const byAmountIn = false
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swapResult = await invariant.swap(
        admin,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)

      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(swapResult.ticks.length)
    })

    it('X to Y', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4888n
      const byAmountIn = false
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swapResult = await invariant.swap(
        admin,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(swapResult.ticks.length)
    })
  })

  context('outdated data in', async function () {
    it('pool', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      await invariant.createPosition(
        admin,
        poolKey,
        -10n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('tickmap', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      await invariant.createPosition(
        admin,
        poolKey,
        -20n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('ticks', async function () {
      this.timeout(40000)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
      const amountIn = 20000n
      const byAmountIn = true
      const xToY = true

      const ticks = await invariant.getLiquidityTicks(poolKey)
      
      await invariant.createPosition(
        admin,
        poolKey,
        -20n,
        10n,
        1000000000000n,
        1000000000000000000000000n,
        0n
      )

      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const tickmap = filterTickmap(
        await invariant.getTickmap(poolKey),
        poolKey.feeTier.tickSpacing as any,
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)
    })
  })
  it('max ticks crossed', async function () {
    this.timeout(2000000)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
    const amountIn = 1000000n
    const byAmountIn = true
    const xToY = true

    const mintAmount = 1n << 120n
    await GRC20.mint(admin.addressRaw, mintAmount, token0Address)
    await GRC20.approve(admin, invariant.programId(), mintAmount, token0Address)
    await GRC20.mint(admin.addressRaw, mintAmount, token1Address)
    await GRC20.approve(admin, invariant.programId(), mintAmount, token1Address)

    const liquidityDelta = 10000000n * 10n ** 6n
    const spotSqrtPrice = 1000000000000000000000000n
    const slippageTolerance = 0n

    const indexes: bigint[] = []

    for (let i = -10n; i < 5; i += 1n) {
      indexes.push(i + 1n)
      await invariant.createPosition(
        admin,
        poolKey,
        i,
        i + 1n,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const tickmap = filterTickmap(
      await invariant.getTickmap(poolKey),
      poolKey.feeTier.tickSpacing as any,
      pool.currentTickIndex,
      xToY
    )

    const ticks = filterTicks(
      await invariant.getLiquidityTicks(poolKey),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      sqrtPriceLimit
    )
    expect(simulation.crossedTicks.length).to.equal(integerSafeCast(MAX_TICK_CROSS + 1n))
    expect(simulation.globalInsufficientLiquidity).to.equal(false)
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(true)
  })

  it('max token amount - X to Y - amount in', async function () {
    this.timeout(40000)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = maxTokenAmount
    const byAmountIn = true
    const xToY = true

    const tickmap = filterTickmap(
      await invariant.getTickmap(poolKey),
      poolKey.feeTier.tickSpacing as any,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getLiquidityTicks(poolKey),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MIN_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, MIN_SQRT_PRICE)
    )
  })

  it('max token amount - X to Y - amount out', async function () {
    this.timeout(40000)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = maxTokenAmount
    const byAmountIn = false
    const xToY = true

    const tickmap = filterTickmap(
      await invariant.getTickmap(poolKey),
      poolKey.feeTier.tickSpacing as any,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getLiquidityTicks(poolKey),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MIN_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, MIN_SQRT_PRICE)
    )
  })

  it('max token amount - Y to X - amount in', async function () {
    this.timeout(40000)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = maxTokenAmount
    const byAmountIn = true
    const xToY = false

    const tickmap = filterTickmap(
      await invariant.getTickmap(poolKey),
      poolKey.feeTier.tickSpacing as any,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getLiquidityTicks(poolKey),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MAX_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, MAX_SQRT_PRICE)
    )
  })

  it('max token amount - Y to X - amount out', async function () {
    this.timeout(40000)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = maxTokenAmount
    const byAmountIn = false
    const xToY = false

    const tickmap = filterTickmap(
      await invariant.getTickmap(poolKey),
      poolKey.feeTier.tickSpacing as any,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getLiquidityTicks(poolKey),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MAX_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(admin, poolKey, xToY, amountIn, byAmountIn, MAX_SQRT_PRICE)
    )
  })
  this.afterAll(async function () {
    await unsub.then(unsub => unsub())
  })
})
