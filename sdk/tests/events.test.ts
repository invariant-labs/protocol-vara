import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { objectEquals } from '../src/test-utils.js'
import {
  PoolKey,
  InvariantEvent,
  PositionCreatedEvent,
  CrossTickEvent,
  PositionRemovedEvent,
  SwapEvent
} from '../src/schema'
import { decodeAddress } from '@gear-js/api'
import { getGlobalMinSqrtPrice, toPercentage, toSqrtPrice } from 'invariant-vara-wasm'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')

let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any
let invariant: Invariant = null as any
const feeTier = newFeeTier(10000000000n, 1n)
let poolKey: PoolKey = null as any

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
    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(admin, feeTier)
    await invariant.createPool(admin, poolKey, toSqrtPrice(1n, 0n))

    await GRC20.mint(admin.addressRaw, 1000000000000n, token0Address)
    await GRC20.mint(admin.addressRaw, 1000000000000n, token1Address)
    await GRC20.approve(admin, invariant.programId(), 1000000000000n, token0Address)
    await GRC20.approve(admin, invariant.programId(), 1000000000000n, token1Address)
    await invariant.depositSingleToken(admin, token0Address, 1000000000000n)
    await invariant.depositSingleToken(admin, token1Address, 1000000000000n)
  })

  it('create position event', async function () {
    this.timeout(80000)

    let wasFired = false

    const expectedPositionCreatedEvent: PositionCreatedEvent = {
      address: decodeAddress(admin.address),
      sqrtPrice: toSqrtPrice(1n, 0n),
      liquidityDelta: 1000000000000n,
      lowerTick: -10n,
      poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    invariant.on({
      ident: InvariantEvent.PositionCreatedEvent,
      callback: (event: PositionCreatedEvent) => {
        objectEquals(event, expectedPositionCreatedEvent, ['timestamp'])
        wasFired = true
      }
    })

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(wasFired, true)
  })

  it('cross tick and swap event', async function () {
    this.timeout(80000)

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    await invariant.createPosition(
      admin,
      poolKey,
      -30n,
      -10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    await invariant.createPosition(
      admin,
      poolKey,
      -50n,
      -30n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    let wasSwapEventFired = false
    let wasCrossTickEventFired = false

    const expectedCrossTickEvent: CrossTickEvent = {
      address: decodeAddress(admin.address),
      poolKey,
      timestamp: 0n,
      indexes: [-10n, -30n]
    }

    const expectedSwapEvent: SwapEvent = {
      address: decodeAddress(admin.address),
      poolKey,
      amountIn: 2500n,
      amountOut: 2464n,
      fee: 27n,
      startSqrtPrice: 1000000000000000000000000n,
      targetSqrtPrice: 997534045508480530459903n,
      xToY: true,
      timestamp: 0n
    }

    invariant.on({
      ident: InvariantEvent.CrossTickEvent,
      callback: (event: CrossTickEvent) => {
        objectEquals(event, expectedCrossTickEvent, ['timestamp'])
        wasCrossTickEventFired = true
      }
    })

    invariant.on({
      ident: InvariantEvent.SwapEvent,
      callback: (event: SwapEvent) => {
        objectEquals(event, expectedSwapEvent, ['timestamp'])
        wasSwapEventFired = true
      }
    })

    await invariant.swap(admin, poolKey, true, 2500n, true, getGlobalMinSqrtPrice())

    assert.deepEqual(wasCrossTickEventFired, true)
    assert.deepEqual(wasSwapEventFired, true)
  })

  it('remove position event', async function () {
    this.timeout(80000)

    let wasFired = false

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    const expectedPositionRemovedEvent: PositionRemovedEvent = {
      address: decodeAddress(admin.address),
      sqrtPrice: toSqrtPrice(1n, 0n),
      liquidityDelta: 1000000000000n,
      lowerTick: -10n,
      poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    invariant.on({
      ident: InvariantEvent.PositionRemovedEvent,
      callback: (event: PositionRemovedEvent) => {
        objectEquals(event, expectedPositionRemovedEvent, ['timestamp'])
        wasFired = true
      }
    })

    await invariant.removePosition(admin, 0n)
    assert.deepEqual(wasFired, true)
  })

  it('on and off methods', async function () {
    this.timeout(80000)

    let timesFired = 0

    const handler = () => {
      timesFired++
    }

    invariant.on({ ident: InvariantEvent.PositionCreatedEvent, callback: handler })

    await invariant.createPosition(
      admin,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 1)

    invariant.off({ ident: InvariantEvent.PositionCreatedEvent, callback: handler })

    await invariant.createPosition(
      admin,
      poolKey,
      -50n,
      50n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 1)

    invariant.on({
      ident: InvariantEvent.PositionCreatedEvent,
      callback: handler
    })

    await invariant.createPosition(
      admin,
      poolKey,
      -40n,
      40n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 2)
  })
  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
