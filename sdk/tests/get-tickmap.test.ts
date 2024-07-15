import 'mocha'
import {
  getMaxChunk,
  getMaxTick,
  getMinTick,
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts.js'
import { Invariant } from '../src/invariant.js'
import { FungibleToken } from '../src/erc20.js'
import { PoolKey } from '../src/schema.js'
import { assert } from 'chai'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const feeTier = newFeeTier(10000000000n, 1n)
const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
let poolKey: PoolKey = null as any
describe('tickmap test', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  beforeEach(async function () {
    this.timeout(80000)

    invariant = await Invariant.deploy(api, admin, 10000000000n)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(admin, feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await GRC20.mint(admin.addressRaw, 1000000000000000000n, token0Address)
    await GRC20.mint(admin.addressRaw, 1000000000000000000n, token1Address)
    await GRC20.approve(admin, invariant.programId(), 1000000000000000000n, token0Address)
    await GRC20.approve(admin, invariant.programId(), 1000000000000000000n, token1Address)

    await invariant.depositTokenPair(
      admin,
      [token0Address, 1000000000000000000n],
      [token1Address, 1000000000000000000n]
    )
  })

  it('get tickmap', async function () {
    this.timeout(40000)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.createPosition(admin, poolKey, ticks[2], ticks[3], 10n, pool.sqrtPrice, 0n)

    const tickmap = await invariant.getTickmap(poolKey)
    assert.deepEqual(tickmap.bitmap.get(3465n), 9223372036854775809n)

    for (const [chunkIndex, value] of tickmap.bitmap.entries()) {
      if (chunkIndex === 3465n) {
        assert.deepEqual(value, 0b1000000000000000000000000000000000000000000000000000000000000001n)
      } else {
        assert.deepEqual(value, 0n)
      }
    }
  })
  it('get tickmap edge ticks initialized', async function () {
    this.timeout(40000)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.createPosition(admin, poolKey, ticks[0], ticks[1], 10n, pool.sqrtPrice, 0n)
    await invariant.createPosition(admin, poolKey, ticks[4], ticks[5], 10n, pool.sqrtPrice, 0n)

    const tickmap = await invariant.getTickmap(poolKey)

    assert.deepEqual(tickmap.bitmap.get(0n), 0b11n)
    assert.deepEqual(
      tickmap.bitmap.get(getMaxChunk(feeTier.tickSpacing)),
      0b11000000000000000000000000000000000000000000000000000n
    )
  })
  it('get tickmap edge ticks initialized 100 tick spacing', async function () {
    this.timeout(40000)

    const feeTier100 = newFeeTier(10000000000n, 100n)
    poolKey = newPoolKey(token0Address, token1Address, feeTier100)
    await invariant.addFeeTier(admin, feeTier100)
    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier100)
    await invariant.createPosition(
      admin,
      poolKey,
      getMinTick(feeTier100.tickSpacing),
      getMaxTick(feeTier100.tickSpacing),
      100n,
      pool.sqrtPrice,
      0n
    )
    await invariant.createPosition(
      admin,
      poolKey,
      getMinTick(feeTier100.tickSpacing) + BigInt(feeTier100.tickSpacing),
      getMaxTick(feeTier100.tickSpacing) - BigInt(feeTier100.tickSpacing),
      100n,
      pool.sqrtPrice,
      0n
    )

    const tickmap = await invariant.getTickmap(poolKey)

    assert.deepEqual(tickmap.bitmap.get(0n), 0b11n)

    assert.deepEqual(
      tickmap.bitmap.get(getMaxChunk(feeTier100.tickSpacing)),
      0b110000000000000000000n
    )
  })
  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
