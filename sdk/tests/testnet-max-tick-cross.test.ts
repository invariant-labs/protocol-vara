import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/network.js'
import { Invariant } from '../src/invariant.js'
import { FungibleToken } from '../src/erc20.js'
import {  SqrtPrice } from '../src/schema.js'
import { getMinSqrtPrice } from '../src/utils.js'
import { assert } from 'chai'
import { MAX_SWAP_STEPS } from '../src/consts.js'
const api = await initGearApi(Network.Testnet)
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

  it('max tick cross', async function () {
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
    const liquidityDelta = 1000000000000000n * 10n ** 5n
    const spotSqrtPrice = 1000000000000000000000000n
    const slippageTolerance = 0n

    const indexes: bigint[] = []
    const indexCount = 160n;

    indexes.push(-indexCount * 256n)
    for (let i = -indexCount; i < 0n; i += 1n) {
      indexes.push((i + 1n) * 256n)
      await invariant.createPosition(
        admin,
        poolKey,
        i * 256n,
        (i + 1n) * 256n,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    const swapGasLimit = 120_000_000_000n
    const amountIn = 5128996965924687n;
    const swap = await invariant.swap(admin, poolKey, true, amountIn, true, getMinSqrtPrice(1n), swapGasLimit)
    assert.equal(swap.ticks.length, Number(MAX_SWAP_STEPS))
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
