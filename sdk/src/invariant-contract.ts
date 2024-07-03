import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder, getServiceNamePrefix, getFnNamePrefix, ZERO_ADDRESS } from 'sails-js';

export interface InvariantConfig {
  admin: string;
  protocol_fee: Percentage;
}

export type Percentage = [U128];

export type U128 = [Array<number | string>];

export interface FeeTier {
  fee: Percentage;
  tick_spacing: number;
}

export interface PoolKey {
  token_x: string;
  token_y: string;
  fee_tier: FeeTier;
}

export type TokenAmount = [number | string];

export type SqrtPrice = [U128];

export type Liquidity = [number | string];

export interface Position {
  pool_key: PoolKey;
  liquidity: Liquidity;
  lower_tick_index: number;
  upper_tick_index: number;
  fee_growth_inside_x: FeeGrowth;
  fee_growth_inside_y: FeeGrowth;
  last_block_number: number | string;
  tokens_owed_x: TokenAmount;
  tokens_owed_y: TokenAmount;
}

export type FeeGrowth = [U128];

export interface CalculateSwapResult {
  amount_in: TokenAmount;
  amount_out: TokenAmount;
  start_sqrt_price: SqrtPrice;
  target_sqrt_price: SqrtPrice;
  fee: TokenAmount;
  pool: Pool;
  ticks: Array<Tick>;
}

export interface Pool {
  liquidity: Liquidity;
  sqrt_price: SqrtPrice;
  current_tick_index: number;
  fee_growth_global_x: FeeGrowth;
  fee_growth_global_y: FeeGrowth;
  fee_protocol_token_x: TokenAmount;
  fee_protocol_token_y: TokenAmount;
  start_timestamp: number | string;
  last_timestamp: number | string;
  fee_receiver: string;
}

export interface Tick {
  index: number;
  sign: boolean;
  liquidity_change: Liquidity;
  liquidity_gross: Liquidity;
  sqrt_price: SqrtPrice;
  fee_growth_outside_x: FeeGrowth;
  fee_growth_outside_y: FeeGrowth;
  seconds_outside: number | string;
}

export type InvariantError = "notAdmin" | "notFeeReceiver" | "poolAlreadyExist" | "poolNotFound" | "tickAlreadyExist" | "invalidTickIndexOrTickSpacing" | "positionNotFound" | "tickNotFound" | "feeTierNotFound" | "poolKeyNotFound" | "amountIsZero" | "wrongLimit" | "priceLimitReached" | "noGainSwap" | "invalidTickSpacing" | "feeTierAlreadyExist" | "poolKeyAlreadyExist" | "unauthorizedFeeReceiver" | "zeroLiquidity" | "recoverableTransferError" | "unrecoverableTransferError" | "transferError" | "tokensAreSame" | "amountUnderMinimumAmountOut" | "invalidFee" | "notEmptyTickDeinitialization" | "invalidInitTick" | "invalidInitSqrtPrice" | "notEnoughGasToExecute" | "tickLimitReached" | "invalidTickIndex" | "noBalanceForTheToken" | "failedToChangeTokenBalance" | "replyHandlingFailed";

export interface QuoteResult {
  amount_in: TokenAmount;
  amount_out: TokenAmount;
  target_sqrt_price: SqrtPrice;
  ticks: Array<Tick>;
}

export interface SwapHop {
  pool_key: PoolKey;
  x_to_y: boolean;
}

export class InvariantContract {
  public readonly registry: TypeRegistry;
  public readonly service: Service;

  constructor(public api: GearApi, public programId?: `0x${string}`) {
    const types: Record<string, any> = {
      InvariantConfig: {"admin":"[u8;32]","protocolFee":"Percentage"},
      Percentage: "(U128)",
      U128: "([u64; 2])",
      FeeTier: {"fee":"Percentage","tickSpacing":"u16"},
      PoolKey: {"tokenX":"[u8;32]","tokenY":"[u8;32]","feeTier":"FeeTier"},
      TokenAmount: "(U256)",
      SqrtPrice: "(U128)",
      Liquidity: "(U256)",
      Position: {"poolKey":"PoolKey","liquidity":"Liquidity","lowerTickIndex":"i32","upperTickIndex":"i32","feeGrowthInsideX":"FeeGrowth","feeGrowthInsideY":"FeeGrowth","lastBlockNumber":"u64","tokensOwedX":"TokenAmount","tokensOwedY":"TokenAmount"},
      FeeGrowth: "(U128)",
      CalculateSwapResult: {"amountIn":"TokenAmount","amountOut":"TokenAmount","startSqrtPrice":"SqrtPrice","targetSqrtPrice":"SqrtPrice","fee":"TokenAmount","pool":"Pool","ticks":"Vec<Tick>"},
      Pool: {"liquidity":"Liquidity","sqrtPrice":"SqrtPrice","currentTickIndex":"i32","feeGrowthGlobalX":"FeeGrowth","feeGrowthGlobalY":"FeeGrowth","feeProtocolTokenX":"TokenAmount","feeProtocolTokenY":"TokenAmount","startTimestamp":"u64","lastTimestamp":"u64","feeReceiver":"[u8;32]"},
      Tick: {"index":"i32","sign":"bool","liquidityChange":"Liquidity","liquidityGross":"Liquidity","sqrtPrice":"SqrtPrice","feeGrowthOutsideX":"FeeGrowth","feeGrowthOutsideY":"FeeGrowth","secondsOutside":"u64"},
      InvariantError: {"_enum":["NotAdmin","NotFeeReceiver","PoolAlreadyExist","PoolNotFound","TickAlreadyExist","InvalidTickIndexOrTickSpacing","PositionNotFound","TickNotFound","FeeTierNotFound","PoolKeyNotFound","AmountIsZero","WrongLimit","PriceLimitReached","NoGainSwap","InvalidTickSpacing","FeeTierAlreadyExist","PoolKeyAlreadyExist","UnauthorizedFeeReceiver","ZeroLiquidity","RecoverableTransferError","UnrecoverableTransferError","TransferError","TokensAreSame","AmountUnderMinimumAmountOut","InvalidFee","NotEmptyTickDeinitialization","InvalidInitTick","InvalidInitSqrtPrice","NotEnoughGasToExecute","TickLimitReached","InvalidTickIndex","NoBalanceForTheToken","FailedToChangeTokenBalance","ReplyHandlingFailed"]},
      QuoteResult: {"amountIn":"TokenAmount","amountOut":"TokenAmount","targetSqrtPrice":"SqrtPrice","ticks":"Vec<Tick>"},
      SwapHop: {"poolKey":"PoolKey","xToY":"bool"},
    }

    this.registry = new TypeRegistry();
    this.registry.setKnownTypes({ types });
    this.registry.register(types);

    this.service = new Service(this);
  }

  newCtorFromCode(code: Uint8Array | Buffer, config: InvariantConfig): TransactionBuilder<null> {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'upload_program',
      ['New', config],
      '(String, InvariantConfig)',
      'String',
      code,
    );

    this.programId = builder.programId;
    return builder;
  }

  newCtorFromCodeId(codeId: `0x${string}`, config: InvariantConfig) {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'create_program',
      ['New', config],
      '(String, InvariantConfig)',
      'String',
      codeId,
    );

    this.programId = builder.programId;
    return builder;
  }
}

export class Service {
  constructor(private _program: InvariantContract) {}

  public addFeeTier(fee_tier: FeeTier): TransactionBuilder<FeeTier> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<FeeTier>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'AddFeeTier', fee_tier],
      '(String, String, FeeTier)',
      'FeeTier',
      this._program.programId
    );
  }

  public changeFeeReceiver(pool_key: PoolKey, fee_receiver: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'ChangeFeeReceiver', pool_key, fee_receiver],
      '(String, String, PoolKey, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public changeProtocolFee(protocol_fee: Percentage): TransactionBuilder<Percentage> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<Percentage>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'ChangeProtocolFee', protocol_fee],
      '(String, String, u128)',
      'u128',
      this._program.programId
    );
  }

  public claimFee(index: number): TransactionBuilder<[TokenAmount, TokenAmount]> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<[TokenAmount, TokenAmount]>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'ClaimFee', index],
      '(String, String, u32)',
      '(U256, U256)',
      this._program.programId
    );
  }

  public createPool(token_x: string, token_y: string, fee_tier: FeeTier, init_sqrt_price: SqrtPrice, init_tick: number): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'CreatePool', token_x, token_y, fee_tier, init_sqrt_price, init_tick],
      '(String, String, [u8;32], [u8;32], FeeTier, u128, i32)',
      'Null',
      this._program.programId
    );
  }

  public createPosition(pool_key: PoolKey, lower_tick: number, upper_tick: number, liquidity_delta: Liquidity, slippage_limit_lower: SqrtPrice, slippage_limit_upper: SqrtPrice): TransactionBuilder<Position> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<Position>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'CreatePosition', pool_key, lower_tick, upper_tick, liquidity_delta, slippage_limit_lower, slippage_limit_upper],
      '(String, String, PoolKey, i32, i32, U256, u128, u128)',
      'Position',
      this._program.programId
    );
  }

  public depositSingleToken(token: string, amount: TokenAmount): TransactionBuilder<TokenAmount> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<TokenAmount>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'DepositSingleToken', token, amount],
      '(String, String, [u8;32], U256)',
      'U256',
      this._program.programId
    );
  }

  public depositTokenPair(token_x: [string, TokenAmount], token_y: [string, TokenAmount]): TransactionBuilder<[TokenAmount, TokenAmount]> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<[TokenAmount, TokenAmount]>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'DepositTokenPair', token_x, token_y],
      '(String, String, ([u8;32], U256), ([u8;32], U256))',
      '(U256, U256)',
      this._program.programId
    );
  }

  public removeFeeTier(fee_tier: FeeTier): TransactionBuilder<FeeTier> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<FeeTier>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'RemoveFeeTier', fee_tier],
      '(String, String, FeeTier)',
      'FeeTier',
      this._program.programId
    );
  }

  public removePosition(index: number): TransactionBuilder<[TokenAmount, TokenAmount]> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<[TokenAmount, TokenAmount]>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'RemovePosition', index],
      '(String, String, u32)',
      '(U256, U256)',
      this._program.programId
    );
  }

  public swap(pool_key: PoolKey, x_to_y: boolean, amount: TokenAmount, by_amount_in: boolean, sqrt_price_limit: SqrtPrice): TransactionBuilder<CalculateSwapResult> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<CalculateSwapResult>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'Swap', pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit],
      '(String, String, PoolKey, bool, U256, bool, u128)',
      'CalculateSwapResult',
      this._program.programId
    );
  }

  public transferPosition(index: number, receiver: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'TransferPosition', index, receiver],
      '(String, String, u32, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public withdrawProtocolFee(pool_key: PoolKey): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'WithdrawProtocolFee', pool_key],
      '(String, String, PoolKey)',
      'Null',
      this._program.programId
    );
  }

  public withdrawSingleToken(token: string, amount: TokenAmount | null): TransactionBuilder<TokenAmount> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<TokenAmount>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'WithdrawSingleToken', token, amount],
      '(String, String, [u8;32], Option<U256>)',
      'U256',
      this._program.programId
    );
  }

  public withdrawTokenPair(token_x: [string, TokenAmount | null], token_y: [string, TokenAmount | null]): TransactionBuilder<[TokenAmount, TokenAmount]> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<[TokenAmount, TokenAmount]>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'WithdrawTokenPair', token_x, token_y],
      '(String, String, ([u8;32], Option<U256>), ([u8;32], Option<U256>))',
      '(U256, U256)',
      this._program.programId
    );
  }

  public async feeTierExists(fee_tier: FeeTier, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<boolean> {
    const payload = this._program.registry.createType('(String, String, FeeTier)', ['Service', 'FeeTierExists', fee_tier]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, bool)', reply.payload);
    return result[2].toJSON() as unknown as boolean;
  }

  public async getAllPositions(owner_id: string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<Position>> {
    const payload = this._program.registry.createType('(String, String, [u8;32])', ['Service', 'GetAllPositions', owner_id]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<Position>)', reply.payload);
    return result[2].toJSON() as unknown as Array<Position>;
  }

  public async getFeeTiers(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<FeeTier>> {
    const payload = this._program.registry.createType('(String, String)', ['Service', 'GetFeeTiers']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<FeeTier>)', reply.payload);
    return result[2].toJSON() as unknown as Array<FeeTier>;
  }

  public async getPool(token_x: string, token_y: string, fee_tier: FeeTier, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: Pool } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, [u8;32], [u8;32], FeeTier)', ['Service', 'GetPool', token_x, token_y, fee_tier]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<Pool, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: Pool } | { err: InvariantError };
  }

  public async getPools(size: number, offset: number, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: Array<PoolKey> } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, u8, u16)', ['Service', 'GetPools', size, offset]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<Vec<PoolKey>, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: Array<PoolKey> } | { err: InvariantError };
  }

  public async getPosition(owner_id: string, index: number, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: Position } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, [u8;32], u32)', ['Service', 'GetPosition', owner_id, index]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<Position, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: Position } | { err: InvariantError };
  }

  public async getProtocolFee(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Percentage> {
    const payload = this._program.registry.createType('(String, String)', ['Service', 'GetProtocolFee']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Percentage)', reply.payload);
    return result[2].toJSON() as unknown as Percentage;
  }

  public async getTick(key: PoolKey, index: number, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: Tick } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, PoolKey, i32)', ['Service', 'GetTick', key, index]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<Tick, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: Tick } | { err: InvariantError };
  }

  public async getUserBalances(user: string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<[string, TokenAmount]>> {
    const payload = this._program.registry.createType('(String, String, [u8;32])', ['Service', 'GetUserBalances', user]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<([u8;32], TokenAmount)>)', reply.payload);
    return result[2].toJSON() as unknown as Array<[string, TokenAmount]>;
  }

  public async isTickInitialized(key: PoolKey, index: number, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<boolean> {
    const payload = this._program.registry.createType('(String, String, PoolKey, i32)', ['Service', 'IsTickInitialized', key, index]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, bool)', reply.payload);
    return result[2].toJSON() as unknown as boolean;
  }

  public async quote(pool_key: PoolKey, x_to_y: boolean, amount: TokenAmount, by_amount_in: boolean, sqrt_price_limit: SqrtPrice, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: QuoteResult } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, PoolKey, bool, TokenAmount, bool, SqrtPrice)', ['Service', 'Quote', pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<QuoteResult, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: QuoteResult } | { err: InvariantError };
  }

  public async quoteRoute(amount_in: TokenAmount, swaps: Array<SwapHop>, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: TokenAmount } | { err: InvariantError }> {
    const payload = this._program.registry.createType('(String, String, TokenAmount, Vec<SwapHop>)', ['Service', 'QuoteRoute', amount_in, swaps]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Result<TokenAmount, InvariantError>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: TokenAmount } | { err: InvariantError };
  }

  public subscribeToPositionCreatedEventEvent(callback: (data: { timestamp: number | string; address: string; pool_key: PoolKey; liquidity_delta: Liquidity; lower_tick: number; upper_tick: number; current_sqrt_price: SqrtPrice }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'PositionCreatedEvent') {
        callback(this._program.registry.createType('(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","liquidityDelta":"Liquidity","lowerTick":"i32","upperTick":"i32","currentSqrtPrice":"SqrtPrice"})', message.payload)[2].toJSON() as any as { timestamp: number | string; address: string; pool_key: PoolKey; liquidity_delta: Liquidity; lower_tick: number; upper_tick: number; current_sqrt_price: SqrtPrice });
      }
    });
  }

  public subscribeToPositionRemovedEventEvent(callback: (data: { timestamp: number | string; address: string; pool_key: PoolKey; liquidity: Liquidity; lower_tick_index: number; upper_tick_index: number; sqrt_price: SqrtPrice }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'PositionRemovedEvent') {
        callback(this._program.registry.createType('(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","liquidity":"Liquidity","lowerTickIndex":"i32","upperTickIndex":"i32","sqrtPrice":"SqrtPrice"})', message.payload)[2].toJSON() as any as { timestamp: number | string; address: string; pool_key: PoolKey; liquidity: Liquidity; lower_tick_index: number; upper_tick_index: number; sqrt_price: SqrtPrice });
      }
    });
  }

  public subscribeToCrossTickEventEvent(callback: (data: { timestamp: number | string; address: string; pool_key: PoolKey; indexes: Array<number> }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'CrossTickEvent') {
        callback(this._program.registry.createType('(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","indexes":"Vec<i32>"})', message.payload)[2].toJSON() as any as { timestamp: number | string; address: string; pool_key: PoolKey; indexes: Array<number> });
      }
    });
  }

  public subscribeToSwapEventEvent(callback: (data: { timestamp: number | string; address: string; pool_key: PoolKey; amount_in: TokenAmount; amount_out: TokenAmount; fee: TokenAmount; start_sqrt_price: SqrtPrice; target_sqrt_price: SqrtPrice; x_to_y: boolean }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'SwapEvent') {
        callback(this._program.registry.createType('(String, String, {"timestamp":"u64","address":"[u8;32]","poolKey":"PoolKey","amountIn":"TokenAmount","amountOut":"TokenAmount","fee":"TokenAmount","startSqrtPrice":"SqrtPrice","targetSqrtPrice":"SqrtPrice","xToY":"bool"})', message.payload)[2].toJSON() as any as { timestamp: number | string; address: string; pool_key: PoolKey; amount_in: TokenAmount; amount_out: TokenAmount; fee: TokenAmount; start_sqrt_price: SqrtPrice; target_sqrt_price: SqrtPrice; x_to_y: boolean });
      }
    });
  }
}