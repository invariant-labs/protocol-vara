import 'mocha'
import {
  calculateFee,
  calculateSqrtPriceAfterSlippage,
  initGearApi,
  newFeeTier,
  newPoolKey,
  priceToSqrtPrice,
  subscribeToNewHeads,
  getLiquidityByY,
  toPercentage,
  toPrice
} from '../src/utils.js'
import { GearKeyring, HexString } from '@gear-js/api'
import { Network } from '../src/network'
import { Invariant } from '../src/invariant'
import { assert } from 'chai'
import { FungibleToken } from '../src/erc20.js'
import { Pool, Tick, Position } from '../src/schema'
import { sortTokens } from '../src/test-utils.js'

const api = await initGearApi(Network.Local)
const admin = await GearKeyring.fromSuri('//Alice')
const GRC20 = await FungibleToken.load(api)
GRC20.setAdmin(admin)
let token0Address: HexString = null as any
let token1Address: HexString = null as any

const INVARIANT_ADDRESS = (await Invariant.deploy(api, admin, 0n)).programId()
let unsub: Promise<VoidFunction> = null as any
describe('sdk guide snippets', async function () {
  this.timeout(80000)
  this.beforeEach(async function () {
    token0Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    token1Address = await FungibleToken.deploy(api, admin, 'Coin', 'COIN', 12n)
    ;[token0Address, token1Address] = sortTokens(token0Address, token1Address)
    await GRC20.mint(admin.addressRaw, 1000000000000000000000000000000n, token0Address)
    await GRC20.mint(admin.addressRaw, 1000000000000000000000000000000n, token1Address)
  })
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

    // approve transfers of both tokens
    await GRC20.approve(admin, invariant.programId(), tokenXAmount, poolKey.tokenX)
    await GRC20.approve(admin, invariant.programId(), tokenYAmount, poolKey.tokenY)

    // deposit tokens in the contract
    await invariant.depositTokenPair(
      admin,
      [poolKey.tokenX, tokenXAmount],
      [poolKey.tokenY, tokenYAmount]
    )

    // check user balances
    const userBalances = await invariant.getUserBalances(admin.addressRaw)
    console.log(userBalances)

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
    await GRC20.approve(admin, invariant.programId(), amount, poolKey.tokenX)
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
    const adminBalanceBeforeClaim = await GRC20.balanceOf(admin.addressRaw, token0Address)
    console.log(adminBalanceBeforeClaim)

    // specify position id
    const positionId = 0n
    // claim fee
    const claimFeeResult = await invariant.claimFee(admin, positionId)
    console.log(claimFeeResult)

    const withdrawResult = await invariant.withdrawSingleToken(admin, poolKey.tokenX, fees[0])
    console.log(withdrawResult)

    // get balance of a specific token after claiming position fees and print it
    const adminBalanceAfterClaim = await GRC20.balanceOf(admin.addressRaw, token0Address)
    console.log(adminBalanceAfterClaim)

    const receiver = await GearKeyring.fromSuri('//Bob')

    const positionToTransfer = await invariant.getPosition(admin.addressRaw, 0n)
    // Transfer position from admin (signer) to receiver
    await invariant.transferPosition(admin, 0n, receiver.addressRaw)
    // load received position
    const receiverPosition = await invariant.getPosition(receiver.addressRaw, 0n)

    // ensure that the position are equal
    assert.deepEqual(positionToTransfer, receiverPosition)
    console.log(receiverPosition)

    // ### retransfer the position back to the original admin
    await invariant.transferPosition(receiver, 0n, admin.addressRaw)
    // ###

    // fetch user balances before removal
    const adminToken0BalanceBeforeRemove = await GRC20.balanceOf(admin.addressRaw, token0Address)
    const adminToken1BalanceBeforeRemove = await GRC20.balanceOf(admin.addressRaw, token1Address)
    console.log(adminToken0BalanceBeforeRemove, adminToken1BalanceBeforeRemove)
    // remove position

    const removePositionResult = await invariant.removePosition(admin, positionId)
    console.log(removePositionResult)

    await invariant.withdrawTokenPair(admin, [poolKey.tokenX, null], [poolKey.tokenY, null])

    // get balance of a specific token after removing position
    const adminToken0BalanceAfterRemove = await GRC20.balanceOf(admin.addressRaw, token0Address)
    const adminToken1BalanceAfterRemove = await GRC20.balanceOf(admin.addressRaw, token1Address)

    // print balances
    console.log(adminToken0BalanceAfterRemove, adminToken1BalanceAfterRemove)
  })
  it('sdk guide - using grc20', async function () {
    this.timeout(80000)

    // deploy token, it will return token address
    const token0Address = await FungibleToken.deploy(api, admin, 'CoinA', 'ACOIN', 12n)
    const token1Address = await FungibleToken.deploy(api, admin, 'CoinB', 'BCOIN', 12n)

    // loading token class, allows you to interact with token contracts
    const GRC20 = await FungibleToken.load(api)
    // set admin account if you want to mint or burn tokens
    // by default admin is set to the deployer of the contract
    GRC20.setAdmin(admin)

    // interact with token 0
    const admin0Balance = await GRC20.balanceOf(admin.addressRaw, token0Address)
    console.log(admin0Balance)

    // if you want to interact with different token,
    // simply pass different contract address as an argument
    const admin1Balance = await GRC20.balanceOf(admin.addressRaw, token1Address)
    console.log(admin1Balance)

    // fetch token metadata for previously deployed token0
    const token0Name = await GRC20.name(token0Address)
    const token0Symbol = await GRC20.symbol(token0Address)
    const token0Decimals = await GRC20.decimals(token0Address)
    console.log(token0Name, token0Symbol, token0Decimals)

    // load diffrent token and load its metadata
    const token1Name = await GRC20.name(token1Address)
    const token1Symbol = await GRC20.symbol(token1Address)
    const token1Decimals = await GRC20.decimals(token1Address)
    console.log(token1Name, token1Symbol, token1Decimals)
  })
  this.afterAll(async function () {
    await unsub.then(unsub => unsub())
  })
})
