import { assert, expect } from 'chai'
import { GearKeyring } from '@gear-js/api'
import { EventListener } from '../src/event-listener.js'
import { UserMessageStatus, initGearApi, subscribeToNewHeads } from '../src/utils.js'
import { LOCAL } from '../src/consts.js'
import { FungibleToken } from '../src/fungible-token.js'

const api = await initGearApi({ providerAddress: LOCAL })
const account0 = await GearKeyring.fromSuri('//Alice')
const account1 = await GearKeyring.fromSuri('//Bob')
let unsub: Promise<VoidFunction> | null = null
const eventListener = new EventListener(api)
eventListener.listen()
let token = await FungibleToken.deploy(api, eventListener, account0, 'Coin', 'COIN', 12n)
describe('FungibleToken', function () {
  before(function () {
    unsub = subscribeToNewHeads(api)
  })
  this.timeout(200000)

  beforeEach(async function () {
    token = await FungibleToken.deploy(api, eventListener, account0, 'Coin', 'COIN', 12n)
  })
  it('mint and burn', async function () {
    const resMint = await token.mint(account0, 100n)
    assert.strictEqual(resMint.status, UserMessageStatus.ProcessedSuccessfully)
    await token.balanceOf(account0.addressRaw)

    const resBurn = await token.burn(account0, 100n)
    assert.strictEqual(resBurn.status, UserMessageStatus.ProcessedSuccessfully)
    await token.balanceOf(account0.addressRaw).then(balance => {
      assert.deepStrictEqual(balance, 0n)
    })
  })
  it('valid transaction timestamp', async function () {
    const currentTime = Date.now()
    const res = await token.approve(account1, account0.addressRaw, 100n, 1n)
    assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)

    const validityTime = await token.getTxValidityTime(account1.addressRaw, 1n)
    assert.notStrictEqual(validityTime, null)

    if (validityTime! < currentTime) {
      throw new Error('Timestamp is invalid')
    }
  })

  it('invalid transaction timestamp', async function () {
    const invalidTransaction = await token.getTxValidityTime(account1.addressRaw, 0n)
    assert.strictEqual(invalidTransaction, null)
  })
  it('approve and transfer', async () => {
    {
      const res = await token.mint(account1, 100n)
      assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
    }
    {
      const res = await token.approve(account1, account0.addressRaw, 100n, 1n)
      assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
    }
    {
      const allowance = token.allowance(account0.addressRaw, account1.addressRaw)
      const balance0 = token.balanceOf(account0.addressRaw)
      const balance1 = token.balanceOf(account1.addressRaw)
      const totalSupply = token.totalSupply()
      const decimals = token.decimals()
      assert.deepStrictEqual(await allowance, 100n)
      assert.deepStrictEqual(await balance0, 0n)
      assert.deepStrictEqual(await balance1, 100n)
      assert.deepStrictEqual(await totalSupply, 100n)
      assert.deepStrictEqual(await decimals, 12n)
    }

    {
      const res = await token.burn(account0, 300n)
      assert.strictEqual(res.status, UserMessageStatus.Panicked)
    }
    {
      const res = await token.mint(account1, 300n)
      assert.strictEqual(res.status, UserMessageStatus.ProcessedSuccessfully)
    }
    {
      const res = await token.transfer(account0, account1.addressRaw, account0.addressRaw, 300n)
      expect(res.status).to.equal(UserMessageStatus.ProcessedWithError)
      expect(res.data).to.deep.equal({ err: 'NotAllowedToTransfer' })
    }
    {
      const res = await token.approve(account1, account0.addressRaw, 300n)
      expect(res.status).to.equal(UserMessageStatus.ProcessedSuccessfully)
    }
    {
      const res = await token.transfer(account0, account1.addressRaw, account0.addressRaw, 300n)
      expect(res.status).to.equal(UserMessageStatus.ProcessedSuccessfully)
    }
    {
      const allowance = token.allowance(account0.addressRaw, account1.addressRaw)
      const balance0 = token.balanceOf(account0.addressRaw)
      const balance1 = token.balanceOf(account1.addressRaw)
      const totalSupply = token.totalSupply()
      expect(await allowance).to.equal(0n)
      expect(await balance0).to.equal(300n)
      expect(await balance1).to.equal(100n)
      expect(await totalSupply).to.equal(400n)
    }
  })
  after(async function () {
    await unsub!.then(unsub => unsub())
  })
})
