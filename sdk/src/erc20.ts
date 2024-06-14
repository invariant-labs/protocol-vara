import { GearApi, HexString } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { Erc20Token } from './erc20-token.js'
import { ActorId, Signer, getWasm, integerSafeCast } from './utils.js'
import { FUNGIBLE_TOKEN_GAS_LIMIT, DEFAULT_ADDRESS } from './consts.js'
export type BalanceEntry = [Uint8Array, number]
export type AllowanceEntry = [Uint8Array, BalanceEntry]
export type FungibleTokenState = {
  name: string
  symbol: string
  totalSupply: number
  balances: Array<BalanceEntry>
  allowances: Array<AllowanceEntry>
  decimals: number
}

export class FungibleToken {
  private constructor(
    private readonly gasLimit: bigint,
    private readonly erc20: Erc20Token,
    private readonly admin?: KeyringPair
  ) {}

  static async deploy(
    api: GearApi,
    deployer: KeyringPair,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n,
    gasLimit: bigint = FUNGIBLE_TOKEN_GAS_LIMIT
  ) {
    const code = await getWasm('gear_erc20')
    const erc20 = new Erc20Token(api)
    const deployTx = await erc20
      .newCtorFromCode(code, name, symbol, integerSafeCast(decimals))
      .withAccount(deployer)
      .withGas(gasLimit)

    {
      const { response } = await deployTx.signAndSend()
      response()
    }
    const grantBurnerRoleTx = await erc20.admin
      .grantRole(deployer.addressRaw as any, 'burner')
      .withAccount(deployer)
      .withGas(gasLimit)
    {
      const { response } = await grantBurnerRoleTx.signAndSend()
      response()
    }

    const grantMinterRoleTx = await erc20.admin
      .grantRole(deployer.addressRaw as any, 'minter')
      .withAccount(deployer)
      .withGas(gasLimit)
    {
      const { response } = await grantMinterRoleTx.signAndSend()
      response()
    }
    return new FungibleToken(gasLimit, erc20, deployer)
  }

  static async load(api: GearApi, programId: HexString, gasLimit: bigint) {
    const erc20 = new Erc20Token(api, programId)
    return new FungibleToken(gasLimit, erc20)
  }

  async allowance(owner: ActorId, spender: ActorId): Promise<bigint> {
    return this.erc20.erc20.allowance(owner as any, spender as any, DEFAULT_ADDRESS)
  }

  async balanceOf(owner: ActorId): Promise<bigint> {
    return this.erc20.erc20.balanceOf(owner as any, DEFAULT_ADDRESS)
  }

  async decimals(): Promise<bigint> {
    return BigInt(await this.erc20.erc20.decimals(DEFAULT_ADDRESS))
  }

  async name(): Promise<string> {
    return this.erc20.erc20.name(DEFAULT_ADDRESS)
  }

  async symbol(): Promise<string> {
    return this.erc20.erc20.symbol(DEFAULT_ADDRESS)
  }

  async totalSupply(): Promise<bigint> {
    return this.erc20.erc20.totalSupply(DEFAULT_ADDRESS)
  }

  async approve(owner: Signer, spender: ActorId, amount: bigint): Promise<boolean> {
    const tx = await this.erc20.erc20
      .approve(spender as any, amount as any)
      .withAccount(owner)
      .withGas(this.gasLimit)
    const { response } = await tx.signAndSend()
    return response()
  }

  async burn(account: ActorId, amount: bigint) {
    if (!this.admin) {
      throw new Error('Admin account is required to burn tokens')
    }

    const tx = await this.erc20.admin
      .burn(account as any, amount as any)
      .withAccount(this.admin)
      .withGas(this.gasLimit)
    const { response } = await tx.signAndSend()
    return response()
  }

  async mint(account: ActorId, amount: bigint) {
    if (!this.admin) {
      throw new Error('Admin account is required to burn tokens')
    }

    const tx = await this.erc20.admin
      .mint(account as any, amount as any)
      .withAccount(this.admin)
      .withGas(this.gasLimit)
    const { response } = await tx.signAndSend()
    return response()
  }

  async transfer(signer: Signer, to: ActorId, amount: bigint) {
    const tx = await this.erc20.erc20
      .transfer(to as any, amount as any)
      .withAccount(signer)
      .withGas(this.gasLimit)
    const { response } = await tx.signAndSend()
    return response()
  }

  async transferFrom(signer: Signer, from: ActorId, to: ActorId, amount: bigint) {
    const tx = await this.erc20.erc20
      .transferFrom(from as any, to as any, amount as any)
      .withAccount(signer)
      .withGas(this.gasLimit)
    const { response } = await tx.signAndSend()
    return response()
  }
}
