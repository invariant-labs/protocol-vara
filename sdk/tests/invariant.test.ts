import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/network'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync } from '../src/test-utils.js'
import { SqrtPrice, Tick } from '../src/schema'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const MAXu64 = 18_446_744_073_709_551_615n
const initProtocolFee = 10000000000n
const maxProtocolFee = MAXu64
const feeTier = newFeeTier(10000000000n, 1n)

describe('Invariant', async function () {
  this.timeout(40000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  this.beforeEach(async function () {
    this.timeout(40000)
    invariant = await Invariant.deploy(api, admin, initProtocolFee)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    await GRC20.mint(user.addressRaw, 1000000000n, token0Address)
    await GRC20.mint(user.addressRaw, 1000000000n, token1Address)
  })

  it('should set and change protocol fee', async function () {
    this.timeout(30000)
    assert.strictEqual(await invariant.getProtocolFee(), initProtocolFee, 'Fee not set')

    assert.strictEqual(await invariant.changeProtocolFee(admin, 1n), 1n)
    assert.strictEqual(await invariant.getProtocolFee(), 1n, 'Fee not set')

    assert.strictEqual(await invariant.changeProtocolFee(admin, maxProtocolFee), maxProtocolFee)
    assert.strictEqual(await invariant.getProtocolFee(), maxProtocolFee, 'Fee overflow')

    // deploy with max fee
    const second = await Invariant.deploy(api, admin, maxProtocolFee)
    assert.strictEqual(await second.getProtocolFee(), maxProtocolFee, 'Fee overflow')
  })

  it('should add fee tier', async () => {
    this.timeout(40000)

    const feeTier = newFeeTier(10000000000n, 5n)
    const anotherFeeTier = newFeeTier(20000000000n, 10n)

    await invariant.addFeeTier(admin, feeTier)
    let addedFeeTierExists = await invariant.feeTierExists(feeTier)
    const notAddedFeeTierExists = await invariant.feeTierExists(anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(notAddedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.addFeeTier(admin, anotherFeeTier)
    const addedBeforeFeeTierExists = await invariant.feeTierExists(feeTier)
    addedFeeTierExists = await invariant.feeTierExists(anotherFeeTier)
    feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(addedBeforeFeeTierExists, true)
    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(feeTiers.length, 2)
  })

  it('should remove fee tier', async () => {
    this.timeout(40000)

    const feeTier = newFeeTier(10000000000n, 5n)
    const anotherFeeTier = newFeeTier(20000000000n, 10n)

    await invariant.addFeeTier(admin, feeTier)
    await invariant.addFeeTier(admin, anotherFeeTier)

    await invariant.removeFeeTier(admin, anotherFeeTier)
    const notRemovedFeeTierExists = await invariant.feeTierExists(feeTier)
    let removedFeeTierExists = await invariant.feeTierExists(anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(notRemovedFeeTierExists, true)
    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.removeFeeTier(admin, feeTier)
    removedFeeTierExists = await invariant.feeTierExists(feeTier)
    const removedBeforeFeeTierExists = await invariant.feeTierExists(anotherFeeTier)
    feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(removedBeforeFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 0)
  })

  it('should get tick and check if it is initialized', async () => {
    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(user, poolKey, 1000000000000000000000000n)

    await GRC20.approve(user, invariant.programId(), 1000000000n, token0Address)
    await GRC20.approve(user, invariant.programId(), 1000000000n, token1Address)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    await invariant.depositSingleToken(user, token0Address, 1000000n)
    await invariant.depositSingleToken(user, token1Address, 1000000n)

    await invariant.createPosition(
      user,
      poolKey,
      -10n,
      10n,
      1000000n,
      pool.sqrtPrice,
      0n
    )

    const lowerTick = await invariant.getTick(poolKey, -10n)

    assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 999500149965000000000000n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: lowerTick.secondsOutside
    } as Tick)
    await assertThrowsAsync(invariant.getTick(poolKey, 0n), 'Error: TickNotFound')
    const upperTick = await invariant.getTick(poolKey, 10n)
    assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 1000500100010000000000000n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: upperTick.secondsOutside
    })

    const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, -10n)
    assert.deepEqual(isLowerTickInitialized, true)
    const isInitTickInitialized = await invariant.isTickInitialized(poolKey, 0n)
    assert.deepEqual(isInitTickInitialized, false)
    const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, 10n)
    assert.deepEqual(isUpperTickInitialized, true)
  })

  it('create pool', async function () {
    this.timeout(60000)
    await invariant.addFeeTier(admin, feeTier)
    const addedFeeTierExists = await invariant.feeTierExists(feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(user, poolKey, initSqrtPrice)
    const pools = await invariant.getPoolKeys(1n, 0n)
    assert.deepEqual(pools[0].length, 1)
    assert.deepEqual(pools[1], 1n)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    assert.deepEqual(pool, {
      liquidity: 0n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      startTimestamp: pool.startTimestamp,
      lastTimestamp: pool.lastTimestamp,
      feeReceiver: pool.feeReceiver
    })
  })
  it('attempt to try create pool with wrong tick sqrtPrice relationship', async () => {
    await invariant.addFeeTier(admin, feeTier)
    const addedFeeTierExists = await invariant.feeTierExists(feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000175003749000000000000n

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    assertThrowsAsync(invariant.createPool(user, poolKey, initSqrtPrice), 'Error: InvalidInitTick')
  })

  it('create pool x/y and y/x', async () => {
    await invariant.addFeeTier(admin, feeTier)
    const addedFeeTierExists = await invariant.feeTierExists(feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.createPool(user, poolKey, initSqrtPrice)

      const pools = await invariant.getPoolKeys(1n, 0n)
      assert.deepEqual(pools[0].length, 1)
      assert.deepEqual(pools[1], 1n)

      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      assert.deepEqual(pool, {
        liquidity: 0n,
        sqrtPrice: 1000000000000000000000000n,
        currentTickIndex: 0n,
        feeGrowthGlobalX: 0n,
        feeGrowthGlobalY: 0n,
        feeProtocolTokenX: 0n,
        feeProtocolTokenY: 0n,
        startTimestamp: pool.startTimestamp,
        lastTimestamp: pool.lastTimestamp,
        feeReceiver: pool.feeReceiver
      })
    }
    {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await assertThrowsAsync(invariant.createPool(user, poolKey, initSqrtPrice))
    }
    const pools = await invariant.getPoolKeys(1n, 0n)
    assert.deepEqual(pools[0].length, 1)
    assert.deepEqual(pools[1], 1n)
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
