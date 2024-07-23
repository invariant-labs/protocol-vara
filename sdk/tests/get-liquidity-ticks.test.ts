import 'mocha'
import {
  positionToTick,
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { CHUNK_SIZE, Network } from '../src/consts.js'
import { Invariant } from '../src/invariant.js'
import { FungibleToken } from '../src/erc20.js'
import { assert } from 'chai'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const initProtocolFee = 10000000000n
const feeTier = newFeeTier(10000000000n, 1n)
let poolKey = null as any

describe('get-liquidity-ticks', async function () {
  this.timeout(40000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  this.beforeEach(async function () {
    this.timeout(100000)
    invariant = await Invariant.deploy(api, admin, initProtocolFee)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    await GRC20.mint(user.addressRaw, 1000000000n, token0Address)
    await GRC20.mint(user.addressRaw, 1000000000n, token1Address)
    await invariant.addFeeTier(admin, feeTier)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)
    await invariant.createPool(user, poolKey, 1000000000000000000000000n)

    await GRC20.approve(user, invariant.programId(), 1000000000n, token0Address)
    await GRC20.approve(user, invariant.programId(), 1000000000n, token1Address)
    await invariant.depositSingleToken(user, token0Address, 1000000n)
    await invariant.depositSingleToken(user, token1Address, 1000000n)
  })

  it('should get liquidity ticks', async () => {
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    for (let i = 1n; i <= 10n; i++) {
      await invariant.createPosition(user, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const tickmap = await invariant.getTickmap(poolKey)
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
    const liquidityTicks = await invariant.getLiquidityTicks(poolKey, tickIndexes)
    const liquidityTicksAmount = await invariant.getLiquidityTicksAmount(poolKey)
    
    assert.deepEqual(liquidityTicksAmount, BigInt(liquidityTicks.length))
    
    assert.deepEqual(
      liquidityTicks.map(tick => tick.index),
      tickIndexes
    )

    const allLiquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)

    assert.deepEqual(
      allLiquidityTicks.map(tick => tick.index),
      tickIndexes
    )
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
