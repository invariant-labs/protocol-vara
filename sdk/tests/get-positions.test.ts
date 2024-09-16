import { Pool, PoolKey } from '../src/schema'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { FungibleToken } from '../src/erc20'
import { initGearApi, newFeeTier, newPoolKey } from '../src/utils'
import { assert } from 'chai'
import { objectEquals } from '../src/test-utils'
import { describe, it } from 'mocha'
import { decodeAddress, GearKeyring, HexString } from '@gear-js/api'

const api = await initGearApi(Network.Local)

const admin = await GearKeyring.fromSuri('//Alice')

let invariant: Invariant
let token0Address: HexString
let token1Address: HexString
const grc20 = await FungibleToken.load(api)
grc20.setAdmin(admin)

const feeTier = newFeeTier(6000000000n, 10n)

let poolKey: PoolKey
let pool: Pool

describe('get-positions', async function () {
  beforeEach(async function () {
    this.timeout(60000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(admin, feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await grc20.mint(admin.addressRaw, 10000000000n, token0Address)
    await grc20.mint(admin.addressRaw, 10000000000n, token1Address)

    await grc20.approve(admin, invariant.programId(), 10000000000n, token0Address)
    await grc20.approve(admin, invariant.programId(), 10000000000n, token1Address)

    pool = await invariant.getPool(token0Address, token1Address, feeTier)

    await invariant.depositTokenPair(
      admin,
      [token0Address, 10000000000n],
      [token1Address, 10000000000n]
    )
    await invariant.createPosition(admin, poolKey, -10n, 10n, 1000000000000n, pool.sqrtPrice, 0n)
    await invariant.createPosition(admin, poolKey, -20n, 20n, 1000000000000n, pool.sqrtPrice, 0n)
  })

  xit('get positions', async function () {
    this.timeout(40000)

    const result = await invariant.getPositions(admin.addressRaw, 2n, 0n)

    assert.equal(result[0].length, 2)
    assert.equal(result[1], 2n)

    const firstExpectedPosition = {
      poolKey,
      liquidity: 1000000000000n,
      lowerTickIndex: -10n,
      upperTickIndex: 10n,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    const firstExpectedPool = {
      liquidity: 2000000000000n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      feeReceiver: decodeAddress(admin.address)
    }

    objectEquals(result[0][0][0], firstExpectedPosition, ['lastBlockNumber'])
    objectEquals(result[0][0][1], firstExpectedPool, ['startTimestamp', 'lastTimestamp'])

    const secondExpectedPosition = {
      poolKey,
      liquidity: 1000000000000n,
      lowerTickIndex: -20n,
      upperTickIndex: 20n,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    const secondExpectedPool = {
      liquidity: 2000000000000n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      feeReceiver: decodeAddress(admin.address)
    }

    objectEquals(result[0][1][0], secondExpectedPosition, ['lastBlockNumber'])
    objectEquals(result[0][1][1], secondExpectedPool, ['startTimestamp', 'lastTimestamp'])
  })

  xit('get positions less than exist', async function () {
    this.timeout(10000)
    const result = await invariant.getPositions(admin.addressRaw, 1n, 0n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  xit('get positions more than exist', async function () {
    this.timeout(10000)
    const result = await invariant.getPositions(admin.addressRaw, 3n, 0n)

    assert.equal(result[0].length, 2)
    assert.equal(result[1], 2n)
  })

  xit('get positions with offset', async function () {
    this.timeout(10000)
    const result = await invariant.getPositions(admin.addressRaw, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  xit('get positions with offset less than exist', async function () {
    this.timeout(10000)
    await invariant.createPosition(admin, poolKey, -30n, 30n, 1000000000000n, pool.sqrtPrice, 0n)
    const result = await invariant.getPositions(admin.addressRaw, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 3n)
  })

  xit('get positions with offset more than exist', async function () {
    const result = await invariant.getPositions(admin.addressRaw, 2n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  it('get positions with 2 pools in the right order', async function () {
    this.timeout(40000)
    const secondPoolKey = newPoolKey(
      poolKey.tokenX,
      poolKey.tokenY,
      newFeeTier(poolKey.feeTier.fee + 1n, poolKey.feeTier.tickSpacing)
    )

    await invariant.addFeeTier(admin, secondPoolKey.feeTier)
    await invariant.createPool(admin, secondPoolKey, 1000000000000000000000000n)

    await invariant.createPosition(
      admin,
      secondPoolKey,
      -10n,
      10n,
      10n,
      1000000000000000000000000n,
      0n
    )
    await invariant.createPosition(admin, poolKey, -10n, 10n, 10n, 1000000000000000000000000n, 0n)
    const result = await invariant.getPositions(admin.addressRaw, 4n, 0n)

    for (let i = 0; i < 4; i++) {
      const expectedPosition = await invariant.getPosition(admin.addressRaw, BigInt(i))
      const actualPosition = result[0][i][0]

      assert.deepEqual(
        actualPosition,
        expectedPosition,
      )
    }

    assert.equal(result[0].length, 4)
    assert.equal(result[1], 4n)
  })
})
