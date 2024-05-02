import { GearApi, GearKeyring } from '@gear-js/api'
import { LOCAL } from './consts.js'
import { FungibleToken } from './fungible_token.js'
import { MetaDataTypes, Uint8ArrayToHexStr, UserMessageStatus, createTypeByName } from './utils.js'
import assert from 'assert'
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
    const res = await token.mint(admin, 100n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }
  {
    const res = await token.mint(user, 100n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }
  {
    const res = await token.approve(user, admin.addressRaw, 100n, 1n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }

  const allowance = await token.allowance(admin.addressRaw, user.addressRaw)
  assert.deepStrictEqual(allowance, createTypeByName(token.meta, MetaDataTypes.u128, 100n))
  const balance = await token.balanceOf(admin.addressRaw)
  assert.deepStrictEqual(balance, createTypeByName(token.meta, MetaDataTypes.u128, 100n))
  const totalSupply = await token.totalSupply()
  assert.deepStrictEqual(totalSupply, createTypeByName(token.meta, MetaDataTypes.u128, 200n))
  const decimals = await token.decimals()
  assert.deepStrictEqual(decimals, createTypeByName(token.meta, MetaDataTypes.u8, 18n))
  const validityTime = await token.getTxValidityTime(user.addressRaw, 1n)
  assert.notEqual(validityTime, null)

  const invalidTransaction = await token.getTxValidityTime(user.addressRaw, 0n)
  assert.strictEqual(invalidTransaction, null)
  {
    const res = await token.burn(admin, 300n)
    assert.strictEqual(res.status, UserMessageStatus.Panicked)
  }
  {
    const res = await token.mint(user, 300n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }
  {
    const res = await token.transfer(admin, user.addressRaw, admin.addressRaw, 300n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedWithError)
  }
  {
    const res = await token.approve(user, admin.addressRaw, 300n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }
  {
    const res = await token.transfer(admin, user.addressRaw, admin.addressRaw, 100n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
  }
}

connect().catch(console.error)
