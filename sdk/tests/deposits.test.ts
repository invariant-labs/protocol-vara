import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync, sortTokens } from '../src/test-utils.js'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any

const feeTier = newFeeTier(10000000000n, 1n)
const assertBalanceChange = async (changeX: bigint, changeY: bigint, callback: () => any) => {
  const userBalancesBefore = await invariant.getUserBalances(admin.addressRaw)

  const balanceXBefore = await GRC20.balanceOf(admin.addressRaw, token0Address)
  const balanceYBefore = await GRC20.balanceOf(admin.addressRaw, token1Address)
  const invariantBalanceXBefore = await GRC20.balanceOf(invariant.programId(), token0Address)
  const invariantBalanceYBefore = await GRC20.balanceOf(invariant.programId(), token1Address)

  await callback()
  const userBalancesAfter = await invariant.getUserBalances(admin.addressRaw)

  assert.deepEqual(
    userBalancesAfter.get(token0Address) || 0n,
    (userBalancesBefore.get(token0Address) || 0n) - changeX,
    'tokenX, user balance mismatch'
  )
  assert.deepEqual(
    userBalancesAfter.get(token1Address) || 0n,
    (userBalancesBefore.get(token1Address) || 0n) - changeY,
    'tokenY, user balance mismatch'
  )

  const balanceXAfter = await GRC20.balanceOf(admin.addressRaw, token0Address)
  const balanceYAfter = await GRC20.balanceOf(admin.addressRaw, token1Address)
  const invariantBalanceXAfter = await GRC20.balanceOf(invariant.programId(), token0Address)
  const invariantBalanceYAfter = await GRC20.balanceOf(invariant.programId(), token1Address)

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
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    ;[token0Address, token1Address] = sortTokens(token0Address, token1Address)
    await invariant.addFeeTier(admin, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(admin, poolKey, 1000000000000000000000000n)

    await GRC20.mint(admin.addressRaw, 100000000000000n, token0Address)
    await GRC20.mint(admin.addressRaw, 100000000000000n, token1Address)

    await GRC20.approve(admin, invariant.programId(), 100000000000000n, token0Address)
    await GRC20.approve(admin, invariant.programId(), 100000000000000n, token1Address)
  })

  it('single deposit and withdraw', async function () {
    this.timeout(200000)
    const amount = 10000000000000n
    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(await invariant.depositSingleToken(admin, token0Address, amount), amount)
      assert.deepEqual(await invariant.depositSingleToken(admin, token1Address, amount), amount)
    })
    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(await invariant.withdrawSingleToken(admin, token0Address, amount), amount)
      assert.deepEqual(await invariant.withdrawSingleToken(admin, token1Address, amount), amount)
    })

    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(await invariant.depositSingleToken(admin, token0Address, amount), amount)
      assert.deepEqual(await invariant.depositSingleToken(admin, token1Address, amount), amount)
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(await invariant.withdrawSingleToken(admin, token0Address, null), amount)
      assert.deepEqual(await invariant.withdrawSingleToken(admin, token1Address, null), amount)
    })
  })

  it('single deposit and withdraw errors', async function () {
    this.timeout(200000)
    const amount = 10000000000000n

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawSingleToken(admin, token0Address, amount),
        "Panic occurred: panicked with 'InvariantError: NoBalanceForTheToken'"
      )
    })

    await GRC20.setTransferFail(true, token0Address)
    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositSingleToken(admin, token0Address, amount),
        "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"
      )
    })

    await GRC20.setTransferFail(false, token0Address)
    await assertBalanceChange(-amount, 0n, async () => {
      await invariant.depositSingleToken(admin, token0Address, amount)
    })
    await GRC20.setTransferFail(true, token0Address)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawSingleToken(admin, token0Address, amount),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  it('deposit and withdraw token pair', async function () {
    this.timeout(80000)
    const amount = 10000000000000n
    await assertBalanceChange(-amount, -amount, async () => {
      assert.deepEqual(
        await invariant.depositTokenPair(admin, [token0Address, amount], [token1Address, amount]),
        [amount, amount]
      )
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(admin, [token0Address, amount], [token1Address, amount]),
        [amount, amount]
      )
    })

    await assertBalanceChange(-amount, -amount, async () => {
      await invariant.depositTokenPair(admin, [token0Address, amount], [token1Address, amount])
    })

    await assertBalanceChange(amount, amount, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(admin, [token0Address, null], [token1Address, null]),
        [amount, amount]
      )
    })

    await assertBalanceChange(-1n, -2n, async () => {
      assert.deepEqual(
        await invariant.depositTokenPair(admin, [token0Address, 1n], [token1Address, 2n]),
        [1n, 2n]
      )
    })

    await assertBalanceChange(1n, 2n, async () => {
      assert.deepEqual(
        await invariant.withdrawTokenPair(admin, [token0Address, 1n], [token1Address, 2n]),
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
          [token0Address, 100n],
          [token1Address, 100n]
        ),
        [100n, 100n]
      )
    })

    await GRC20.setTransferFail(true, token1Address)

    await assertBalanceChange(1n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await GRC20.setTransferFail(true, token0Address)
    await GRC20.setTransferFail(false, token1Address)

    await assertBalanceChange(0n, 1n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await GRC20.setTransferFail(true, token0Address)
    await GRC20.setTransferFail(true, token1Address)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  it('deposit token pair errors', async function () {
    this.timeout(200000)
    await GRC20.setTransferFail(true, token0Address)
    await GRC20.setTransferFail(true, token1Address)

    await assertBalanceChange(0n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"
      )
    })

    await GRC20.setTransferFail(false, token1Address)

    await assertBalanceChange(0n, -1n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })

    await GRC20.setTransferFail(false, token0Address)
    await GRC20.setTransferFail(true, token1Address)

    await assertBalanceChange(-1n, 0n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [token0Address, 1n], [token1Address, 1n]),
        "Panic occurred: panicked with 'InvariantError: RecoverableTransferError'"
      )
    })
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
