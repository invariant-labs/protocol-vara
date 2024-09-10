import 'mocha'
import { PoolKey } from '../src/schema'
import { GearKeyring } from '@gear-js/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { FungibleToken } from '../src/erc20'
import { HexString, initGearApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initGearApi(Network.Local)

const admin = await GearKeyring.fromSuri('//Alice')

let invariant: Invariant
let token0Address: HexString
let token1Address: HexString
let poolKey0: PoolKey
let poolKey1: PoolKey

const feeTier10ts = newFeeTier(6000000000n, 10n)
const feeTier20ts = newFeeTier(6000000000n, 20n)

describe('query-on-pair', async function () {
  this.timeout(50000)
  before(async function () {
    this.timeout(50000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    token0Address = await FungibleToken.deploy(api, admin)
    token1Address = await FungibleToken.deploy(api, admin)

    poolKey0 = newPoolKey(token0Address, token1Address, feeTier10ts)
    poolKey1 = newPoolKey(token0Address, token1Address, feeTier20ts)

    await invariant.addFeeTier(admin, feeTier10ts)
    await invariant.addFeeTier(admin, feeTier20ts)

    await invariant.createPool(admin, poolKey0, 1000000000000000000000000n)
    await invariant.createPool(admin, poolKey1, 2000000000000000000000000n)
  })
  it('query all pools for pair', async function () {
    this.timeout(50000)
    const pools = await invariant.getAllPoolsForPair(token0Address, token1Address)
    const expectedPool0 = await invariant.getPool(
      poolKey0.tokenX,
      poolKey0.tokenY,
      poolKey0.feeTier
    )
    const expectedPool1 = await invariant.getPool(
      poolKey1.tokenX,
      poolKey1.tokenY,
      poolKey1.feeTier
    )

    assert.deepEqual(pools, [
      [poolKey0.feeTier, expectedPool0],
      [poolKey1.feeTier, expectedPool1]
    ])
  })
})
