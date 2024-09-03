import {
  Pool,
  PoolKey,
  Position
} from '@invariant-labs/vara-sdk-wasm'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/consts'
import { FungibleToken } from '../src/erc20'
import { assertThrowsAsync, objectEquals } from '../src/test-utils'
import { HexString, initGearApi, newFeeTier, newPoolKey } from '../src/utils'
import { describe, it } from 'mocha'
import { GearKeyring } from '@gear-js/api'

const api = await initGearApi(Network.Local)

const account = await GearKeyring.fromSuri('//Alice')

let invariant: Invariant
let token0Address: HexString
let token1Address: HexString 
const GRC20 = await FungibleToken.load(api)
GRC20.setAdmin(account)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey: PoolKey
let pool: Pool

describe('get-position-with-associates', async function() {
  beforeEach(async function () {
    this.timeout(90000)
    invariant = await Invariant.deploy(api, account, 10000000000n)
    token0Address = await FungibleToken.deploy(api, account, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, account, 'Coin', 'COIN', 0n)
    await GRC20.mint(account.addressRaw, 1000000000n, token0Address)
    await GRC20.mint(account.addressRaw, 1000000000n, token1Address)
    
    await GRC20.approve(account, invariant.programId(), 1000000000n, token0Address)
    await GRC20.approve(account, invariant.programId(), 1000000000n, token1Address)
    
    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.depositTokenPair(account, [token0Address, 1000000000n], [token1Address, 1000000000n]);

    const result = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )

    const expectedPosition: Position = {
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      lastBlockNumber: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n,
      liquidity: 1000000000000n,
      lowerTickIndex: -20n,
      upperTickIndex: 10n,
      poolKey,
    }

    objectEquals(result, expectedPosition, ['lastBlockNumber'])
  })

  it('position, pool and ticks match', async function () {
    this.timeout(40000)

    const positionRegular = await invariant.getPosition(account.addressRaw, 0n)
    const poolRegular = await invariant.getPool(token0Address, token1Address, poolKey.feeTier)
    const lowerTickRegular = await invariant.getTick(poolKey, positionRegular.lowerTickIndex)
    const upperTickRegular = await invariant.getTick(poolKey, positionRegular.upperTickIndex)

    const [position, pool, lowerTick, upperTick] = await invariant.getPositionWithAssociates(
      account.addressRaw,
      0n
    )

    assert.deepEqual(position, positionRegular)
    assert.deepEqual(pool, poolRegular)
    assert.deepEqual(lowerTick, lowerTickRegular)
    assert.deepEqual(upperTick, upperTickRegular)
  })

  it('position does not exist', async function() {
    this.timeout(40000)
    
    await assertThrowsAsync(
      invariant.getPositionWithAssociates(account.addressRaw, 1n),
      "Error: PositionNotFound"
    )
  })
})