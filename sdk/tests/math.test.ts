import 'mocha'
import {
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads,
  getLiquidityByX,
  getLiquidityByY
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { objectEquals, sortTokens } from '../src/test-utils.js'
import { Position, PoolKey, SqrtPrice } from '../src/schema'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')
const user = await GearKeyring.fromSuri('//Bob')

let unsub: Promise<VoidFunction> | null = null
let token0Address: HexString = null as any
let token1Address: HexString = null as any
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(admin)
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
      token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      ;[token0Address, token1Address] = sortTokens(token0Address, token1Address)

      poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.addFeeTier(admin, feeTier)

      const initSqrtPrice: SqrtPrice = 1005012269622000000000000n

      await invariant.createPool(admin, poolKey, initSqrtPrice)
      await GRC20.mint(admin.addressRaw, 10000000000n, token0Address)
      await GRC20.mint(admin.addressRaw, 10000000000n, token1Address)
      await GRC20.approve(admin, invariant.programId(), 10000000000n, token0Address)
      await GRC20.approve(admin, invariant.programId(), 10000000000n, token1Address)
      await invariant.depositSingleToken(admin, token0Address, 10000000000n)
      await invariant.depositSingleToken(admin, token1Address, 10000000000n)
    })
    it('check get liquidity by x', async function () {
      this.timeout(200000)
      // below range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

        getLiquidityByX(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
      }
      // in range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await GRC20.mint(positionOwner.addressRaw, providedAmount, token0Address)
        await GRC20.approve(positionOwner, invariant.programId(), providedAmount, token0Address)
        await GRC20.mint(positionOwner.addressRaw, amount, token1Address)
        await GRC20.approve(positionOwner, invariant.programId(), amount, token1Address)
        await invariant.depositSingleToken(positionOwner, token0Address, providedAmount)
        await invariant.depositSingleToken(positionOwner, token1Address, amount)

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

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await GRC20.mint(positionOwner.addressRaw, providedAmount, token0Address)
        await GRC20.approve(positionOwner, invariant.programId(), providedAmount, token0Address)
        await invariant.depositSingleToken(positionOwner, token0Address, providedAmount)

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

    let poolKey = newPoolKey(token0Address, token1Address, feeTier)

    beforeEach(async function () {
      invariant = await Invariant.deploy(api, admin, 10000000000n)
      token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)
      token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 0n)

      poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.addFeeTier(admin, feeTier)

      const initSqrtPrice: SqrtPrice = 367897834491000000000000n

      await invariant.createPool(admin, poolKey, initSqrtPrice)
      await GRC20.mint(admin.addressRaw, 10000000000n, token0Address)
      await GRC20.mint(admin.addressRaw, 10000000000n, token1Address)
      await GRC20.approve(admin, invariant.programId(), 10000000000n, token0Address)
      await GRC20.approve(admin, invariant.programId(), 10000000000n, token1Address)
      await invariant.depositSingleToken(admin, token0Address, 10000000000n)
      await invariant.depositSingleToken(admin, token1Address, 10000000000n)
    })

    it('check get liquidity by y', async function () {
      this.timeout(80000)

      // below range
      {
        const lowerTickIndex = -22000n
        const upperTickIndex = -21000n

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await GRC20.mint(positionOwner.addressRaw, providedAmount, token1Address)
        await GRC20.approve(positionOwner, invariant.programId(), providedAmount, token1Address)
        await invariant.depositSingleToken(positionOwner, token1Address, providedAmount)

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

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await GRC20.mint(positionOwner.addressRaw, providedAmount, token0Address)
        await GRC20.approve(positionOwner, invariant.programId(), providedAmount, token0Address)
        await GRC20.mint(positionOwner.addressRaw, amount, token1Address)
        await GRC20.approve(positionOwner, invariant.programId(), amount, token1Address)

        await invariant.depositSingleToken(positionOwner, token0Address, providedAmount)
        await invariant.depositSingleToken(positionOwner, token1Address, amount)

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

        const pool = await invariant.getPool(token0Address, token1Address, feeTier)

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
