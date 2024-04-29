import { GearApi, GearKeyring } from '@gear-js/api'
import { LOCAL } from './consts.js'
import { FungibleToken } from './fungilbe_token.js'
async function connect() {
  const gearApi = await GearApi.create({
    providerAddress: LOCAL
  })

  const [chain, nodeName, nodeVersion] = await Promise.all([
    gearApi.chain(),
    gearApi.nodeName(),
    gearApi.nodeVersion()
  ])

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`)

  await gearApi.blocks.subscribeNewHeads(header => {
    console.log(
      `New block with number: ${header.number.toNumber()} and hash: ${header.hash.toHex()}`
    )
  })

  const admin = await GearKeyring.fromSuri('//Alice')

  const token = await FungibleToken.deploy(gearApi, admin, 'Test Token', 'TT', 18n)
  console.log(token)
}

connect().catch(console.error)
