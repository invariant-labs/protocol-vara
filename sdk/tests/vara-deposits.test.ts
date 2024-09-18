import 'mocha'
import { initGearApi, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring } from '@gear-js/api'
import { Network } from '../src/network'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync } from '../src/test-utils.js'
import { VARA_ADDRESS } from '../src/consts'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let invariant: Invariant = null as any

const assertVaraBalanceChange = async (deltaVara: bigint, callback: () => any) => {
  const userVaraBalanceBefore = (await api.balance.findOut(admin.address)).toBigInt()
  const invariantVaraBalanceBefore = (await api.balance.findOut(invariant.programId())).toBigInt()
  const userBalancesBefore = await invariant.getUserBalances(admin.addressRaw)
  await callback()

  const userVaraBalanceAfter = (await api.balance.findOut(admin.address)).toBigInt()
  const invariantVaraBalanceAfter = (await api.balance.findOut(invariant.programId())).toBigInt()
  const userBalancesAfter = await invariant.getUserBalances(admin.addressRaw)
  const grc20KeysBefore = Array.from(userBalancesAfter.keys())
  const grc20KeysAfter = Array.from(userBalancesBefore.keys())

  assert(
    grc20KeysBefore.filter(v => v !== VARA_ADDRESS).length -
      grc20KeysAfter.filter(v => v !== VARA_ADDRESS).length ===
      0,
    `One of the grc20 token transfers was executed, actual: ${grc20KeysAfter}, expected: ${grc20KeysBefore}`
  )

  assert.deepEqual(
    userBalancesAfter.get(VARA_ADDRESS) || 0n,
    (userBalancesBefore.get(VARA_ADDRESS) || 0n) - deltaVara,
    'user balance mismatch in contract state'
  )
  const precision = 10000000000000n //necessary due to gas costs
  assert(precision >= deltaVara, 'Precision too small')
  assert(
    userVaraBalanceAfter + precision >= userVaraBalanceBefore + deltaVara &&
      userVaraBalanceAfter <= userVaraBalanceBefore + deltaVara,
    `user balance mismatch, actual: ${userVaraBalanceAfter}, expected: ${
      userVaraBalanceBefore + deltaVara
    }`
  )

  assert.deepEqual(
    invariantVaraBalanceAfter,
    invariantVaraBalanceBefore - deltaVara,
    'invariant balance mismatch'
  )
}

describe('vara-deposits', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })

  beforeEach(async function () {
    this.timeout(200000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
  })

  it('deposit and withdraw', async function () {
    this.timeout(200000)

    const amount = api.existentialDeposit.toBigInt()
    await assertVaraBalanceChange(-amount, async () => {
      await invariant.depositVara(admin, amount)
    })
    await assertVaraBalanceChange(amount, async () => {
      await invariant.withdrawVara(admin, amount)
    })
    await assertVaraBalanceChange(0n, async () => {
      await invariant.withdrawVara(admin, null)
    })
  })

  it('deposit and withdraw failures', async function () {
    this.timeout(200000)

    const balance = (await api.balance.findOut(admin.address)).toBigInt()
    const amount = api.existentialDeposit.toBigInt()
    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.depositVara(admin, balance + 1n),
        `{"docs":"Insufficient user balance.","method":"InsufficientBalance","name":"InsufficientBalance"}`
      )
    })
    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.depositVara(admin, amount - 1n),
        'Value is less than existential deposit'
      )
    })

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawVara(admin, amount),
        "Panic occurred: panicked with 'InvariantError: NoBalanceForTheToken'"
      )
    })

    await assertVaraBalanceChange(-amount * 2n, async () => {
      await invariant.depositVara(admin, amount * 2n)
    })

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawVara(admin, amount * 3n),
        "Panic occurred: panicked with 'InvariantError: FailedToChangeTokenBalance'"
      )
    })

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawVara(admin, amount - 1n),
        "Value is less than existential deposit"
      )
    })
  })

  it('deposit and withdraw with normal entrypoints', async function () {
    this.timeout(200000)

    const token = await FungibleToken.deploy(api, admin) //any address would fit really
    const amount = api.existentialDeposit.toBigInt()
    await GRC20.mint(admin.addressRaw, amount, token)
    await GRC20.approve(admin, invariant.programId(), amount, token)

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.depositSingleToken(admin, VARA_ADDRESS, amount),
        "Panic occurred: panicked with 'InvariantError: InvalidVaraDepositAttempt'"
      )
    })

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.depositTokenPair(admin, [VARA_ADDRESS, amount], [token, amount]),
        "Panic occurred: panicked with 'InvariantError: InvalidVaraDepositAttempt'"
      )
    })

    await assertVaraBalanceChange(-amount, async () => {
      await invariant.depositVara(admin, amount)
    })

    await invariant.depositSingleToken(admin, token, amount)

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawTokenPair(admin, [VARA_ADDRESS, amount], [token, amount]),
        "Panic occurred: panicked with 'InvariantError: InvalidVaraWithdrawAttempt'"
      )
    })

    await assertVaraBalanceChange(0n, async () => {
      await assertThrowsAsync(
        invariant.withdrawSingleToken(admin, VARA_ADDRESS, amount),
        "Panic occurred: panicked with 'InvariantError: InvalidVaraWithdrawAttempt'"
      )
    })

    await assertVaraBalanceChange(amount, async () => {
      await invariant.withdrawVara(admin, amount)
    })
  })
  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
