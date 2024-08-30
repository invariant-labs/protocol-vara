import {
  GearKeyring,
  Network,
  FungibleToken,
  initGearApi,
  subscribeToNewHeads
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const api = await initGearApi(Network.Testnet)
  await subscribeToNewHeads(api)
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)

  const BTC_ADDRESS = await FungibleToken.deploy(api, account, 'Bitcoin', 'BTC', 8n)
  const ETH_ADDRESS = await FungibleToken.deploy(api, account, 'Ether', 'ETH', 18n)
  const USDC_ADDRESS = await FungibleToken.deploy(api, account, 'USDC', 'USDC', 6n)

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)
  console.log(`BTC: ${BTC_ADDRESS}, ETH: ${ETH_ADDRESS}, USDC: ${USDC_ADDRESS}`)

  process.exit(0)
}

main()
