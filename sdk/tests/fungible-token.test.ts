import 'mocha'
import { assert, expect } from 'chai'
import { GearKeyring } from '@gear-js/api'
import { initGearApi, subscribeToNewHeads } from '../src/utils.js'
import { Network } from '../src/network'
import { FungibleToken } from '../src/erc20.js'
import { assertThrowsAsync } from '../src/test-utils.js'

const api = await initGearApi(Network.Testnet)
const account0 = await GearKeyring.fromSuri('//Alice')
const account1 = await GearKeyring.fromSuri('//Bob')
let unsub: Promise<VoidFunction> | null = null
const GRC20: FungibleToken = await FungibleToken.load(api)
GRC20.setAdmin(account0)
let tokenAddress = null as any
describe('FungibleToken', function () {
  before(function () {
    unsub = subscribeToNewHeads(api)
  })
  this.timeout(200000)

  beforeEach(async function () {
    tokenAddress = await FungibleToken.deploy(api, account0, 'Coin', 'COIN', 12n)
  })
  it('mint and burn', async function () {
    const resMint = await GRC20.mint(account0.addressRaw, 100n, tokenAddress)
    assert.strictEqual(resMint, true)
    await GRC20.balanceOf(account0.addressRaw, tokenAddress).then(balance => {
      assert.deepStrictEqual(balance, 100n)
    })
    const resBurn = await GRC20.burn(account0.addressRaw, 100n, tokenAddress)
    assert.strictEqual(resBurn, true)
    await GRC20.balanceOf(account0.addressRaw, tokenAddress).then(balance => {
      assert.deepStrictEqual(balance, 0n)
    })
  })
  it('mint and burn tx', async function () {
    {
      const response = await (await GRC20.mintTx(account0.addressRaw, 100n, tokenAddress))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(response, true)
    }

    await GRC20.balanceOf(account0.addressRaw, tokenAddress).then(balance => {
      assert.deepStrictEqual(balance, 100n)
    })
    {
      const response = await (await GRC20.burnTx(account0.addressRaw, 100n, tokenAddress))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    await GRC20.balanceOf(account0.addressRaw, tokenAddress).then(balance => {
      assert.deepStrictEqual(balance, 0n)
    })
  })
  it('approve and transfer', async () => {
    {
      const res = await GRC20.mint(account1.addressRaw, 100n, tokenAddress)
      assert.strictEqual(res, true)
    }
    {
      const res = await GRC20.approve(account1, account0.addressRaw, 100n, tokenAddress)
      assert.strictEqual(res, true)
    }
    {
      const allowance = GRC20.allowance(account1.addressRaw, account0.addressRaw, tokenAddress)
      const balance0 = GRC20.balanceOf(account0.addressRaw, tokenAddress)
      const balance1 = GRC20.balanceOf(account1.addressRaw, tokenAddress)
      const totalSupply = GRC20.totalSupply(tokenAddress)
      const decimals = GRC20.decimals(tokenAddress)
      assert.deepStrictEqual(await allowance, 100n, 'allowance mismatch')
      assert.deepStrictEqual(await balance0, 0n, 'balance0 mismatch')
      assert.deepStrictEqual(await balance1, 100n, 'balance1 mismatch')
      assert.deepStrictEqual(await totalSupply, 100n, 'totalSupply mismatch')
      assert.deepStrictEqual(await decimals, 12n, 'decimals mismatch')
    }
    {
      const res = await GRC20.mint(account1.addressRaw, 300n, tokenAddress)
      assert.strictEqual(res, true)
    }
    await assertThrowsAsync(
      GRC20.transferFrom(account0, account1.addressRaw, account0.addressRaw, 300n, tokenAddress),
      'Error: Panic occurred: InsufficientAllowance'
    )
    {
      const res = await GRC20.approve(account1, account0.addressRaw, 300n, tokenAddress)
      expect(res).to.equal(true)
    }
    {
      const res = await GRC20.transferFrom(
        account0,
        account1.addressRaw,
        account0.addressRaw,
        300n,
        tokenAddress
      )
      expect(res).to.equal(true)
    }
    {
      const allowance = GRC20.allowance(account1.addressRaw, account0.addressRaw, tokenAddress)
      const balance0 = GRC20.balanceOf(account0.addressRaw, tokenAddress)
      const balance1 = GRC20.balanceOf(account1.addressRaw, tokenAddress)
      const totalSupply = GRC20.totalSupply(tokenAddress)
      expect(await allowance).to.equal(0n)
      expect(await balance0).to.equal(300n)
      expect(await balance1).to.equal(100n)
      expect(await totalSupply).to.equal(400n)
    }
  })
  it('approve and transfer tx', async () => {
    {
      const response = await (await GRC20.mintTx(account1.addressRaw, 100n, tokenAddress))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    {
      const response = await (await GRC20.approveTx(account0.addressRaw, 100n, tokenAddress))
        .withAccount(account1)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    {
      const allowance = GRC20.allowance(account1.addressRaw, account0.addressRaw, tokenAddress)
      const balance0 = GRC20.balanceOf(account0.addressRaw, tokenAddress)
      const balance1 = GRC20.balanceOf(account1.addressRaw, tokenAddress)
      const totalSupply = GRC20.totalSupply(tokenAddress)
      const decimals = GRC20.decimals(tokenAddress)
      assert.deepStrictEqual(await allowance, 100n, 'allowance mismatch')
      assert.deepStrictEqual(await balance0, 0n, 'balance0 mismatch')
      assert.deepStrictEqual(await balance1, 100n, 'balance1 mismatch')
      assert.deepStrictEqual(await totalSupply, 100n, 'totalSupply mismatch')
      assert.deepStrictEqual(await decimals, 12n, 'decimals mismatch')
    }
    {
      const response = await (await GRC20.mintTx(account1.addressRaw, 300n, tokenAddress))
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    await assertThrowsAsync(
      (async () => {
        const response = await (
          await GRC20.transferFromTx(account1.addressRaw, account0.addressRaw, 300n, tokenAddress)
        )
          .withAccount(account0)
          .signAndSend()
        assert.strictEqual(response, true)
      })(),
      'Error: Panic occurred: InsufficientAllowance'
    )
    {
      const response = await (await GRC20.approveTx(account0.addressRaw, 300n, tokenAddress))
        .withAccount(account1)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    {
      const response = await (
        await GRC20.transferFromTx(account1.addressRaw, account0.addressRaw, 300n, tokenAddress)
      )
        .withAccount(account0)
        .signAndSend()
      assert.strictEqual(response, true)
    }
    {
      const allowance = GRC20.allowance(account1.addressRaw, account0.addressRaw, tokenAddress)
      const balance0 = GRC20.balanceOf(account0.addressRaw, tokenAddress)
      const balance1 = GRC20.balanceOf(account1.addressRaw, tokenAddress)
      const totalSupply = GRC20.totalSupply(tokenAddress)
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
