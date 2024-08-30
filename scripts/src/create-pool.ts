import {
  Invariant,
  GearKeyring,
  Network,
  TESTNET_BTC_ADDRESS,
  TESTNET_ETH_ADDRESS,
  TESTNET_INVARIANT_ADDRESS,
  initGearApi,
  newFeeTier,
  newPoolKey,
  toPercentage,
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initGearApi(network)

  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)

  const FEE_TIER = newFeeTier(toPercentage(1n, 4n), 1n)
  const TOKEN_0_ADDRESS = TESTNET_ETH_ADDRESS
  const TOKEN_1_ADDRESS = TESTNET_BTC_ADDRESS
  const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)

  const invariant = await Invariant.load(api, TESTNET_INVARIANT_ADDRESS)

  await invariant.createPool(account, POOL_KEY, 1000000000000000000000000n)

  process.exit(0)
}

main()
