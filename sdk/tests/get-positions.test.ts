import { Pool, PoolKey } from '../src/schema'
import { Invariant } from '../src/invariant'
import { Network } from '../src/consts'
import { FungibleToken } from '../src/erc20'
import { initGearApi, newFeeTier, newPoolKey } from '../src/utils'
import { assert } from 'chai'
import { objectEquals } from '../src/test-utils'
import { describe, it } from 'mocha'
import { GearKeyring, HexString } from '@gear-js/api'

const api = await initGearApi({ providerAddress: Network.Local })

const admin = await GearKeyring.fromSuri('//Alice')

let invariant: Invariant
let token0Address: HexString
let token1Address: HexString
const grc20 = await FungibleToken.load(api)
grc20.setAdmin(admin)

const feeTier = newFeeTier(6000000000n, 10n)

let poolKey: PoolKey
let pool: Pool

describe('get-positions', async () => {
  beforeEach(async () => {
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

  it('get positions', async () => {
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
      feeReceiver: admin.addressRaw
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
      feeReceiver: admin.addressRaw
    }

    objectEquals(result[0][1][0], secondExpectedPosition, ['lastBlockNumber'])
    objectEquals(result[0][1][1], secondExpectedPool, ['startTimestamp', 'lastTimestamp'])
  })

  it('get positions less than exist', async () => {
    const result = await invariant.getPositions(admin.addressRaw, 1n, 0n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  it('get positions more than exist', async () => {
    const result = await invariant.getPositions(admin.addressRaw, 3n, 0n)

    assert.equal(result[0].length, 2)
    assert.equal(result[1], 2n)
  })

  it('get positions with offset', async () => {
    const result = await invariant.getPositions(admin.addressRaw, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  it('get positions with offset less than exist', async () => {
    await invariant.createPosition(admin, poolKey, -30n, 30n, 1000000000000n, pool.sqrtPrice, 0n)
    const result = await invariant.getPositions(admin.addressRaw, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 3n)
  })

  it('get positions with offset more than exist', async () => {
    const result = await invariant.getPositions(admin.addressRaw, 2n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })
})
