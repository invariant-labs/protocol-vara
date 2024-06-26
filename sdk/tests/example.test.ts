import 'mocha'
import {
  calculateFee,
  calculateSqrtPriceAfterSlippage,
  initGearApi,
  newFeeTier,
  newPoolKey,
  priceToSqrtPrice,
  subscribeToNewHeads
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/consts'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20'
import { Pool, Tick, Position } from '../src/schema'
import { getLiquidityByY, toPercentage, toPrice } from 'invariant-vara-wasm'

const api = await initGearApi({ providerAddress: Network.Local })
const admin = await GearKeyring.fromSuri('//Alice')

let token0Address: HexString = null as any
let token1Address: HexString = null as any

{
  let token0 = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
  let token1 = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
  await token0.mint(admin.addressRaw, 1000000000000000000000000000000n)
  await token1.mint(admin.addressRaw, 1000000000000000000000000000000n)
  token0Address = token0.programId()
  token1Address = token1.programId()
}

const INVARIANT_ADDRESS = (await Invariant.deploy(api, admin, 0n)).programId()
let unsub: Promise<VoidFunction> = null as any
describe('sdk guide snippets', async function () {
  this.timeout(80000)

  this.beforeAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
  it('sdk guide', async function () {
    this.timeout(80000)

    // load invariant contract
    const invariant = await Invariant.load(api, INVARIANT_ADDRESS)

    // set fee tier, make sure that fee tier with specified parameters exists
    const feeTier = newFeeTier(toPercentage(1n, 2n), 1n) // fee: 0.01 = 1%, tick spacing: 1

    // If the fee tier does not exist, you have to add it
    const isAdded = await invariant.feeTierExists(feeTier)
    if (!isAdded) {
      await invariant.addFeeTier(admin, feeTier)
    }

    // set initial price of the pool, we set it to 1.00
    // all endpoints only accept sqrt price so we need to convert it before passing it
    const price = toPrice(1n, 0n)
    const initSqrtPrice = priceToSqrtPrice(price)

    // set pool key, make sure that pool with specified parameters does not exists
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(admin, poolKey, initSqrtPrice)

    // token y has 12 decimals and we want to add 8 actual tokens to our position
    const tokenYAmount = 8n * 10n ** 12n

    // set lower and upper tick indexes, we want to create position in range [-10, 10]
    const lowerTickIndex = -10n
    const upperTickIndex = 10n

    // calculate amount of token x we need to give to create position
    const { amount: tokenXAmount, l: positionLiquidity } = getLiquidityByY(
      tokenYAmount,
      lowerTickIndex,
      upperTickIndex,
      initSqrtPrice,
      true
    )

    // print amount of token x and y we need to give to create position based on parameters we passed
    console.log(tokenXAmount, tokenYAmount)

    // load token contracts
    const tokenX = await FungibleToken.load(api, poolKey.tokenX)
    const tokenY = await FungibleToken.load(api, poolKey.tokenY)

    // approve transfers of both tokens
    await tokenX.approve(admin, invariant.programId(), tokenXAmount)
    await tokenY.approve(admin, invariant.programId(), tokenYAmount)

    // deposit tokens in the contract
    await invariant.depositTokenPair(
      admin,
      [poolKey.tokenX, tokenXAmount],
      [poolKey.tokenY, tokenYAmount]
    )

    // create position
    const createPositionResult = await invariant.createPosition(
      admin,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      positionLiquidity,
      initSqrtPrice,
      0n
    )

    console.log(createPositionResult) // print transaction result
    {
      // withdraw tokens from the contract
      // passing null will try to withdraw all tokens in case no tokens are deposited
      const withdrawResult = await invariant.withdrawTokenPair(
        admin,
        [poolKey.tokenX, null],
        [poolKey.tokenY, null]
      )
      console.log(withdrawResult)
    }
    // we want to swap 6 token0
    // token0 has 12 decimals so we need to multiply it by 10^12
    const amount = 6n * 10n ** 12n

    // approve token x transfer
    await tokenX.approve(admin, invariant.programId(), amount)
    // deposit tokenX
    await invariant.depositSingleToken(admin, poolKey.tokenX, amount)

    // get estimated result of swap
    const quoteResult = await invariant.quote(poolKey, true, amount, true)

    // slippage is a price change you are willing to accept,
    // for examples if current price is 1 and your slippage is 1%, then price limit will be 1.01
    const allowedSlippage = toPercentage(1n, 3n) // 0.001 = 0.1%

    // calculate sqrt price limit based on slippage
    const sqrtPriceLimit = calculateSqrtPriceAfterSlippage(
      quoteResult.targetSqrtPrice,
      allowedSlippage,
      false
    )

    const swapResult = await invariant.swap(admin, poolKey, true, amount, true, sqrtPriceLimit)

    console.log(swapResult)

    await invariant.withdrawSingleToken(admin, poolKey.tokenY, null)

    // query state
    const pool: Pool = await invariant.getPool(token0Address, token1Address, feeTier)
    const position: Position = await invariant.getPosition(admin.addressRaw, 0n)
    const lowerTick: Tick = await invariant.getTick(poolKey, position.lowerTickIndex)
    const upperTickAfter: Tick = await invariant.getTick(poolKey, position.upperTickIndex)

    // check amount of tokens the owner is able to claim
    const fees = calculateFee(pool, position, lowerTick, upperTickAfter)

    // print amount of unclaimed x and y token
    console.log(fees)

    // get balance of a specific token before claiming position fees and print it
    const adminBalanceBeforeClaim = await tokenX.balanceOf(admin.addressRaw)
    console.log(adminBalanceBeforeClaim)

    // specify position id
    const positionId = 0n
    // claim fee
    const claimFeeResult = await invariant.claimFee(admin, positionId)
    console.log(claimFeeResult)

    const withdrawResult = await invariant.withdrawSingleToken(admin, poolKey.tokenX, fees[0])
    console.log(withdrawResult)

    // get balance of a specific token after claiming position fees and print it
    const adminBalanceAfterClaim = await tokenX.balanceOf(admin.addressRaw)
    console.log(adminBalanceAfterClaim)

    const receiver = await GearKeyring.fromSuri('//Bob')

    const positionToTransfer = await invariant.getPosition(admin.addressRaw, 0n)
    // Transfer position from admin (signer) to receiver
    await invariant.transferPosition(admin, 0n, receiver.addressRaw)
    const receiverPosition = await invariant.getPosition(receiver.addressRaw, 0n)
    assert.deepEqual(positionToTransfer, receiverPosition)
    console.log(receiverPosition)

    // ### retransfer the position back to the original admin
    await invariant.transferPosition(receiver, 0n, admin.addressRaw)
    // ###

    // fetch user balances before removal
    const adminToken0BalanceBeforeRemove = await tokenX.balanceOf(admin.addressRaw)
    const adminToken1BalanceBeforeRemove = await tokenY.balanceOf(admin.addressRaw)
    console.log(adminToken0BalanceBeforeRemove, adminToken1BalanceBeforeRemove)
    // remove position

    const removePositionResult = await invariant.removePosition(admin, positionId)
    console.log(removePositionResult)

    await invariant.withdrawTokenPair(admin, [poolKey.tokenX, null], [poolKey.tokenY, null])

    // get balance of a specific token after removing position
    const adminToken0BalanceAfterRemove = await tokenX.balanceOf(admin.addressRaw)
    const adminToken1BalanceAfterRemove = await tokenY.balanceOf(admin.addressRaw)

    // print balances
    console.log(adminToken0BalanceAfterRemove, adminToken1BalanceAfterRemove)
  })
  it('sdk guide - using grc20', async function () {
    this.timeout(80000)

    // deploy token, it will return token object
    const token0 = await FungibleToken.deploy(api, admin, 'CoinA', 'ACOIN', 12n)
    // token address can be accessed by calling programId method
    const token1Address = (
      await FungibleToken.deploy(api, admin, 'CoinB', 'BCOIN', 12n)
    ).programId()

    // load token by passing its address (you can use existing one), it allows you to interact with it
    // eslint disable-next-line
    const token1 = await FungibleToken.load(api, token1Address)

    // interact with token 0
    const admin0Balance = await token0.balanceOf(admin.addressRaw)
    console.log(admin0Balance)

    // if you want to interact with different token,
    // simply pass different contract address as an argument
    const admin1Balance = await token1.balanceOf(admin.addressRaw)
    console.log(admin1Balance)

    // fetch token metadata for previously deployed token0
    const token0Name = await token0.name()
    const token0Symbol = await token0.symbol()
    const token0Decimals = await token0.decimals()
    console.log(token0Name, token0Symbol, token0Decimals)

    // load diffrent token and load its metadata
    const token1Name = await token1.name()
    const token1Symbol = await token1.symbol()
    const token1Decimals = await token1.decimals()
    console.log(token1Name, token1Symbol, token1Decimals)
  })
  this.afterAll(async function () {
    unsub = subscribeToNewHeads(api)
  })
})
