import { assert, expect } from 'chai'
import { GearKeyring } from '@gear-js/api'
import { initGearApi, subscribeToNewHeads } from '../src/utils.js'
import { Network } from '../src/consts.js'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync } from '../src/test-utils.js'

const api = await initGearApi({ providerAddress: Network.Local })
const account0 = await GearKeyring.fromSuri('//Alice')
const account1 = await GearKeyring.fromSuri('//Bob')
let unsub: Promise<VoidFunction> | null = null
let token: FungibleToken = null as any
describe('FungibleToken', function () {
  before(function () {
    unsub = subscribeToNewHeads(api)
  })
  this.timeout(200000)

  beforeEach(async function () {
    token = await FungibleToken.deploy(api, account0, 'Coin', 'COIN', 12n)
  })
  it('mint and burn', async function () {
    const resMint = await token.mint(account0.addressRaw, 100n)
    assert.strictEqual(resMint, true)
    await token.balanceOf(account0.addressRaw).then(balance => {
      assert.deepStrictEqual(balance, 100n)
    })
    const resBurn = await token.burn(account0.addressRaw, 100n)
    assert.strictEqual(resBurn, true)
    await token.balanceOf(account0.addressRaw).then(balance => {
      assert.deepStrictEqual(balance, 0n)
    })
  })
  it('mint and burn tx', async function () {
    {
      const { response } = await (await token.mintTx(account0.addressRaw, 100n))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }

    await token.balanceOf(account0.addressRaw).then(balance => {
      assert.deepStrictEqual(balance, 100n)
    })
    {
      const { response } = await (await token.burnTx(account0.addressRaw, 100n))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    await token.balanceOf(account0.addressRaw).then(balance => {
      assert.deepStrictEqual(balance, 0n)
    })
  })
  it('approve and transfer', async () => {
    {
      const res = await token.mint(account1.addressRaw, 100n)
      assert.strictEqual(res, true)
    }
    {
      const res = await token.approve(account1, account0.addressRaw, 100n)
      assert.strictEqual(res, true)
    }
    {
      const allowance = token.allowance(account1.addressRaw, account0.addressRaw)
      const balance0 = token.balanceOf(account0.addressRaw)
      const balance1 = token.balanceOf(account1.addressRaw)
      const totalSupply = token.totalSupply()
      const decimals = token.decimals()
      assert.deepStrictEqual(await allowance, 100n, 'allowance mismatch')
      assert.deepStrictEqual(await balance0, 0n, 'balance0 mismatch')
      assert.deepStrictEqual(await balance1, 100n, 'balance1 mismatch')
      assert.deepStrictEqual(await totalSupply, 100n, 'totalSupply mismatch')
      assert.deepStrictEqual(await decimals, 12n, 'decimals mismatch')
    }
    {
      const res = await token.mint(account1.addressRaw, 300n)
      assert.strictEqual(res, true)
    }
    await assertThrowsAsync(
      token.transferFrom(account0, account1.addressRaw, account0.addressRaw, 300n),
      'Error: Panic occurred: InsufficientAllowance'
    )
    {
      const res = await token.approve(account1, account0.addressRaw, 300n)
      expect(res).to.equal(true)
    }
    {
      const res = await token.transferFrom(account0, account1.addressRaw, account0.addressRaw, 300n)
      expect(res).to.equal(true)
    }
    {
      const allowance = token.allowance(account1.addressRaw, account0.addressRaw)
      const balance0 = token.balanceOf(account0.addressRaw)
      const balance1 = token.balanceOf(account1.addressRaw)
      const totalSupply = token.totalSupply()
      expect(await allowance).to.equal(0n)
      expect(await balance0).to.equal(300n)
      expect(await balance1).to.equal(100n)
      expect(await totalSupply).to.equal(400n)
    }
  })
  it('approve and transfer tx', async () => {
    {
      const { response } = await (await token.mintTx(account1.addressRaw, 100n))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    {
      const { response } = await (await token.approveTx(account0.addressRaw, 100n))
        .withAccount(account1)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    {
      const allowance = token.allowance(account1.addressRaw, account0.addressRaw)
      const balance0 = token.balanceOf(account0.addressRaw)
      const balance1 = token.balanceOf(account1.addressRaw)
      const totalSupply = token.totalSupply()
      const decimals = token.decimals()
      assert.deepStrictEqual(await allowance, 100n, 'allowance mismatch')
      assert.deepStrictEqual(await balance0, 0n, 'balance0 mismatch')
      assert.deepStrictEqual(await balance1, 100n, 'balance1 mismatch')
      assert.deepStrictEqual(await totalSupply, 100n, 'totalSupply mismatch')
      assert.deepStrictEqual(await decimals, 12n, 'decimals mismatch')
    }
    {
      const { response } = await (await token.mintTx(account1.addressRaw, 300n))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    await assertThrowsAsync(
      (async () => {
        const { response } = await (
          await token.transferFromTx(account1.addressRaw, account0.addressRaw, 300n)
        )
          .withAccount(account0)
          .signAndSend()
        assert.strictEqual(await response(), true)
      })(),
      'Error: Panic occurred: InsufficientAllowance'
    )
    {
      const { response } = await (await token.approveTx(account0.addressRaw, 300n))
        .withAccount(account1)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    {
      const { response } = await (
        await token.transferFromTx(account1.addressRaw, account0.addressRaw, 300n)
      )
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(await response(), true)
    }
    {
      const allowance = token.allowance(account1.addressRaw, account0.addressRaw)
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
