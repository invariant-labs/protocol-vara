import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20'
import { assertThrowsAsync, sortTokens } from '../src/test-utils.js'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
let tokenX: FungibleToken = null as any
let tokenY: FungibleToken = null as any
let invariant: Invariant = null as any
const feeTier = newFeeTier(10000000000n, 1n)
const assertBalanceChange = async (changeX: bigint, changeY: bigint, callback: () => any) => {
  const userBalancesBefore = await invariant.getUserBalances(admin.addressRaw)

  const balanceXBefore = await tokenX.balanceOf(admin.addressRaw)
  const balanceYBefore = await tokenY.balanceOf(admin.addressRaw)
  const invariantBalanceXBefore = await tokenX.balanceOf(invariant.programId())
  const invariantBalanceYBefore = await tokenY.balanceOf(invariant.programId())

  await callback()
  const userBalancesAfter = await invariant.getUserBalances(admin.addressRaw)

  assert.deepEqual(
    userBalancesAfter.get(tokenX.programId()) || 0n,
    (userBalancesBefore.get(tokenX.programId()) || 0n) - changeX,
    'tokenX, user balance mismatch'
  )
  assert.deepEqual(
    userBalancesAfter.get(tokenY.programId()) || 0n,
    (userBalancesBefore.get(tokenY.programId()) || 0n) - changeY,
    'tokenY, user balance mismatch'
  )

  const balanceXAfter = await tokenX.balanceOf(admin.addressRaw)
  const balanceYAfter = await tokenY.balanceOf(admin.addressRaw)
  const invariantBalanceXAfter = await tokenX.balanceOf(invariant.programId())
  const invariantBalanceYAfter = await tokenY.balanceOf(invariant.programId())

  assert.deepEqual(balanceXAfter, balanceXBefore + changeX, 'tokenX, balance mismatch')
  assert.deepEqual(balanceYAfter, balanceYBefore + changeY, 'tokenY, balance mismatch')
  assert.deepEqual(
    invariantBalanceXAfter,
    invariantBalanceXBefore - changeX,
    'tokenX, invariant balance mismatch'
  )
  assert.deepEqual(
    invariantBalanceYAfter,
    invariantBalanceYBefore - changeY,
    'tokenY, invariant balance mismatch'
  )
}
describe('deposits', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  beforeEach(async function () {
    this.timeout(200000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    tokenX = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    tokenY = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    ;[tokenX, tokenY] = sortTokens(tokenX, tokenY)
    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(tokenX.programId(), tokenY.programId(), feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await tokenX.mint(admin.addressRaw, 100000000000000n)
    await tokenY.mint(admin.addressRaw, 100000000000000n)

    await tokenX.approve(admin, invariant.programId(), 100000000000000n)
    await tokenY.approve(admin, invariant.programId(), 100000000000000n)
  })

  it('single deposit and withdraw', async function () {
    this.timeout(200000)
    const amount = 10000000000000n
    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(
        await invariant.depositSingleToken(admin, tokenX.programId(), amount),
        amount
      )
      assert.deepEqual(
        await invariant.depositSingleToken(admin, tokenY.programId(), amount),
        amount
      )
    })
    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(
        await invariant.withdrawSingleToken(admin, tokenX.programId(), amount),
        amount
      )
      assert.deepEqual(
        await invariant.withdrawSingleToken(admin, tokenY.programId(), amount),
        amount
      )
    })

    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(
        await invariant.depositSingleToken(admin, tokenX.programId(), amount),
        amount
      )
      assert.deepEqual(
        await invariant.depositSingleToken(admin, tokenY.programId(), amount),
        amount
      )
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(await invariant.withdrawSingleToken(admin, tokenX.programId(), null), amount)
      assert.deepEqual(await invariant.withdrawSingleToken(admin, tokenY.programId(), null), amount)
    })
  })

  it('single deposit and withdraw errors', async function () {
    this.timeout(200000)
    const amount = 10000000000000n

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawSingleToken(admin, tokenX.programId(), amount),
        "Panic occurred: panicked with 'InvariantError: NoBalanceForTheToken'"
      )
    })

    await tokenX.setTransferFail(true)
    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositSingleToken(admin, tokenX.programId(), amount),
        "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"
      )
    })

    await tokenX.setTransferFail(false)
    await assertBalanceChange(-amount, 0n, async () => {
      await invariant.depositSingleToken(admin, tokenX.programId(), amount)
    })
    await tokenX.setTransferFail(true)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawSingleToken(admin, tokenX.programId(), amount),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  it('deposit and withdraw token pair', async function () {
    this.timeout(80000)
    const amount = 10000000000000n
    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(
        await invariant.depositTokenPair(
          admin,
          [tokenX.programId(), amount],
          [tokenY.programId(), amount]
        ),
        [amount, amount]
      )
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(
          admin,
          [tokenX.programId(), amount],
          [tokenY.programId(), amount]
        ),
        [amount, amount]
      )
    })

    await assertBalanceChange(-amount, -amount, async () => {
      await invariant.depositTokenPair(
        admin,
        [tokenX.programId(), amount],
        [tokenY.programId(), amount]
      )
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(
          admin,
          [tokenX.programId(), null],
          [tokenY.programId(), null]
        ),
        [amount, amount]
      )
    })

    await assertBalanceChange(-1n, -2n, async () => {
      assert.deepEqual(
        await invariant.depositTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 2n]),
        [1n, 2n]
      )
    })

    await assertBalanceChange(1n, 2n, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(
          admin,
          [tokenX.programId(), 1n],
          [tokenY.programId(), 2n]
        ),
        [1n, 2n]
      )
    })
  })

  it('withdraw token pair errors', async function () {
    this.timeout(200000)

    await assertBalanceChange(-100n, -100n, async () => {
      assert.deepEqual(
        await invariant.depositTokenPair(
          admin,
          [tokenX.programId(), 100n],
          [tokenY.programId(), 100n]
        ),
        [100n, 100n]
      )
    })

    await tokenY.setTransferFail(true)

    await assertBalanceChange(1n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await tokenX.setTransferFail(true)
    await tokenY.setTransferFail(false)

    await assertBalanceChange(0n, 1n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await tokenX.setTransferFail(true)
    await tokenY.setTransferFail(true)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  it('deposit token pair errors', async function () {
    this.timeout(200000)
    await tokenX.setTransferFail(true)
    await tokenY.setTransferFail(true)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"
      )
    })

    await tokenY.setTransferFail(false)

    await assertBalanceChange(0n, -1n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await tokenX.setTransferFail(false)
    await tokenY.setTransferFail(true)

    await assertBalanceChange(-1n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [tokenX.programId(), 1n], [tokenY.programId(), 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
