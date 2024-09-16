import 'mocha'
import {
  PositionRemovedEvent,
  PositionCreatedEvent,
  Pool,
  PoolKey,
  Position,
  SqrtPrice,
  TokenAmount,
  InvariantEvent
} from '../src/schema'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { FungibleToken } from '../src/erc20'
import { assertThrowsAsync, objectEquals, sortTokens } from '../src/test-utils'
import {
  calculateTokenAmounts,
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads,
  getLiquidityByX
} from '../src/utils'
import { u8aToHex } from '@polkadot/util'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/network'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
let tokenXAddress: HexString = null as any
let tokenYAddress: HexString = null as any
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let invariant: Invariant = null as any
const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey: PoolKey = null as any
let pool: Pool = null as any

let createEvent: any
let removeEvent: any

describe('position', async function () {
  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
  beforeEach(async function () {
    this.timeout(80000)
    invariant = await Invariant.deploy(api, admin, 10000000000n)
    tokenXAddress = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    tokenYAddress = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
    ;[tokenXAddress, tokenYAddress] = sortTokens(tokenXAddress, tokenYAddress)
    poolKey = newPoolKey(tokenXAddress, tokenYAddress, feeTier)

    await invariant.addFeeTier(admin, feeTier)

    await invariant.createPool(user, poolKey, 1000000000000000000000000n)

    pool = await invariant.getPool(tokenXAddress, tokenYAddress, feeTier)

    invariant.on({
      ident: InvariantEvent.PositionCreatedEvent,
      callback: event => {
        createEvent = event
      }
    })
    invariant.on({
      ident: InvariantEvent.PositionRemovedEvent,
      callback: event => {
        removeEvent = event
      }
    })

    await GRC20.mint(user.addressRaw, 10000000000n, tokenXAddress)
    await GRC20.mint(user.addressRaw, 10000000000n, tokenYAddress)
    await GRC20.approve(user, invariant.programId(), 10000000000n, tokenXAddress)
    await GRC20.approve(user, invariant.programId(), 10000000000n, tokenYAddress)

    await invariant.depositSingleToken(user, tokenXAddress, 10000000000n)
    await invariant.depositSingleToken(user, tokenYAddress, 10000000000n)

    await invariant.createPosition(
      user,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      100000000000n,
      pool.sqrtPrice,
      0n
    )

    const expectedCreatePositionEvent: PositionCreatedEvent = {
      address: u8aToHex(user.addressRaw),
      sqrtPrice: 1000000000000000000000000n,
      liquidityDelta: 100000000000n,
      lowerTick: -20n,
      poolKey,
      upperTick: 10n,
      timestamp: 0n
    }
    objectEquals(createEvent, expectedCreatePositionEvent, ['timestamp'])
  })

  it('create position', async function () {
    this.timeout(80000)

    const position = await invariant.getPosition(user.addressRaw, 0n)
    const expectedPosition: Position = {
      poolKey: poolKey,
      liquidity: 100000000000n,
      lowerTickIndex: lowerTickIndex,
      upperTickIndex: upperTickIndex,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      lastBlockNumber: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    await objectEquals(position, expectedPosition, ['lastBlockNumber'])
  })
  it('calculate token amounts from position liquidity', async function () {
    this.timeout(80000)

    const position = await invariant.getPosition(user.addressRaw, 0n)
    const pool = await invariant.getPool(tokenXAddress, tokenYAddress, feeTier)

    const providedAmount = 500n
    const { amount: expectedYAmount } = getLiquidityByX(
      providedAmount,
      lowerTickIndex,
      upperTickIndex,
      pool.sqrtPrice,
      false
    )

    const [x, y] = calculateTokenAmounts(pool, position)
    // 1n diffrence in result comes from rounding in `getLiquidityByX`
    assert.deepEqual(x, providedAmount - 1n)
    assert.deepEqual(y, expectedYAmount)
  })
  it('remove position', async function () {
    this.timeout(80000)

    {
      await invariant.removePosition(user, 0n)

      const expectedRemovePositionEvent: PositionRemovedEvent = {
        address: u8aToHex(user.addressRaw),
        sqrtPrice: 1000000000000000000000000n,
        liquidityDelta: 100000000000n,
        lowerTick: -20n,
        poolKey,
        upperTick: 10n,
        timestamp: 0n
      }

      objectEquals(removeEvent, expectedRemovePositionEvent, ['timestamp'])

      await assertThrowsAsync(invariant.getPosition(user.addressRaw, 0n), 'Error: PositionNotFound')
      const positions = await invariant.getAllPositions(admin.addressRaw)
      console.log(positions)
      assert.deepEqual(positions.length, 1)
      assert.deepEqual(positions[0].entries.length, 0)
    }
    {
      await assertThrowsAsync(invariant.getTick(poolKey, lowerTickIndex), 'Error: TickNotFound')

      await assertThrowsAsync(invariant.getTick(poolKey, upperTickIndex), 'Error: TickNotFound')

      const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, lowerTickIndex)
      assert.exists(!isLowerTickInitialized)

      const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, upperTickIndex)

      assert.exists(!isUpperTickInitialized)
    }
  })

  it('transfer position', async function () {
    this.timeout(80000)
    const positionOwner = user
    const receiver = admin
    {
      await invariant.transferPosition(positionOwner, 0n, receiver.addressRaw)

      await assertThrowsAsync(
        invariant.getPosition(positionOwner.addressRaw, 0n),
        'Error: PositionNotFound'
      )
      const position = await invariant.getPosition(receiver.addressRaw, 0n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: 100000000000n,
        lowerTickIndex: lowerTickIndex,
        upperTickIndex: upperTickIndex,
        feeGrowthInsideX: 0n,
        feeGrowthInsideY: 0n,
        lastBlockNumber: 0n,
        tokensOwedX: 0n,
        tokensOwedY: 0n
      }
      await objectEquals(position, expectedPosition, ['lastBlockNumber'])
    }
  })

  it('claim fee', async function () {
    this.timeout(100000)
    const positionOwner = user
    const swapper = admin
    //clear balances from beforeEach block

    await invariant.withdrawSingleToken(positionOwner, tokenXAddress)
    await invariant.withdrawSingleToken(positionOwner, tokenYAddress)
    {
      const amount: TokenAmount = 1000n

      await GRC20.mint(swapper.addressRaw, amount, tokenXAddress)
      await GRC20.approve(swapper, invariant.programId(), amount, tokenXAddress)
      await invariant.depositSingleToken(swapper, tokenXAddress, amount)

      const poolBefore = await invariant.getPool(tokenXAddress, tokenYAddress, feeTier)

      const targetSqrtPrice: SqrtPrice = 15258932000000000000n
      await invariant.swap(swapper, poolKey, true, amount, true, targetSqrtPrice)

      await invariant.withdrawSingleToken(swapper, tokenYAddress)
      const poolAfter = await invariant.getPool(tokenXAddress, tokenYAddress, feeTier)

      await assertThrowsAsync(
        invariant.withdrawSingleToken(swapper, tokenXAddress, amount),
        "Panic occurred: panicked with 'InvariantError: NoBalanceForTheToken'"
      )

      const swapperX = await GRC20.balanceOf(swapper.addressRaw, tokenXAddress)
      const swapperY = await GRC20.balanceOf(swapper.addressRaw, tokenYAddress)

      assert.equal(swapperX, 0n)
      assert.equal(swapperY, 993n)
      const invariantX = await GRC20.balanceOf(invariant.programId(), tokenXAddress)
      const invariantY = await GRC20.balanceOf(invariant.programId(), tokenYAddress)
      assert.equal(invariantX, 1500n)
      assert.equal(invariantY, 7n)
      assert.deepEqual(poolAfter.liquidity, poolBefore.liquidity)
      assert.notDeepEqual(poolAfter.sqrtPrice, poolBefore.sqrtPrice)
      assert.deepEqual(poolAfter.currentTickIndex, lowerTickIndex)
      assert.deepEqual(poolAfter.feeGrowthGlobalX, 50000000000000000000000n)
      assert.deepEqual(poolAfter.feeGrowthGlobalY, 0n)
      assert.deepEqual(poolAfter.feeProtocolTokenX, 1n)
      assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)
    }
    {
      const positionOwnerBeforeX = await GRC20.balanceOf(positionOwner.addressRaw, tokenXAddress)
      const invariantBeforeX = await GRC20.balanceOf(invariant.programId(), tokenXAddress)
      await invariant.claimFee(positionOwner, 0n)
      await invariant.withdrawSingleToken(positionOwner, tokenXAddress)
      const withdrawnTokens = await invariant.withdrawSingleToken(positionOwner, tokenYAddress)
      assert.deepEqual(withdrawnTokens, 0n)

      const positionOwnerAfterX = await GRC20.balanceOf(positionOwner.addressRaw, tokenXAddress)

      const invariantAfterX = await GRC20.balanceOf(invariant.programId(), tokenXAddress)

      const position = await invariant.getPosition(positionOwner.addressRaw, 0n)
      const pool = await invariant.getPool(tokenXAddress, tokenYAddress, feeTier)
      const expectedTokensClaimed = 5n
      console.log(invariantAfterX)
      assert.deepEqual(positionOwnerAfterX - expectedTokensClaimed, positionOwnerBeforeX)
      assert.deepEqual(invariantAfterX + expectedTokensClaimed, invariantBeforeX)

      assert.deepEqual(position.feeGrowthInsideX, pool.feeGrowthGlobalX)
      assert.deepEqual(position.tokensOwedX, 0n)
    }
  })

  it('slippage tolerance works', async function () {
    this.timeout(80000)
    await invariant.createPosition(
      user,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      10000000000000n,
      pool.sqrtPrice,
      10000000000n
    )

    await invariant.createPosition(
      user,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      10000000000000n,
      953462589245592315446776n,
      100000000000n
    )

    await assertThrowsAsync(
      invariant.createPosition(
        user,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        10000000000000n,
        953462589245592315446775n,
        100000000000n
      )
    )
  })
  this.afterAll(async function () {
    await unsub!.then(u => u())
  })
})
