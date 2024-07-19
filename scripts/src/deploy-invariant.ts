import {
  FeeTier,
  Invariant,
  GearKeyring,
  Network,
  TESTNET_ETH_ADDRESS,
  TESTNET_USDC_ADDRESS,
  initGearApi,
  newFeeTier,
  toPercentage,
  newPoolKey
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initGearApi({ providerAddress: network })

  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = await GearKeyring.fromMnemonic(mnemonic)

  const INVARIANT = await Invariant.deploy(api, account, toPercentage(1n, 2n))

  const hundredthOfPercentage = toPercentage(1n, 4n)
  const generateFee = (tickCount: bigint): FeeTier => {
    return newFeeTier(tickCount * hundredthOfPercentage, tickCount)
  }

  console.log(`Invariant: ${INVARIANT.programId()}`)
  console.log(`Deployer: ${account.address}, Mnemonic: ${mnemonic}`)

  const feeTiers = [
    generateFee(1n),
    generateFee(2n),
    generateFee(5n),
    generateFee(10n),
    generateFee(30n),
    generateFee(100n)
  ]

  for (const feeTier of feeTiers) {
    await INVARIANT.addFeeTier(account, feeTier).catch(err => {
      console.error(err), process.exit(1)
    })
    console.log(`Fee tier added: ${feeTier.fee}, ${feeTier.tickSpacing}`)
  }

  await INVARIANT.createPool(
    account,
    newPoolKey(TESTNET_USDC_ADDRESS, TESTNET_ETH_ADDRESS, feeTiers[0]),
    1000000000000000000000000n
  ).catch(err => {
    console.error(err), process.exit(1)
  })

  console.log(`Pool added ${TESTNET_USDC_ADDRESS}, ${TESTNET_ETH_ADDRESS}`)
  process.exit(0)
}

main()
