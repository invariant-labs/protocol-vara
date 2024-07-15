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
  convertCalculateSwapResult,
  InvariantEventCallback,
  decodeEvent,
  calculateSqrtPriceAfterSlippage,
  convertQuoteResult,
  getMaxSqrtPrice,
  getMinSqrtPrice,
  calculateTick,
  convertLiquidityTick,
  positionToTick
} from './utils.js'
import { CHUNK_SIZE, DEFAULT_ADDRESS, INVARIANT_GAS_LIMIT, LIQUIDITY_TICKS_LIMIT } from './consts.js'
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
  TokenAmount,
  InvariantEvent,
  QuoteResult,
  LiquidityTick,
  Tickmap
} from './schema.js'
import { getServiceNamePrefix, ZERO_ADDRESS, getFnNamePrefix } from 'sails-js'

export class Invariant {
  eventListenerStarted: boolean = false
  private eventListeners: {
    [key in InvariantEvent]?: ((data: any) => void)[]
  } = {}

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

  static async load(api: GearApi, programId: HexString, gasLimit: bigint = INVARIANT_GAS_LIMIT) {
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

  on(callback: InvariantEventCallback): void {
    if (!this.eventListenerStarted) {
      this.listen()
    }

    this.eventListeners[callback.ident] = this.eventListeners[callback.ident] || []
    this.eventListeners[callback.ident]?.push(callback.callback)
  }

  private listen() {
    this.eventListenerStarted = true
    this.contract.api.gearEvents.subscribeToGearEvent(
      'UserMessageSent',
      ({ data: { message } }) => {
        if (!message.source.eq(this.contract.programId) || !message.destination.eq(ZERO_ADDRESS)) {
          return
        }

        const payload = message.payload.toHex()
        if (getServiceNamePrefix(payload) === 'Service') {
          const prefix = getFnNamePrefix(payload)
          if (Object.values(InvariantEvent).includes(prefix as any)) {
            const event = decodeEvent(this.contract.registry, payload, prefix)
            const callbacks = this.eventListeners[prefix as InvariantEvent]
            callbacks?.map(callback => callback(event))
          }
        }
      }
    )
  }

  off(callback: InvariantEventCallback): void {
    this.eventListeners[callback.ident] = this.eventListeners[callback.ident]?.filter(
      eventListener => {
        if (callback.callback) {
          return !(callback.callback === eventListener)
        } else {
          return false
        }
      }
    )
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

  async getPools(size: bigint, offset: bigint): Promise<PoolKey[]> {
    return unwrapResult(
      await this.contract.service.getPools(size as any, offset as any, DEFAULT_ADDRESS)
    ).map(convertPoolKey)
  }

  async getPosition(ownerId: ActorId, index: bigint): Promise<Position> {
    return convertPosition(
      unwrapResult(
        await this.contract.service.getPosition(
          ownerId as any,
          index as any,
          DEFAULT_ADDRESS as any
        )
      )
    )
  }

  async getAllPositions(ownerId: ActorId): Promise<Position[]> {
    return (
      await this.contract.service.getAllPositions(ownerId as any, DEFAULT_ADDRESS as any)
    ).map(val => convertPosition(val))
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

  async getTickmap(key: PoolKey): Promise<Tickmap> {
    return {
      bitmap: new Map(
        (await this.contract.service.getTickmap(key as any, DEFAULT_ADDRESS)).map(val => [
          BigInt(val[0]),
          BigInt(val[1])
        ])
      )
    }
  }

  async getAllLiquidityTicks(key: PoolKey, tickmap: Tickmap): Promise<LiquidityTick[]> {
    const tickIndexes: bigint[] = []
    for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
      for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
        const checkedBit = chunk & (1n << bit)
        if (checkedBit) {
          const tickIndex = positionToTick(chunkIndex, bit, key.feeTier.tickSpacing)
          tickIndexes.push(tickIndex)
        }
      }
    }
    const tickLimit = integerSafeCast(LIQUIDITY_TICKS_LIMIT)
    const promises: Promise<LiquidityTick[]>[] = []
    for (let i = 0; i < tickIndexes.length; i += tickLimit) {
      promises.push(this.getLiquidityTicks(key, tickIndexes.slice(i, i + tickLimit)))
    }
    const tickResults = await Promise.all(promises)
    return tickResults.flat(1)
  }
  async getLiquidityTicks(key: PoolKey, ticks: bigint[]): Promise<LiquidityTick[]> {
    return (
      unwrapResult(
        await this.contract.service.getLiquidityTicks(key as any, ticks as any, DEFAULT_ADDRESS)
      ) as any
    ).map(convertLiquidityTick)
  }

  async isTickInitialized(key: PoolKey, index: bigint): Promise<boolean> {
    return this.contract.service.isTickInitialized(
      key as any,
      integerSafeCast(index),
      DEFAULT_ADDRESS
    )
  }

  async getUserBalances(user: ActorId): Promise<Map<string, TokenAmount>> {
    const result = (await this.contract.service.getUserBalances(user as any, DEFAULT_ADDRESS)).map(
      arr => {
        arr[1] = BigInt(arr[1] as any) as any
        return arr
      }
    ) as any
    return new Map(result)
  }

  async quote(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean
  ): Promise<QuoteResult> {
    const sqrtPriceLimit: SqrtPrice = xToY
      ? getMinSqrtPrice(poolKey.feeTier.tickSpacing)
      : getMaxSqrtPrice(poolKey.feeTier.tickSpacing)

    return convertQuoteResult(
      unwrapResult(
        await this.contract.service.quote(
          poolKey as any,
          xToY,
          amount as any,
          byAmountIn,
          sqrtPriceLimit as any,
          DEFAULT_ADDRESS
        )
      )
    )
  }

  async changeProtocolFeeTx(
    fee: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<Percentage>> {
    return new TransactionWrapper<Percentage>(
      await this.contract.service.changeProtocolFee(fee as any).withGas(gasLimit)
    )
  }

  async addMultiplePositions(
    signer: Signer,
    poolKey: PoolKey,
    index: number,
    amount: bigint,
    step: number,
    maxTicks = false,
    gasLimit: bigint = this.gasLimit
  ): Promise<null> {
    const { response } = await (
      await this.contract.service
        .addMultiplePositions(poolKey as any, index as any, amount as any, step, maxTicks)
        .withGas(gasLimit)
    )
      .withAccount(signer)
      .signAndSend()
    return response()
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

  async claimFeeTx(
    index: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.claimFee(index as any).withGas(gasLimit)
    ).withDecode(arr => arr.map(BigInt))
  }

  async claimFee(
    signer: Signer,
    index: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.claimFeeTx(index, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async createPoolTx(
    key: PoolKey,
    initSqrtPrice: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<null>> {
    const initTick = calculateTick(initSqrtPrice, key.feeTier.tickSpacing)
    return new TransactionWrapper<null>(
      await this.contract.service
        .createPool(
          key.tokenX,
          key.tokenY,
          key.feeTier as any,
          initSqrtPrice as any,
          initTick as any
        )
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
    spotSqrtPrice: SqrtPrice,
    slippageTolerance: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<Position>> {
    const slippageLimitLower = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      false
    )
    const slippageLimitUpper = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      true
    )

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
    spotSqrtPrice: SqrtPrice,
    slippageTolerance: SqrtPrice,
    gasLimit: bigint = this.gasLimit
  ): Promise<Position> {
    const tx = (
      await this.createPositionTx(
        key,
        lowerTick,
        upperTick,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance,
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
    return new TransactionWrapper<TokenAmount>(
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

  async depositTokenPairTx(
    tokenX: [ActorId, TokenAmount],
    tokenY: [ActorId, TokenAmount],
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.depositTokenPair(tokenX as any, tokenY as any).withGas(gasLimit)
    ).withDecode(arr => arr.map(BigInt))
  }

  async depositTokenPair(
    signer: Signer,
    tokenX: [ActorId, TokenAmount],
    tokenY: [ActorId, TokenAmount],
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.depositTokenPairTx(tokenX, tokenY, gasLimit)).withAccount(signer)
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

  async removePositionTx(
    index: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.removePosition(index as any).withGas(gasLimit)
    ).withDecode(arr => arr.map(BigInt))
  }

  async removePosition(
    signer: Signer,
    index: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.removePositionTx(index, gasLimit)).withAccount(signer)
    return tx.send()
  }

  async transferPositionTx(
    index: bigint,
    receiver: ActorId,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<null>> {
    return new TransactionWrapper<null>(
      await this.contract.service.transferPosition(index as any, receiver as any).withGas(gasLimit)
    )
  }

  async transferPosition(
    signer: Signer,
    index: bigint,
    receiver: ActorId,
    gasLimit: bigint = this.gasLimit
  ): Promise<null> {
    const tx = (await this.transferPositionTx(index, receiver, gasLimit)).withAccount(signer)
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
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
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
    return new TransactionWrapper<TokenAmount>(
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

  async withdrawTokenPairTx(
    tokenX: [ActorId, TokenAmount | null],
    tokenY: [ActorId, TokenAmount | null],
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.withdrawTokenPair(tokenX as any, tokenY as any).withGas(gasLimit)
    ).withDecode(arr => arr.map(BigInt))
  }

  async withdrawTokenPair(
    signer: Signer,
    tokenX: [ActorId, TokenAmount | null],
    tokenY: [ActorId, TokenAmount | null],
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.withdrawTokenPairTx(tokenX, tokenY, gasLimit)).withAccount(signer)
    return tx.send()
  }
}
