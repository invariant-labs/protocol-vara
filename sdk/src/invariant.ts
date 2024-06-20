import { GearApi, HexString } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import {
  ActorId,
  Signer,
  getWasm,
  integerSafeCast,
  unwrapResult,
  convertTick,
  convertPool,
  convertFeeTier,
  convertPosition,
  convertPoolKey,
  TransactionWrapper,
  convertCalculateSwapResult
} from './utils.js'
import { DEFAULT_ADDRESS, INVARIANT_GAS_LIMIT } from './consts.js'
import { InvariantContract } from './invariant-contract.js'
import {
  CalculateSwapResult,
  FeeTier,
  Liquidity,
  Position,
  Pool,
  Percentage,
  PoolKey,
  SqrtPrice,
  Tick,
  TokenAmount
} from 'invariant-vara-wasm'

export class Invariant {
  private constructor(
    private readonly contract: InvariantContract,
    private readonly gasLimit: bigint
  ) {}

  static async deploy(
    api: GearApi,
    deployer: KeyringPair,
    protocolFee: Percentage,
    gasLimit: bigint = INVARIANT_GAS_LIMIT
  ) {
    const code = await getWasm('invariant')
    const invariant = new InvariantContract(api)
    const deployTx = await invariant
      .newCtorFromCode(code, {
        admin: deployer.addressRaw,
        protocolFee
      } as any)
      .withAccount(deployer)
      .withGas(gasLimit)

    {
      const { response } = await deployTx.signAndSend()
      response()
    }

    return new Invariant(invariant, gasLimit)
  }

  static async load(api: GearApi, programId: HexString, gasLimit: bigint) {
    const invariant = new InvariantContract(api, programId)
    return new Invariant(invariant, gasLimit)
  }

  programId(): HexString {
    const id = this.contract.programId

    if (id === undefined || id === null) {
      throw new Error('Program id is not set')
    }

    return id
  }

  async feeTierExists(feeTier: FeeTier): Promise<boolean> {
    return this.contract.service.feeTierExists(feeTier as any, DEFAULT_ADDRESS)
  }

  async getFeeTiers(): Promise<FeeTier[]> {
    return ((await this.contract.service.getFeeTiers(DEFAULT_ADDRESS)) as any[]).map(convertFeeTier)
  }

  async getPool(token0: ActorId, token1: ActorId, feeTier: FeeTier): Promise<Pool> {
    return convertPool(
      unwrapResult(
        await this.contract.service.getPool(
          token0 as any,
          token1 as any,
          feeTier as any,
          DEFAULT_ADDRESS as any
        )
      )
    )
  }

  async getProtocolFee(): Promise<Percentage> {
    return BigInt((await this.contract.service.getProtocolFee(DEFAULT_ADDRESS)) as any)
  }

  async getTick(key: PoolKey, index: bigint): Promise<Tick> {
    return convertTick(
      unwrapResult(
        await this.contract.service.getTick(key as any, integerSafeCast(index), DEFAULT_ADDRESS)
      )
    )
  }

  async isTickInitialized(key: PoolKey, index: bigint): Promise<boolean> {
    return this.contract.service.isTickInitialized(
      key as any,
      integerSafeCast(index),
      DEFAULT_ADDRESS
    )
  }

  async getPools(size: bigint, offset: bigint): Promise<PoolKey[]> {
    return unwrapResult(
      await this.contract.service.getPools(size as any, offset as any, DEFAULT_ADDRESS)
    ).map(convertPoolKey)
  }

  async changeProtocolFeeTx(
    fee: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<Percentage>> {
    return new TransactionWrapper<Percentage>(
      await this.contract.service.changeProtocolFee(fee as any).withGas(gasLimit)
    )
  }

  async changeProtocolFee(
    signer: Signer,
    fee: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<Percentage> {
    const tx = (await this.changeProtocolFeeTx(fee, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async addFeeTierTx(
    feeTier: FeeTier,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<FeeTier>> {
    return new TransactionWrapper<FeeTier>(
      await this.contract.service.addFeeTier(feeTier as any).withGas(gasLimit)
    ).withDecode(convertFeeTier)
  }

  async addFeeTier(
    signer: Signer,
    feeTier: FeeTier,
    gasLimit: bigint = this.gasLimit
  ): Promise<FeeTier> {
    const tx = (await this.addFeeTierTx(feeTier as any, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async changeFeeReceiverTx(
    poolKey: PoolKey,
    feeReceiver: ActorId,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<null>> {
    return new TransactionWrapper<null>(
      await this.contract.service
        .changeFeeReceiver(poolKey as any, feeReceiver as any)
        .withGas(gasLimit)
    )
  }

  async changeFeeReceiver(
    signer: Signer,
    poolKey: PoolKey,
    feeReceiver: ActorId,
    gasLimit: bigint = this.gasLimit
  ): Promise<null> {
    const tx = (
      await this.changeFeeReceiverTx(poolKey as any, feeReceiver as any, gasLimit)
    ).withAccount(signer)
    return tx.send()
  }

  async createPoolTx(
    key: PoolKey,
    initSqrtPrice: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<null>> {
    return new TransactionWrapper<null>(
      await this.contract.service
        .createPool(key.tokenX, key.tokenY, key.feeTier as any, initSqrtPrice as any, 0n as any)
        .withGas(gasLimit)
    )
  }

  async createPool(
    signer: Signer,
    key: PoolKey,
    initSqrtPrice: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<null> {
    const tx = (await this.createPoolTx(key, initSqrtPrice, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async createPositionTx(
    key: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    slippageLimitLower: SqrtPrice,
    slippageLimitUpper: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<Position>> {
    return new TransactionWrapper<Position>(
      await this.contract.service
        .createPosition(
          key as any,
          lowerTick as any,
          upperTick as any,
          liquidityDelta as any,
          slippageLimitLower as any,
          slippageLimitUpper as any
        )
        .withGas(gasLimit)
    ).withDecode(convertPosition)
  }

  async createPosition(
    signer: Signer,
    key: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    slippageLimitLower: SqrtPrice,
    slippageLimitUpper: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<Position> {
    const tx = (
      await this.createPositionTx(
        key,
        lowerTick,
        upperTick,
        liquidityDelta,
        slippageLimitLower,
        slippageLimitUpper,
        gasLimit
      )
    ).withAccount(signer)
    return tx.send()
  }

  async depositTx(
    token: ActorId,
    amount: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<TokenAmount>> {
    return new TransactionWrapper(
      await this.contract.service.depositSingleToken(token as any, amount as any).withGas(gasLimit)
    )
  }

  async depositSingleToken(
    signer: Signer,
    token: ActorId,
    amount: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TokenAmount> {
    const tx = (await this.depositTx(token, amount, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async removeFeeTierTx(
    feeTier: FeeTier,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<FeeTier>> {
    return new TransactionWrapper<FeeTier>(
      await this.contract.service.removeFeeTier(feeTier as any).withGas(gasLimit)
    ).withDecode(convertFeeTier)
  }

  async removeFeeTier(
    signer: Signer,
    feeTier: FeeTier,
    gasLimit: bigint = this.gasLimit
  ): Promise<FeeTier> {
    const tx = (await this.removeFeeTierTx(feeTier as any, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async swapTx(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<CalculateSwapResult>> {
    return new TransactionWrapper<CalculateSwapResult>(
      await this.contract.service
        .swap(poolKey as any, xToY, amount as any, byAmountIn, sqrtPriceLimit as any)
        .withGas(gasLimit)
    ).withDecode(convertCalculateSwapResult)
  }

  async swap(
    signer: Signer,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<CalculateSwapResult> {
    const tx = (
      await this.swapTx(poolKey, xToY, amount, byAmountIn, sqrtPriceLimit, gasLimit)
    ).withAccount(signer)
    return tx.send()
  }

  async withdrawProtocolFeeTx(
    poolKey: PoolKey,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper(
      await this.contract.service.withdrawProtocolFee(poolKey as any).withGas(gasLimit)
    )
  }

  async withdrawProtocolFee(
    signer: Signer,
    poolKey: PoolKey,
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.withdrawProtocolFeeTx(poolKey, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async withdrawSingleTokenTx(
    token: ActorId,
    amount: TokenAmount | null = null,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<TokenAmount>> {
    return new TransactionWrapper(
      await this.contract.service.withdrawSingleToken(token as any, amount as any).withGas(gasLimit)
    )
  }

  async withdrawSingleToken(
    signer: Signer,
    token: ActorId,
    amount: TokenAmount | null = null,
    gasLimit: bigint = this.gasLimit
  ): Promise<TokenAmount> {
    const tx = (await this.withdrawSingleTokenTx(token, amount, gasLimit)).withAccount(signer)
    return tx.send()
  }
}
