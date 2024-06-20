import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring } from '@gear-js/api'
import { LOCAL } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20'

const api = await initGearApi({ providerAddress: LOCAL })
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
let tokenX: FungibleToken = null as any
let tokenY: FungibleToken = null as any
let invariant: Invariant = null as any
const feeTier = newFeeTier(10000000000n, 1n)
const assertBalanceChange = async (changeX: bigint, changeY: bigint, callback: () => any) => {
  const balanceXBefore = await tokenX.balanceOf(admin.addressRaw)
  const balanceYBefore = await tokenY.balanceOf(admin.addressRaw)
  await callback()
  const balanceXAfter = await tokenX.balanceOf(admin.addressRaw)
  const balanceYAfter = await tokenY.balanceOf(admin.addressRaw)
  assert.equal(balanceXAfter - changeX, balanceXBefore)
  assert.equal(balanceYAfter - changeY, balanceYBefore)
}
describe('deposits', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  beforeEach(async function () {
    this.timeout(80000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    tokenX = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    tokenY = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(tokenX.programId(), tokenY.programId(), feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await tokenX.mint(admin.addressRaw, 100000000000000n)
    await tokenY.mint(admin.addressRaw, 100000000000000n)

    await tokenX.approve(admin, invariant.programId(), 100000000000000n)
    await tokenY.approve(admin, invariant.programId(), 100000000000000n)
  })

  it('single deposit and withdraw', async function () {
    this.timeout(80000)
    const amount = 10000000000000n
    await assertBalanceChange(-amount, -amount, async () => {
      await invariant.depositSingleToken(admin, tokenX.programId(), amount)
      await invariant.depositSingleToken(admin, tokenY.programId(), amount)
    })

    await assertBalanceChange(amount, amount, async () => {
      await invariant.withdrawSingleToken(admin, tokenX.programId(), amount)
      await invariant.withdrawSingleToken(admin, tokenY.programId(), amount)
    })

    await assertBalanceChange(-amount, -amount, async () => {
      await invariant.depositSingleToken(admin, tokenX.programId(), amount)
      await invariant.depositSingleToken(admin, tokenY.programId(), amount)
    })

    await assertBalanceChange(amount, amount, async () => {
      await invariant.withdrawSingleToken(admin, tokenX.programId(), null)
      await invariant.withdrawSingleToken(admin, tokenY.programId(), null)
    })
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
