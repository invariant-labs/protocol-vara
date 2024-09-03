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
  convertPositionTick,
  convertPositions,
  positionToTick,
  validateInvariantPairDeposit,
  validateInvariantPairWithdraw,
  validateInvariantSingleDeposit,
  validateInvariantSingleWithdraw
} from './utils.js'
import {
  CHUNK_SIZE,
  DEFAULT_ADDRESS,
  INVARIANT_GAS_LIMIT,
  LIQUIDITY_TICKS_LIMIT,
  MAX_POOL_KEYS_RETURNED,
  POSITIONS_ENTRIES_LIMIT
} from './consts.js'
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
  Tickmap,
  PositionTick,
  SwapHop
} from './schema.js'
import { getServiceNamePrefix, ZERO_ADDRESS, getFnNamePrefix } from 'sails-js'

export type Page = { index: number; entries: [Position, Pool][] }

export class Invariant {
  eventListenerStarted: boolean = false
  private eventListeners: {
    [key in InvariantEvent]?: ((data: any) => void)[]
  } = {}

  private constructor(readonly contract: InvariantContract, private readonly gasLimit: bigint) {}

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

  async getPoolKeys(size: bigint, offset: bigint): Promise<[PoolKey[], bigint]> {
    const response = await this.contract.service.getPoolKeys(
      size as any,
      offset as any,
      DEFAULT_ADDRESS
    )
    return [response[0].map(convertPoolKey), BigInt(response[1])]
  }

  async getAllPoolKeys(): Promise<PoolKey[]> {
    const [poolKeys, poolKeysCount] = await this.getPoolKeys(MAX_POOL_KEYS_RETURNED, 0n)
    const promises: Promise<[PoolKey[], bigint]>[] = []
    for (let i = 1; i < Math.ceil(Number(poolKeysCount) / Number(MAX_POOL_KEYS_RETURNED)); i++) {
      promises.push(this.getPoolKeys(MAX_POOL_KEYS_RETURNED, BigInt(i) * MAX_POOL_KEYS_RETURNED))
    }

    const poolKeysEntries = await Promise.all(promises)
    return [...poolKeys, ...poolKeysEntries.map(([poolKeys]) => poolKeys).flat(1)]
  }
  async getAllPoolsForPair(token0: ActorId, token1: ActorId): Promise<[FeeTier, Pool][]> {
    return unwrapResult(
      await this.contract.service.getAllPoolsForPair(token0 as any, token1 as any, DEFAULT_ADDRESS)
    ).map((entry: any) => [convertFeeTier(entry[0]), convertPool(entry[1])])
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

  async getAllPositions(
    owner: ActorId,
    positionsCount?: bigint,
    skipPages?: number[],
    positionsPerPage?: bigint
  ) {
    const firstPageIndex = skipPages?.find(i => !skipPages.includes(i)) || 0
    const positionsPerPageLimit = positionsPerPage || POSITIONS_ENTRIES_LIMIT

    let pages: Page[] = []
    let actualPositionsCount = positionsCount
    if (!positionsCount) {
      const [positionEntries, positionsCount] = await this.getPositions(
        owner,
        positionsPerPageLimit,
        BigInt(firstPageIndex) * positionsPerPageLimit
      )

      pages.push({ index: 0, entries: positionEntries })
      actualPositionsCount = positionsCount
    }

    const promises: Promise<[[Position, Pool][], bigint]>[] = []
    const pageIndexes: number[] = []

    for (
      let i = positionsCount ? firstPageIndex : firstPageIndex + 1;
      i < Math.ceil(Number(actualPositionsCount) / Number(positionsPerPageLimit));
      i++
    ) {
      if (skipPages?.includes(i)) {
        continue
      }

      pageIndexes.push(i)
      promises.push(
        this.getPositions(owner, positionsPerPageLimit, BigInt(i) * positionsPerPageLimit)
      )
    }

    const positionsEntriesList = await Promise.all(promises)
    pages = [
      ...pages,
      ...positionsEntriesList.map(([positionsEntries], index) => {
        return { index: pageIndexes[index], entries: positionsEntries }
      })
    ]

    return pages
  }

  async getPositionWithAssociates(
    owner: ActorId,
    index: bigint
  ): Promise<[Position, Pool, Tick, Tick]> {
    const result = unwrapResult(
      await this.contract.service.getPositionWithAssociates(
        owner as any,
        index as any,
        DEFAULT_ADDRESS
      )
    )
    const position = convertPosition(result[0])
    const pool = convertPool(result[1])
    const lowerTick = convertTick(result[2])
    const upperTick = convertTick(result[3])

    return [position, pool, lowerTick, upperTick]
  }
  async getPositions(
    ownerId: ActorId,
    size: bigint,
    offset: bigint
  ): Promise<[[Position, Pool][], bigint]> {
    const response = unwrapResult(
      await this.contract.service.getPositions(
        ownerId as any,
        size as any,
        offset as any,
        DEFAULT_ADDRESS as any
      )
    )
    const mapEntries = ([pool, positions]: [Pool, Position[]]): [Position, Pool][] => {
      return positions.map(position => {
        return [position, pool]
      })
    }

    return [convertPositions(response[0]).map(mapEntries).flat(1), BigInt(response[1])]
  }

  async _getAllPositions(ownerId: ActorId): Promise<Position[]> {
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

  async getLiquidityTicks(key: PoolKey, tickmap: bigint[]): Promise<LiquidityTick[]> {
    return unwrapResult(
      await this.contract.service.getLiquidityTicks(key as any, tickmap as any, DEFAULT_ADDRESS)
    ).map(convertLiquidityTick)
  }

  async getLiquidityTicksAmount(key: PoolKey): Promise<TokenAmount> {
    return BigInt(await this.contract.service.getLiquidityTicksAmount(key as any, DEFAULT_ADDRESS))
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

  async getUserPositionAmount(key: PoolKey, owner: ActorId): Promise<TokenAmount> {
    return BigInt(
      await this.contract.service.getUserPositionAmount(key as any, owner as any, DEFAULT_ADDRESS)
    )
  }

  async getPositionTicks(owner: ActorId, offset: bigint): Promise<PositionTick[]> {
    return (
      await this.contract.service.getPositionTicks(owner as any, offset as any, DEFAULT_ADDRESS)
    ).map(tick => convertPositionTick(tick))
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

  async changeProtocolFee(
    signer: Signer,
    fee: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<Percentage> {
    const tx = (await this.changeProtocolFeeTx(fee, gasLimit)).withAccount(signer)
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
  }

  async depositSingleTokenTx(
    token: ActorId,
    amount: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<TokenAmount>> {
    return new TransactionWrapper<TokenAmount>(
      await this.contract.service.depositSingleToken(token as any, amount as any).withGas(gasLimit)
    ).withValidate(validateInvariantSingleDeposit)
  }

  async depositSingleToken(
    signer: Signer,
    token: ActorId,
    amount: bigint,
    gasLimit: bigint = this.gasLimit
  ): Promise<TokenAmount> {
    const tx = (await this.depositSingleTokenTx(token, amount, gasLimit)).withAccount(signer)
    return tx.signAndSend()
  }

  async depositTokenPairTx(
    tokenX: [ActorId, TokenAmount],
    tokenY: [ActorId, TokenAmount],
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.depositTokenPair(tokenX as any, tokenY as any).withGas(gasLimit)
    )
      .withDecode(arr => arr.map(BigInt))
      .withValidate(validateInvariantPairDeposit)
  }

  async depositTokenPair(
    signer: Signer,
    tokenX: [ActorId, TokenAmount],
    tokenY: [ActorId, TokenAmount],
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.depositTokenPairTx(tokenX, tokenY, gasLimit)).withAccount(signer)
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
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
    return tx.signAndSend()
  }

  async swapWithSlippageTx(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    estimatedSqrtPrice: SqrtPrice,
    slippage: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<CalculateSwapResult>> {
    const sqrtPriceAfterSlippage = calculateSqrtPriceAfterSlippage(
      estimatedSqrtPrice,
      slippage,
      !xToY
    )
    return new TransactionWrapper<CalculateSwapResult>(
      await this.contract.service
        .swap(
          poolKey as any,
          xToY,
          amount as any,
          byAmountIn,
          xToY ? sqrtPriceAfterSlippage - 1n : ((sqrtPriceAfterSlippage + 1n) as any)
        )
        .withGas(gasLimit)
    ).withDecode(convertCalculateSwapResult)
  }

  async swapWithSlippage(
    signer: Signer,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    estimatedSqrtPrice: SqrtPrice,
    slippage: Percentage,
    gasLimit: bigint = this.gasLimit
  ): Promise<CalculateSwapResult> {
    const tx = (
      await this.swapWithSlippageTx(
        poolKey,
        xToY,
        amount,
        byAmountIn,
        estimatedSqrtPrice,
        slippage,
        gasLimit
      )
    ).withAccount(signer)
    return tx.signAndSend()
  }

  async swapRouteTx(
    amountIn: TokenAmount,
    expectedAmountOut: TokenAmount,
    slippage: Percentage,
    swaps: SwapHop[],
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<TokenAmount>> {
    return new TransactionWrapper<TokenAmount>(
      await this.contract.service
        .swapRoute(amountIn as any, expectedAmountOut as any, slippage as any, swaps as any[])
        .withGas(gasLimit)
    )
  }

  async swapRoute(
    signer: Signer,
    amountIn: TokenAmount,
    expectedAmountOut: TokenAmount,
    slippage: Percentage,
    swaps: SwapHop[],
    gasLimit: bigint = this.gasLimit
  ): Promise<TokenAmount> {
    const tx = (
      await this.swapRouteTx(amountIn, expectedAmountOut, slippage, swaps, gasLimit)
    ).withAccount(signer)
    return tx.signAndSend()
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
    return tx.signAndSend()
  }

  async withdrawSingleTokenTx(
    token: ActorId,
    amount: TokenAmount | null = null,
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<TokenAmount>> {
    return new TransactionWrapper<TokenAmount>(
      await this.contract.service.withdrawSingleToken(token as any, amount as any).withGas(gasLimit)
    ).withValidate(validateInvariantSingleWithdraw)
  }

  async withdrawSingleToken(
    signer: Signer,
    token: ActorId,
    amount: TokenAmount | null = null,
    gasLimit: bigint = this.gasLimit
  ): Promise<TokenAmount> {
    const tx = (await this.withdrawSingleTokenTx(token, amount, gasLimit)).withAccount(signer)
    return tx.signAndSend()
  }

  async withdrawTokenPairTx(
    tokenX: [ActorId, TokenAmount | null],
    tokenY: [ActorId, TokenAmount | null],
    gasLimit: bigint = this.gasLimit
  ): Promise<TransactionWrapper<[TokenAmount, TokenAmount]>> {
    return new TransactionWrapper<[TokenAmount, TokenAmount]>(
      await this.contract.service.withdrawTokenPair(tokenX as any, tokenY as any).withGas(gasLimit)
    )
      .withDecode(arr => arr.map(BigInt))
      .withValidate(validateInvariantPairWithdraw)
  }

  async withdrawTokenPair(
    signer: Signer,
    tokenX: [ActorId, TokenAmount | null],
    tokenY: [ActorId, TokenAmount | null],
    gasLimit: bigint = this.gasLimit
  ): Promise<[TokenAmount, TokenAmount]> {
    const tx = (await this.withdrawTokenPairTx(tokenX, tokenY, gasLimit)).withAccount(signer)
    return tx.signAndSend()
  }
}
