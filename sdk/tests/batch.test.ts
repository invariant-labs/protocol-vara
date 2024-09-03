import 'mocha'
import {
  BatchError,
  batchTxs,
  initGearApi,
  subscribeToNewHeads,
  toPercentage
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any

describe('events', async function () {
  this.timeout(80000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
  this.beforeEach(async function () {
    this.timeout(80000)

    invariant = await Invariant.deploy(api, admin, toPercentage(1n, 2n))
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
  })

  it('batch deposit and withdraw', async function () {
    this.timeout(20000)
    await batchTxs(api, admin, [
      await GRC20.mintTx(admin.addressRaw, 1000000000000n, token0Address),
      await GRC20.mintTx(admin.addressRaw, 1000000000000n, token1Address),
      await GRC20.approveTx(invariant.programId(), 1000000000000n, token0Address),
      await GRC20.approveTx(invariant.programId(), 1000000000000n, token1Address),
      await invariant.depositSingleTokenTx(token0Address, 1000000000000n),
      await invariant.depositSingleTokenTx(token1Address, 1000000000000n)
    ])

    {
      const balances = await invariant.getUserBalances(admin.addressRaw)
      assert.deepEqual(
        [balances.get(token0Address), balances.get(token1Address)],
        [1000000000000n, 1000000000000n]
      )
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token0Address), 0n)
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token1Address), 0n)
      assert.strictEqual(
        await GRC20.balanceOf(invariant.programId(), token0Address),
        1000000000000n
      )
      assert.strictEqual(
        await GRC20.balanceOf(invariant.programId(), token1Address),
        1000000000000n
      )
    }

    await batchTxs(api, admin, [
      await invariant.withdrawSingleTokenTx(token0Address, 1000000000000n),
      await invariant.withdrawSingleTokenTx(token1Address, 1000000000000n)
    ])

    {
      const balances = await invariant.getUserBalances(admin.addressRaw)
      assert.deepEqual(
        [balances.get(token0Address), balances.get(token1Address)],
        [undefined, undefined]
      )
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token0Address), 1000000000000n)
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token1Address), 1000000000000n)
      assert.strictEqual(await GRC20.balanceOf(invariant.programId(), token0Address), 0n)
      assert.strictEqual(await GRC20.balanceOf(invariant.programId(), token1Address), 0n)
    }
  })

  it('batch errors', async function () {
    this.timeout(20000)
    try {
      await batchTxs(api, admin, [
        await GRC20.burnTx(admin.addressRaw, 1000000000000n, token0Address),
        await GRC20.burnTx(admin.addressRaw, 1000000000000n, token1Address),
        await GRC20.approveTx(invariant.programId(), 1000000000000n, token0Address),
        await GRC20.approveTx(invariant.programId(), 1000000000000n, token1Address),
        await invariant.depositSingleTokenTx(token0Address, 1000000000000n),
        await invariant.depositSingleTokenTx(token1Address, 1000000000000n)
      ])
    } catch (e) {
      assert(e instanceof BatchError)
      assert.deepEqual(
        e.message,
        `Batch error occurred\n` +
          `Request number 0 failed: Panic occurred: Underflow\n` +
          `Request number 1 failed: Panic occurred: Underflow\n` +
          `Request number 4 failed: Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'\n` +
          `Request number 5 failed: Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'`
      )
      assert.deepEqual(
        e.failedTxs,
        new Map([
          [0, 'Panic occurred: Underflow'],
          [1, 'Panic occurred: Underflow'],
          [4, "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"],
          [5, "Panic occurred: panicked with 'InvariantError: UnrecoverableTransferError'"]
        ])
      )
    }

    {
      const balances = await invariant.getUserBalances(admin.addressRaw)
      assert.deepEqual(
        [balances.get(token0Address), balances.get(token1Address)],
        [undefined, undefined]
      )
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token0Address), 0n)
      assert.strictEqual(await GRC20.balanceOf(admin.addressRaw, token1Address), 0n)
      assert.strictEqual(await GRC20.balanceOf(invariant.programId(), token0Address), 0n)
      assert.strictEqual(await GRC20.balanceOf(invariant.programId(), token1Address), 0n)
    }
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
