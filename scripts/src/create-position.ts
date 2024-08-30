import {
  HexString,
  Invariant,
  GearKeyring,
  Network,
  FungibleToken,
  TESTNET_ETH_ADDRESS,
  TESTNET_INVARIANT_ADDRESS,
  TESTNET_USDC_ADDRESS,
  initGearApi,
  newFeeTier,
  newPoolKey,
  toPercentage
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initGearApi(network)

  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const receiver = (process.env.RECEIVER_ADDRESS as HexString) ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)

  const FEE_TIER = newFeeTier(toPercentage(1n, 4n), 1n)
  const TOKEN_0_ADDRESS = TESTNET_USDC_ADDRESS
  const TOKEN_1_ADDRESS = TESTNET_ETH_ADDRESS
  const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)
  const AMOUNT = 1000000000000000000n

  const invariant = await Invariant.load(api, TESTNET_INVARIANT_ADDRESS)
  const grc20 = await FungibleToken.load(api)
  grc20.setAdmin(account)

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await grc20.mint(account.addressRaw, AMOUNT, TOKEN_0_ADDRESS)
  await grc20.approve(account, TESTNET_INVARIANT_ADDRESS, AMOUNT, TOKEN_0_ADDRESS)

  await grc20.mint(account.addressRaw, AMOUNT, TOKEN_1_ADDRESS)
  await grc20.approve(account, TESTNET_INVARIANT_ADDRESS, AMOUNT, TOKEN_1_ADDRESS)

  await invariant.depositTokenPair(account, [POOL_KEY.tokenX, AMOUNT], [POOL_KEY.tokenY, AMOUNT])
  await invariant.createPosition(
    account,
    POOL_KEY,
    -10n,
    10n,
    AMOUNT,
    1000000000000000000000000n,
    10000000000n
  )
  await invariant.withdrawTokenPair(account, [POOL_KEY.tokenX, null], [POOL_KEY.tokenY, null])

  if (receiver) {
    await invariant.transferPosition(account, 0n, receiver)
  }

  process.exit(0)
}

main()
