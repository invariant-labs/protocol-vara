import 'mocha'
import { initGearApi, newFeeTier, newPoolKey, subscribeToNewHeads } from '../src/utils.js'
import { GearKeyring } from '@gear-js/api'
import { LOCAL } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20'
import { objectEquals, sortTokens } from '../src/test-utils.js'
import { PoolKey, SqrtPrice } from '../src/schema'
import { Position, getLiquidityByX, getLiquidityByY } from 'invariant-vara-wasm'

const api = await initGearApi({ providerAddress: LOCAL })
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
let tokenX: FungibleToken = null as any
let tokenY: FungibleToken = null as any
let invariant: Invariant = null as any
let poolKey: PoolKey = null as any
const positionOwner = user
const providedAmount = 430000n
const feeTier = newFeeTier(6000000000n, 10n)

describe('math', async function () {
  this.timeout(200000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
  it('get-liquidity-by-x', async function () {
    beforeEach(async function () {
      this.timeout(80000)

      invariant = await Invariant.deploy(api, admin, 10000000000n)
      tokenX = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      tokenY = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      ;[tokenX, tokenY] = sortTokens(tokenX, tokenY)

      poolKey = newPoolKey(tokenX.programId(), tokenY.programId(), feeTier)

      await invariant.addFeeTier(admin, feeTier)

      const initSqrtPrice: SqrtPrice = 1005012269622000000000000n

      await invariant.createPool(admin, poolKey, initSqrtPrice)
      await tokenX.mint(admin.addressRaw, 10000000000n)
      await tokenY.mint(admin.addressRaw, 10000000000n)
      await tokenX.approve(admin, invariant.programId(), 10000000000n)
      await tokenY.approve(admin, invariant.programId(), 10000000000n)
      await invariant.depositSingleToken(admin, tokenX.programId(), 10000000000n)
      await invariant.depositSingleToken(admin, tokenY.programId(), 10000000000n)
    })
    it('check get liquidity by x', async function () {
      this.timeout(200000)
      // below range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        getLiquidityByX(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
      }
      // in range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await tokenX.mint(positionOwner.addressRaw, providedAmount)
        await tokenX.approve(positionOwner, invariant.programId(), providedAmount)
        await tokenY.mint(positionOwner.addressRaw, amount)
        await tokenY.approve(positionOwner, invariant.programId(), amount)
        await invariant.depositSingleToken(positionOwner, tokenX.programId(), providedAmount)
        await invariant.depositSingleToken(positionOwner, tokenY.programId(), amount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          0n
        )

        const position = await invariant.getPosition(positionOwner.addressRaw, 0n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
          lowerTickIndex: lowerTickIndex,
          upperTickIndex: upperTickIndex,
          feeGrowthInsideX: 0n,
          feeGrowthInsideY: 0n,
          lastBlockNumber: 0n,
          tokensOwedX: 0n,
          tokensOwedY: 0n
        }

        objectEquals(position, expectedPosition, ['lastBlockNumber'])
      }
      // above range
      {
        const lowerTickIndex = 150n
        const upperTickIndex = 800n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await tokenX.mint(positionOwner.addressRaw, providedAmount)
        await tokenX.approve(positionOwner, invariant.programId(), providedAmount)
        await invariant.depositSingleToken(positionOwner, tokenX.programId(), providedAmount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          0n
        )

        const position = await invariant.getPosition(positionOwner.addressRaw, 1n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
  })

  it('get liquidity by y', async function () {
    this.timeout(200000)

    const providedAmount = 47600000000n
    const feeTier = newFeeTier(6000000000n, 10n)

    let poolKey = newPoolKey(tokenX.programId(), tokenY.programId(), feeTier)

    beforeEach(async function () {
      invariant = await Invariant.deploy(api, admin, 10000000000n)
      tokenX = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      tokenY = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

      poolKey = newPoolKey(tokenX.programId(), tokenY.programId(), feeTier)

      await invariant.addFeeTier(admin, feeTier)

      const initSqrtPrice: SqrtPrice = 367897834491000000000000n

      await invariant.createPool(admin, poolKey, initSqrtPrice)
      await tokenX.mint(admin.addressRaw, 10000000000n)
      await tokenY.mint(admin.addressRaw, 10000000000n)
      await tokenX.approve(admin, invariant.programId(), 10000000000n)
      await tokenY.approve(admin, invariant.programId(), 10000000000n)
      await invariant.depositSingleToken(admin, tokenX.programId(), 10000000000n)
      await invariant.depositSingleToken(admin, tokenY.programId(), 10000000000n)
    })

    it('check get liquidity by y', async function () {
      this.timeout(80000)

      // below range
      {
        const lowerTickIndex = -22000n
        const upperTickIndex = -21000n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await tokenY.mint(positionOwner.addressRaw, providedAmount)
        await tokenY.approve(positionOwner, invariant.programId(), providedAmount)
        await invariant.depositSingleToken(positionOwner, tokenY.programId(), providedAmount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          0n
        )

        const position = await invariant.getPosition(positionOwner.addressRaw, 0n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
      // in range
      {
        const lowerTickIndex = -25000n
        const upperTickIndex = -19000n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await tokenY.mint(positionOwner.addressRaw, providedAmount)
        await tokenY.approve(positionOwner, invariant.programId(), providedAmount)
        await tokenX.mint(positionOwner.addressRaw, amount)
        await tokenX.approve(positionOwner, invariant.programId(), amount)

        await invariant.depositSingleToken(positionOwner, tokenY.programId(), providedAmount)
        await invariant.depositSingleToken(positionOwner, tokenX.programId(), amount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          0n
        )

        const position = await invariant.getPosition(positionOwner.addressRaw, 1n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
      // above range
      {
        const lowerTickIndex = -10000n
        const upperTickIndex = 0n

        const pool = await invariant.getPool(tokenX.programId(), tokenY.programId(), feeTier)

        assert.throw(() => {
          getLiquidityByY(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
        })
      }
    })
  })
  this.afterAll(async function () {
    await unsub!.then(unsub => unsub())
  })
})
