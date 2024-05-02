import { GearApi, GearKeyring } from '@gear-js/api'
import { LOCAL } from './consts.js'
import { FungibleToken } from './fungible_token.js'
import {
  Uint8ArrayToHexStr,
  assertPanicked,
  assertProcessed as assertProcessedSuccessfully,
  assertProcessedWithError
} from './utils.js'
import { EventListener } from './event_listener.js'

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

  const admin = await GearKeyring.fromSuri('//Bob')
  const user = await GearKeyring.fromSuri('//Alice')

  const eventListener = new EventListener(gearApi)
  eventListener.listen()
  const token = await FungibleToken.deploy(gearApi, eventListener, admin, 'Test Token', 'TT', 18n)
  console.log('TokenId', token.programId)
  console.log('AdminId', Uint8ArrayToHexStr(admin.publicKey))

  {
    const res = await token.burn(admin, 100n)
    assertPanicked(res)
  }
  {
    const res = await token.mint(user, 100n)
    assertProcessedSuccessfully(res)
  }
  {
    const res = await token.transfer(admin, user.addressRaw, admin.addressRaw, 100n)
    assertProcessedWithError(res)
  }
  {
    const res = await token.approve(user, admin.addressRaw, 100n)
    assertProcessedSuccessfully(res)
  }
  {
    const res = await token.transfer(admin, user.addressRaw, admin.addressRaw, 100n)
    assertProcessedSuccessfully(res)
  }
}

connect().catch(console.error)
