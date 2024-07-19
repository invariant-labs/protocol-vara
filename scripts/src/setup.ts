import {
  FEE_TIERS,
  Invariant,
  GearKeyring,
  Network,
  FungibleToken,
  PoolKey,
  calculateTick,
  initGearApi,
  newPoolKey,
  priceToSqrtPrice,
  toPercentage,
  subscribeToNewHeads,
  calculateSqrtPrice
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initGearApi({ providerAddress: network })
  await subscribeToNewHeads(api)

  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)
  console.log(`Deployer: ${account.address}, Mnemonic: ${mnemonic}`)

  const invariant = await Invariant.deploy(api, account, toPercentage(1n, 2n))
  console.log(`Invariant: ${invariant.programId()}`)

  for (const feeTier of FEE_TIERS) {
    await invariant.addFeeTier(account, feeTier)
  }
  console.log('Successfully added fee tiers')

  const BTCAddress = await FungibleToken.deploy(api, account, 'Bitcoin', 'BTC', 8n)
  const ETHAddress = await FungibleToken.deploy(api, account, 'Ether', 'ETH', 12n)
  const USDCAddress = await FungibleToken.deploy(api, account, 'USDC', 'USDC', 6n)
  const decimals = {
    [BTCAddress]: 8n,
    [ETHAddress]: 12n,
    [USDCAddress]: 6n
  }
  console.log(`BTC: ${BTCAddress}, ETH: ${ETHAddress}, USDC: ${USDCAddress}`)

  const response = await fetch(
    'https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=bitcoin,ethereum,aleph-zero'
  )
  const data = await response.json()
  const prices = {
    [BTCAddress]: data.find((coin: any) => coin.id === 'bitcoin').current_price,
    [ETHAddress]: data.find((coin: any) => coin.id === 'ethereum').current_price,
    [USDCAddress]: 1
  }
  console.log(
    `BTC: ${prices[BTCAddress]}, ETH: ${prices[ETHAddress]}, USDC: ${prices[USDCAddress]}`
  )

  const poolKeys: [PoolKey, bigint][] = [
    [newPoolKey(BTCAddress, ETHAddress, FEE_TIERS[1]), 130559235944405760n],
    [newPoolKey(BTCAddress, USDCAddress, FEE_TIERS[1]), 7865049221247086n],
    [newPoolKey(ETHAddress, USDCAddress, FEE_TIERS[1]), 3454809855596621497n]
  ]
  for (const [poolKey] of poolKeys) {
    try {
      const poolSqrtPrice = getTicksAndSqrtPriceFromPrice(decimals, prices, poolKey).poolSqrtPrice
      await invariant.createPool(account, poolKey, poolSqrtPrice)
    } catch (e) {
      console.log('Create pool error', poolKey, e)
    }
  }
  console.log('Successfully added pools')

  const grc20 = await FungibleToken.load(api)
  grc20.setAdmin(account)
  await grc20.mint(account.addressRaw, 2n ** 96n - 1n, BTCAddress)
  await grc20.mint(account.addressRaw, 2n ** 96n - 1n, ETHAddress)
  await grc20.mint(account.addressRaw, 2n ** 96n - 1n, USDCAddress)
  await grc20.approve(account, invariant.programId(), 2n ** 96n - 1n, BTCAddress)
  await grc20.approve(account, invariant.programId(), 2n ** 96n - 1n, ETHAddress)
  await grc20.approve(account, invariant.programId(), 2n ** 96n - 1n, USDCAddress)

  const BTCBefore = await grc20.balanceOf(account.addressRaw, BTCAddress)
  const ETHBefore = await grc20.balanceOf(account.addressRaw, ETHAddress)
  const USDCBefore = await grc20.balanceOf(account.addressRaw, USDCAddress)
  for (const [poolKey, amount] of poolKeys) {
    try {
      const { lowerTick, upperTick, poolSqrtPrice } = getTicksAndSqrtPriceFromPrice(
        decimals,
        prices,
        poolKey
      )

      await invariant.depositTokenPair(account, [poolKey.tokenX, amount], [poolKey.tokenY, amount])
      await invariant.createPosition(
        account,
        poolKey,
        lowerTick,
        upperTick,
        amount,
        poolSqrtPrice,
        0n
      )
    } catch (e) {
      console.log('Create position error', poolKey, e)
    }
  }
  const BTCAfter = await grc20.balanceOf(account.addressRaw, BTCAddress)
  const ETHAfter = await grc20.balanceOf(account.addressRaw, ETHAddress)
  const USDCAfter = await grc20.balanceOf(account.addressRaw, USDCAddress)

  console.log(
    `BTC: ${BTCBefore - BTCAfter}, ETH: ${ETHBefore - ETHAfter}, USDC: ${USDCBefore - USDCAfter}`
  )
  console.log('Successfully created positions')

  process.exit(0)
}
const getTicksAndSqrtPriceFromPrice = (
  decimals: { [key: string]: bigint },
  prices: { [key: string]: number },
  poolKey: PoolKey
) => {
  const price =
    (1 / (prices[poolKey.tokenY] / prices[poolKey.tokenX])) *
    10 ** (Number(decimals[poolKey.tokenY]) - Number(decimals[poolKey.tokenX])) *
    10 ** 24
  const lowerSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price * 0.95)))
  const upperSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price * 1.05)))
  const poolSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price)))
  const lowerTick = calculateTick(lowerSqrtPrice, FEE_TIERS[1].tickSpacing)
  const upperTick = calculateTick(upperSqrtPrice, FEE_TIERS[1].tickSpacing)
  const poolTick = calculateTick(poolSqrtPrice, FEE_TIERS[1].tickSpacing)

  const tickAdjustedSqrtPrice = calculateSqrtPrice(poolTick)
  return { lowerTick, upperTick, poolSqrtPrice: tickAdjustedSqrtPrice }
}
main()
