import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { FungibleToken } from '../src/erc20.js'
import { SqrtPrice } from '../src/schema'
import { getMaxSqrtPrice, getMinSqrtPrice, getMinTick } from 'invariant-vara-wasm'
const api = await initGearApi({ providerAddress: Network.Testnet })
// const admin = await GearKeyring.fromSuri("//Alice")
const admin = await GearKeyring.fromMnemonic(process.env.VARA_TESTNET_MNEMONIC as string)
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
    let i = -(863n * 256n) as any
    await invariant.addMultiplePositions(admin, poolKey, i, 800n, 256)
    i += 800n * 256n
    console.log(i)
    await invariant.addMultiplePositions(admin, poolKey, i, 318n, 256)
    {
      let low = 0n
      let high = 2n ** 128n
      let mid = 0n
      while (low + 1n < high) {
        mid = (low + high) / 2n
        try {
          let res = await invariant.quote(poolKey, false, mid, true)
          console.log('success', mid, res.ticks.length)
          low = mid
        } catch (e: any) {
          console.log(e)
          // response size too large error
          if (e.message.match(/131072/)) {
            low = mid
          } else {
            high = mid
          }
        }
        console.log(low, mid, high)
      }
      try {
        const result = await invariant.swap(admin, poolKey, false, low, true, getMaxSqrtPrice(1n))
        console.log('completed', low)
        console.log(result.ticks.length)
      } catch (e) {
        console.log('failed', low, e)
      }
    }
    {
      let low = 63309419510914185176n
      try {
        const result = await invariant.swap(admin, poolKey, true, low, true, getMinSqrtPrice(1n))
        console.log('completed', low)
        console.log(result.ticks.length)
      } catch (e) {
        console.log('failed', low, e)
      }
    }
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
