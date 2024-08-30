import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync } from '../src/test-utils.js'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0: HexString = null as any
let token1: HexString = null as any
let invariant: Invariant = null as any
const feeTier = newFeeTier(10000000000n, 1n)

describe('protocol-fee', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  beforeEach(async function () {
    this.timeout(80000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    token0 = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1 = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(token0, token1, feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await GRC20.mint(admin.addressRaw, 10000000000000n, token0)
    await GRC20.mint(admin.addressRaw, 10000000000000n, token1)

    await GRC20.approve(admin, invariant.programId(), 10000000000000n, token0)
    await GRC20.approve(admin, invariant.programId(), 10000000000000n, token1)

    await invariant.depositSingleToken(admin, token0, 10000000000000n)
    await invariant.depositSingleToken(admin, token1, 10000000000000n)

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      10000000000000n,
      1000000000000000000000000n,
      10000000000n
    )

    await GRC20.approve(admin, invariant.programId(), 1000000000n, token0)
    await GRC20.approve(admin, invariant.programId(), 1000000000n, token1)

    await invariant.swap(admin, poolKey, true, 4999n, true, 999505344804856076727628n)
  })

  it('should withdraw protocol fee', async function () {
    this.timeout(80000)
    let withdrawnToken
    const poolKey = newPoolKey(token0, token1, feeTier)

    if (poolKey.tokenX === token0) {
      withdrawnToken = token0
    } else {
      withdrawnToken = token1
    }

    await invariant.withdrawSingleToken(admin, withdrawnToken)
    const token0Before = await GRC20.balanceOf(admin.addressRaw, token0)
    const token1Before = await GRC20.balanceOf(admin.addressRaw, token1)

    const poolBefore = await invariant.getPool(token0, token1, feeTier)
    assert.deepEqual(poolBefore.feeProtocolTokenX, 1n, "tokenX fee mismatch")
    assert.deepEqual(poolBefore.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawProtocolFee(admin, poolKey)

    const poolAfter = await invariant.getPool(token0, token1, feeTier)
    assert.deepEqual(poolAfter.feeProtocolTokenX, 0n, "tokenX fee mismatch")
    assert.deepEqual(poolAfter.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawSingleToken(admin, withdrawnToken)
    const token0After = await GRC20.balanceOf(admin.addressRaw, token0)
    const token1After = await GRC20.balanceOf(admin.addressRaw, token1)    

    if (poolKey.tokenX === token0) {
      assert.deepEqual(token0Before + 1n, token0After)
      assert.deepEqual(token1Before, token1After)
    } else {
      assert.deepEqual(token0Before, token0After)
      assert.deepEqual(token1Before + 1n, token1After)
    }
  })

  it('should change fee receiver', async function () {
    this.timeout(80000)
    const poolKey = newPoolKey(token0, token1, feeTier)
    
    let withdrawnToken
    if (poolKey.tokenX === token0) {
      withdrawnToken = token0
    } else {
      withdrawnToken = token1
    }

    await invariant.changeFeeReceiver(admin, poolKey, user.addressRaw)

    const token0Before = await GRC20.balanceOf(user.addressRaw, token0)
    const token1Before = await GRC20.balanceOf(user.addressRaw, token1)

    const poolBefore = await invariant.getPool(token0, token1, feeTier)
    assert.strictEqual(poolBefore.feeProtocolTokenX, 1n, "tokenX fee mismatch")
    assert.strictEqual(poolBefore.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawProtocolFee(user, poolKey)
    await assertThrowsAsync(
      invariant.withdrawProtocolFee(admin, poolKey),
      "Panic occurred: panicked with 'InvariantError: NotFeeReceiver'"
    )

    const poolAfter = await invariant.getPool(token0, token1, feeTier)
    assert.strictEqual(poolAfter.feeProtocolTokenX, 0n, "tokenX fee mismatch")
    assert.strictEqual(poolAfter.feeProtocolTokenY, 0n, "tokenY fee mismatch")


    await invariant.withdrawSingleToken(user, withdrawnToken)
    const token0After = await GRC20.balanceOf(user.addressRaw, token0)
    const token1After = await GRC20.balanceOf(user.addressRaw, token1)

    if (poolKey.tokenX === token0) {
      assert.deepEqual(token0Before + 1n, token0After)
      assert.deepEqual(token1Before, token1After)
    } else {
      assert.deepEqual(token0Before, token0After)
      assert.deepEqual(token1Before + 1n, token1After)
    }
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
