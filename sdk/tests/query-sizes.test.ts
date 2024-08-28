import 'mocha'
import {
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads,
  getMinTick,
  getMaxTick
} from '../src/utils.js'
import { decodeAddress, GearKeyring, HexString } from '@gear-js/api'
import { CHUNK_SIZE, DEFAULT_ADDRESS, Network } from '../src/consts.js'
import { Invariant } from '../src/invariant.js'
import { FungibleToken } from '../src/erc20.js'
import { assert } from 'chai'
import { getPercentageDenominator } from '@invariant-labs/vara-sdk-wasm'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const initProtocolFee = 10000000000n
const feeTier = newFeeTier(getPercentageDenominator() - 1n, 100n)
let poolKey = null as any

describe('query sizes', async function () {
  this.timeout(40000)

  this.beforeAll(async function () {
    this.timeout(300000)

    unsub = subscribeToNewHeads(api)

    invariant = await Invariant.deploy(api, admin, initProtocolFee)
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    await GRC20.mint(user.addressRaw, 1000000000000n, token0Address)
    await GRC20.mint(user.addressRaw, 1000000000000n, token1Address)
    await invariant.addFeeTier(admin, feeTier)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)
    await invariant.createPool(user, poolKey, 1000000000000000000000000n)

    await GRC20.approve(user, invariant.programId(), 1000000000000n, token0Address)
    await GRC20.approve(user, invariant.programId(), 1000000000000n, token1Address)
    await invariant.depositSingleToken(user, token0Address, 1000000000000n)
    await invariant.depositSingleToken(user, token1Address, 1000000000000n)
    for (let i = 0n; i < CHUNK_SIZE; i++) {
      await invariant.createPosition(
        user,
        poolKey,
        getMinTick(100n) + i * 100n,
        getMaxTick(100n) - i * 100n,
        10n,
        1000000000000000000000000n,
        0n
      )
    }
  })

  it('liquidity tick', async () => {
    const payload = invariant.contract.registry
      .createType('(String, String, PoolKey, Vec<i32>)', [
        'Service',
        'GetLiquidityTicks',
        poolKey,
        [getMinTick(100n)]
      ])
      .toHex()
    const reply = await invariant.contract.api.message.calculateReply({
      destination: invariant.contract.programId!,
      origin: decodeAddress(DEFAULT_ADDRESS),
      payload,
      value: 0,
      gasLimit: invariant.contract.api.blockGasLimit.toBigInt()
    })
    assert.equal(reply.payload.length, 65)
  })

  it('tickmap with 2 ticks', async () => {
    const payload = invariant.contract.registry
      .createType('(String, String, PoolKey)', ['Service', 'GetTickmap', poolKey])
      .toHex()

    const reply = await invariant.contract.api.message.calculateReply({
      destination: invariant.contract.programId!,
      origin: decodeAddress(DEFAULT_ADDRESS),
      payload,
      value: 0,
      gasLimit: invariant.contract.api.blockGasLimit.toBigInt()
    })

    assert.equal(reply.payload.length, 50)
  })

  it('positions with one position', async () => {
    const payload = invariant.contract.registry
      .createType('(String, String, [u8;32], u32, u32)', [
        'Service',
        'GetPositions',
        user.addressRaw,
        1,
        0
      ])
      .toHex()

    const reply = await invariant.contract.api.message.calculateReply({
      destination: invariant.contract.programId!,
      origin: decodeAddress(DEFAULT_ADDRESS),
      payload,
      value: 0,
      gasLimit: invariant.contract.api.blockGasLimit.toBigInt()
    })

    assert.equal(reply.payload.length, 450)
  })

  it('2 position ticks', async () => {
    const payload = invariant.contract.registry
      .createType('(String, String, [u8;32], u32)', [
        'Service',
        'GetPositionTicks',
        user.addressRaw,
        CHUNK_SIZE - 1n
      ])
      .toHex()

    const reply = await invariant.contract.api.message.calculateReply({
      destination: invariant.contract.programId!,
      origin: decodeAddress(DEFAULT_ADDRESS),
      payload,
      value: 0,
      gasLimit: invariant.contract.api.blockGasLimit.toBigInt()
    })

    assert.equal(reply.payload.length, 114)
  })

  it('1 pool for token pair', async () => {
    const payload = invariant.contract.registry
      .createType('(String, String, [u8;32], [u8;32])', [
        'Service',
        'GetAllPoolsForPair',
        token0Address,
        token1Address
      ])
      .toHex()

    const reply = await invariant.contract.api.message.calculateReply({
      destination: invariant.contract.programId!,
      origin: decodeAddress(DEFAULT_ADDRESS),
      payload,
      value: 0,
      gasLimit: invariant.contract.api.blockGasLimit.toBigInt()
    })
    assert.equal(reply.payload.length, 243)
  })

  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
