import 'mocha'
import {
  getMaxTick,
  getMinTick,
  initGearApi,
  newFeeTier,
  newPoolKey,
  positionToTick,
  subscribeToNewHeads
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { CHUNK_SIZE, LIQUIDITY_TICKS_LIMIT, Network } from '../src/consts.js'
import { Invariant } from '../src/invariant.js'
import { FungibleToken } from '../src/erc20.js'
import { SqrtPrice } from '../src/schema.js'
import { assert } from 'chai'
const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')
// const admin = await GearKeyring.fromMnemonic(process.env.VARA_TESTNET_MNEMONIC as string)
let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const initProtocolFee = 10000000000n
const feeTier = newFeeTier(10000000000n, 1n)

describe('Invariant', async function () {
  this.timeout(40000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  this.beforeEach(async function () {
    this.timeout(40000)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
  })

  it('all chunks active', async function () {
    this.timeout(8000000)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    invariant = await Invariant.deploy(api, admin, initProtocolFee)
    await invariant.addFeeTier(admin, feeTier)
    await invariant.createPool(admin, poolKey, initSqrtPrice)
    await GRC20.mint(admin.addressRaw, 2n ** 256n - 1n, token0Address)
    await GRC20.mint(admin.addressRaw, 2n ** 256n - 1n, token1Address)
    await GRC20.approve(admin, invariant.programId(), 2n ** 256n - 1n, token0Address)
    await GRC20.approve(admin, invariant.programId(), 2n ** 256n - 1n, token1Address)
    await invariant.depositTokenPair(
      admin,
      [token0Address, 2n ** 256n - 1n],
      [token1Address, 2n ** 256n - 1n]
    )
    let totalAm = 0
    const maxLiquidityTicksReturn = LIQUIDITY_TICKS_LIMIT
    const step = (getMaxTick(1n) * 2n) / maxLiquidityTicksReturn
    let i = getMinTick(1n)
    while (totalAm < 20800) {
      let amount = 400n
      let tickStep = 400n * step * 2n

      if (i + tickStep > getMaxTick(1n)) {
        tickStep = getMaxTick(1n)
        amount = maxLiquidityTicksReturn - BigInt(totalAm)
      }
      await invariant.addMultiplePositions(admin, poolKey, i as any, amount, step as any, true)
      const tickmap = await invariant.getTickmap(poolKey)
      const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
      totalAm = ticks.length
      i += tickStep
    }

    assert.equal(BigInt(totalAm), 20800n)
    await invariant.addMultiplePositions(admin, poolKey, i as any, 372n, step as any, true)

    const tickmap = await invariant.getTickmap(poolKey)
    const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
    assert.equal(BigInt(ticks.length), maxLiquidityTicksReturn)

    {
      const tickIndexes: bigint[] = []
      for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
        for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
          const checkedBit = chunk & (1n << bit)
          if (checkedBit) {
            const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
            tickIndexes.push(tickIndex)
          }
        }
      }
      assert.deepEqual(
        ticks.map(tick => tick.index),
        tickIndexes
      )
    }

    await invariant.addMultiplePositions(admin, poolKey, 0n as any, 100n, 1 as any, true)
    const ticksAboveLimit = await invariant.getAllLiquidityTicks(poolKey, await invariant.getTickmap(poolKey))
    console.log(ticksAboveLimit.length)
    assert.equal(BigInt(ticksAboveLimit.length), maxLiquidityTicksReturn + 190n)

    {
      const tickIndexes: bigint[] = []
      for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
        for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
          const checkedBit = chunk & (1n << bit)
          if (checkedBit) {
            const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
            tickIndexes.push(tickIndex)
          }
        }
      }
      assert.deepEqual(
        ticks.map(tick => tick.index),
        tickIndexes
      )
    }
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
