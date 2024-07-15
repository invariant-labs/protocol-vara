import 'mocha'
import {
  getMaxTick,
  getMinTick,
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { FungibleToken } from '../src/erc20.js'
import { SqrtPrice } from '../src/schema'
import { assert } from 'chai'
const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri("//Alice")
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
    let i = getMinTick(1n)
    while (i <= getMaxTick(1n)) {
      const tickIndexStep = 601n * 64n
      let amount = 600n
      if (tickIndexStep + i > getMaxTick(1n)) {
        amount = 320n
      }
      console.log(i, amount)
      await invariant.addMultiplePositions(admin, poolKey, i as any, amount, 64)
      i += tickIndexStep
    }
    const map = await invariant.getTickmap(poolKey)
    assert.equal(map.bitmap.size, 6932)
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
