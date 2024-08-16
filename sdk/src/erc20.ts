import { GearApi, HexString } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { Erc20Token } from './erc20-token.js'
import { ActorId, Signer, getWasm, integerSafeCast } from './utils.js'
import { FUNGIBLE_TOKEN_GAS_LIMIT, DEFAULT_ADDRESS } from './consts.js'
import { TransactionWrapper } from './utils.js'

export class FungibleToken {
  private constructor(
    private readonly gasLimit: bigint,
    private readonly erc20: Erc20Token,
    private admin?: KeyringPair
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
    if (!erc20.programId) {
      throw new Error('Failed to initialize FungibleToken program')
    }

    return erc20.programId
  }

  static async load(api: GearApi, gasLimit: bigint = FUNGIBLE_TOKEN_GAS_LIMIT) {
    const erc20 = new Erc20Token(api)
    return new FungibleToken(gasLimit, erc20)
  }

  programId(): HexString {
    const id = this.erc20.programId

    if (id === undefined || id === null) {
      throw new Error('Program id is not set')
    }

    return id
  }

  setAdmin(admin: KeyringPair) {
    this.admin = admin
  }

  async allowance(owner: ActorId, spender: ActorId, tokenAddress: HexString): Promise<bigint> {
    this.erc20.programId = tokenAddress

    return this.erc20.vft.allowance(owner as any, spender as any, DEFAULT_ADDRESS)
  }

  async balanceOf(owner: ActorId, tokenAddress: HexString): Promise<bigint> {
    this.erc20.programId = tokenAddress

    return this.erc20.vft.balanceOf(owner as any, DEFAULT_ADDRESS)
  }

  async decimals(tokenAddress: HexString): Promise<bigint> {
    this.erc20.programId = tokenAddress

    return BigInt(await this.erc20.vft.decimals(DEFAULT_ADDRESS))
  }

  async name(tokenAddress: HexString): Promise<string> {
    this.erc20.programId = tokenAddress

    return this.erc20.vft.name(DEFAULT_ADDRESS)
  }

  async symbol(tokenAddress: HexString): Promise<string> {
    this.erc20.programId = tokenAddress

    return this.erc20.vft.symbol(DEFAULT_ADDRESS)
  }

  async totalSupply(tokenAddress: HexString): Promise<bigint> {
    this.erc20.programId = tokenAddress

    return this.erc20.vft.totalSupply(DEFAULT_ADDRESS)
  }

  async approveTx(spender: ActorId, amount: bigint, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    return new TransactionWrapper<boolean>(
      await this.erc20.vft.approve(spender as any, amount as any).withGas(this.gasLimit)
    )
  }

  async approve(
    owner: Signer,
    spender: ActorId,
    amount: bigint,
    tokenAddress: HexString
  ): Promise<boolean> {
    const tx = await this.approveTx(spender, amount, tokenAddress)
    return tx.withAccount(owner).signAndSend()
  }

  async burnTx(account: ActorId, amount: bigint, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    return new TransactionWrapper<boolean>(
      await this.erc20.admin.burn(account as any, amount as any).withGas(this.gasLimit)
    )
  }

  async burn(account: ActorId, amount: bigint, tokenAddress: HexString) {
    if (!this.admin) {
      throw new Error('Admin account is required to burn tokens')
    }

    const tx = await this.burnTx(account, amount, tokenAddress)
    return tx.withAccount(this.admin).signAndSend()
  }

  async mintTx(account: ActorId, amount: bigint, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    return new TransactionWrapper<boolean>(
      await this.erc20.admin.mint(account as any, amount as any).withGas(this.gasLimit)
    )
  }

  async mint(account: ActorId, amount: bigint, tokenAddress: HexString) {
    if (!this.admin) {
      throw new Error('Admin account is required to mint tokens')
    }

    const tx = await this.mintTx(account, amount, tokenAddress)
    return tx.withAccount(this.admin).signAndSend()
  }

  async setTransferFail(flag: boolean, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    if (!this.admin) {
      throw new Error('Admin account is required to set transfer failure')
    }

    const tx = await this.erc20.vft.setFailTransfer(flag).withGas(this.gasLimit)
    const { response } = await tx.withAccount(this.admin).signAndSend()
    return response()
  }

  async transferTx(to: ActorId, amount: bigint, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    return new TransactionWrapper<boolean>(
      await this.erc20.vft.transfer(to as any, amount as any).withGas(this.gasLimit)
    )
  }

  async transfer(signer: Signer, to: ActorId, amount: bigint, tokenAddress: HexString) {
    const tx = await this.transferTx(to, amount, tokenAddress)
    return tx.withAccount(signer).signAndSend()
  }

  async transferFromTx(from: ActorId, to: ActorId, amount: bigint, tokenAddress: HexString) {
    this.erc20.programId = tokenAddress

    return new TransactionWrapper<boolean>(
      await this.erc20.vft
        .transferFrom(from as any, to as any, amount as any)
        .withGas(this.gasLimit)
    )
  }

  async transferFrom(
    signer: Signer,
    from: ActorId,
    to: ActorId,
    amount: bigint,
    tokenAddress: HexString
  ) {
    const tx = await this.transferFromTx(from, to, amount, tokenAddress)
    return tx.withAccount(signer).signAndSend()
  }
}
