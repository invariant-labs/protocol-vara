import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring } from '@gear-js/api'
import { LOCAL } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20'
import { assertThrowsAsync } from '../src/test-utils.js'

const api = await initGearApi({ providerAddress: LOCAL })
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
let token0: FungibleToken = null as any
let token1: FungibleToken = null as any
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

    const poolKey = newPoolKey(token0.programId(), token1.programId(), feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await token0.mint(admin.addressRaw, 10000000000000n)
    await token1.mint(admin.addressRaw, 10000000000000n)

    await token0.approve(admin, invariant.programId(), 10000000000000n)
    await token1.approve(admin, invariant.programId(), 10000000000000n)

    await invariant.depositSingleToken(admin, token0.programId(), 10000000000000n)
    await invariant.depositSingleToken(admin, token1.programId(), 10000000000000n)

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      10000000000000n,
      1000000000000000000000000n,
      1000000000000000000000000n
    )

    await token0.approve(admin, invariant.programId(), 1000000000n)
    await token1.approve(admin, invariant.programId(), 1000000000n)

    await invariant.swap(admin, poolKey, true, 4999n, true, 999505344804856076727628n)
  })

  it('should withdraw protocol fee', async function () {
    this.timeout(80000)
    let withdrawnToken
    const poolKey = newPoolKey(token0.programId(), token1.programId(), feeTier)

    if (poolKey.tokenX === token0.programId()) {
      withdrawnToken = token0
    } else {
      withdrawnToken = token1
    }

    await invariant.withdrawSingleToken(admin, withdrawnToken.programId())
    const token0Before = await token0.balanceOf(admin.addressRaw)
    const token1Before = await token1.balanceOf(admin.addressRaw)

    const poolBefore = await invariant.getPool(token0.programId(), token1.programId(), feeTier)
    assert.deepEqual(poolBefore.feeProtocolTokenX, 1n, "tokenX fee mismatch")
    assert.deepEqual(poolBefore.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawProtocolFee(admin, poolKey)

    const poolAfter = await invariant.getPool(token0.programId(), token1.programId(), feeTier)
    assert.deepEqual(poolAfter.feeProtocolTokenX, 0n, "tokenX fee mismatch")
    assert.deepEqual(poolAfter.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawSingleToken(admin, withdrawnToken.programId())
    const token0After = await token0.balanceOf(admin.addressRaw)
    const token1After = await token1.balanceOf(admin.addressRaw)    

    if (poolKey.tokenX === token0.programId()) {
      assert.deepEqual(token0Before + 1n, token0After)
      assert.deepEqual(token1Before, token1After)
    } else {
      assert.deepEqual(token0Before, token0After)
      assert.deepEqual(token1Before + 1n, token1After)
    }
  })

  it('should change fee receiver', async function () {
    this.timeout(80000)
    const poolKey = newPoolKey(token0.programId(), token1.programId(), feeTier)
    
    let withdrawnToken
    if (poolKey.tokenX === token0.programId()) {
      withdrawnToken = token0
    } else {
      withdrawnToken = token1
    }

    await invariant.changeFeeReceiver(admin, poolKey, user.addressRaw)

    const token0Before = await token0.balanceOf(user.addressRaw)
    const token1Before = await token1.balanceOf(user.addressRaw)

    const poolBefore = await invariant.getPool(token0.programId(), token1.programId(), feeTier)
    assert.strictEqual(poolBefore.feeProtocolTokenX, 1n, "tokenX fee mismatch")
    assert.strictEqual(poolBefore.feeProtocolTokenY, 0n, "tokenY fee mismatch")

    await invariant.withdrawProtocolFee(user, poolKey)
    await assertThrowsAsync(
      invariant.withdrawProtocolFee(admin, poolKey),
      "Panic occurred: panicked with 'InvariantError: NotFeeReceiver'"
    )

    const poolAfter = await invariant.getPool(token0.programId(), token1.programId(), feeTier)
    assert.strictEqual(poolAfter.feeProtocolTokenX, 0n, "tokenX fee mismatch")
    assert.strictEqual(poolAfter.feeProtocolTokenY, 0n, "tokenY fee mismatch")


    await invariant.withdrawSingleToken(user, withdrawnToken.programId())
    const token0After = await token0.balanceOf(user.addressRaw)
    const token1After = await token1.balanceOf(user.addressRaw)

    if (poolKey.tokenX === token0.programId()) {
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
