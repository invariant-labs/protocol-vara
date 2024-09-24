import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder, getServiceNamePrefix, getFnNamePrefix, ZERO_ADDRESS } from 'sails-js';

export class Erc20Token {
  public readonly registry: TypeRegistry;
  public readonly vft: Vft;

  constructor(public api: GearApi, public programId?: `0x${string}`) {
    const types: Record<string, any> = {
    }

    this.registry = new TypeRegistry();
    this.registry.setKnownTypes({ types });
    this.registry.register(types);

    this.vft = new Vft(this);
  }

  newCtorFromCode(code: Uint8Array | any, name: string, symbol: string, decimals: number): TransactionBuilder<null> {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'upload_program',
      ['New', name, symbol, decimals],
      '(String, String, String, u8)',
      'String',
      code,
    );

    this.programId = builder.programId;
    return builder;
  }

  newCtorFromCodeId(codeId: `0x${string}`, name: string, symbol: string, decimals: number) {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'create_program',
      ['New', name, symbol, decimals],
      '(String, String, String, u8)',
      'String',
      codeId,
    );

    this.programId = builder.programId;
    return builder;
  }
}

export class Vft {
  constructor(private _program: Erc20Token) {}

  public burn(from: string, value: number | string): TransactionBuilder<boolean> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<boolean>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'Burn', from, value],
      '(String, String, [u8;32], U256)',
      'bool',
      this._program.programId
    );
  }

  public grantAdminRole(to: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'GrantAdminRole', to],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public grantBurnerRole(to: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'GrantBurnerRole', to],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public grantMinterRole(to: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'GrantMinterRole', to],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public mint(to: string, value: number | string): TransactionBuilder<boolean> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<boolean>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'Mint', to, value],
      '(String, String, [u8;32], U256)',
      'bool',
      this._program.programId
    );
  }

  public revokeAdminRole(from: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'RevokeAdminRole', from],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public revokeBurnerRole(from: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'RevokeBurnerRole', from],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public revokeMinterRole(from: string): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'RevokeMinterRole', from],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId
    );
  }

  public setTransferFail(flag: boolean): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'SetTransferFail', flag],
      '(String, String, bool)',
      'Null',
      this._program.programId
    );
  }

  public transfer(to: string, value: number | string): TransactionBuilder<boolean> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<boolean>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'Transfer', to, value],
      '(String, String, [u8;32], U256)',
      'bool',
      this._program.programId
    );
  }

  public transferFrom(from: string, to: string, value: number | string): TransactionBuilder<boolean> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<boolean>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'TransferFrom', from, to, value],
      '(String, String, [u8;32], [u8;32], U256)',
      'bool',
      this._program.programId
    );
  }

  public approve(spender: string, value: number | string): TransactionBuilder<boolean> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<boolean>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Vft', 'Approve', spender, value],
      '(String, String, [u8;32], U256)',
      'bool',
      this._program.programId
    );
  }

  public async admins(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<string>> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Admins']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<[u8;32]>)', reply.payload);
    return result[2].toJSON() as unknown as Array<string>;
  }

  public async burners(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<string>> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Burners']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<[u8;32]>)', reply.payload);
    return result[2].toJSON() as unknown as Array<string>;
  }

  public async minters(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<string>> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Minters']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, Vec<[u8;32]>)', reply.payload);
    return result[2].toJSON() as unknown as Array<string>;
  }

  public async allowance(owner: string, spender: string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<bigint> {
    const payload = this._program.registry.createType('(String, String, [u8;32], [u8;32])', ['Vft', 'Allowance', owner, spender]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, U256)', reply.payload);
    return result[2].toBigInt() as unknown as bigint;
  }

  public async balanceOf(account: string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<bigint> {
    const payload = this._program.registry.createType('(String, String, [u8;32])', ['Vft', 'BalanceOf', account]).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, U256)', reply.payload);
    return result[2].toBigInt() as unknown as bigint;
  }

  public async decimals(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<number> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Decimals']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, u8)', reply.payload);
    return result[2].toNumber() as unknown as number;
  }

  public async name(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<string> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Name']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, String)', reply.payload);
    return result[2].toString() as unknown as string;
  }

  public async symbol(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<string> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'Symbol']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, String)', reply.payload);
    return result[2].toString() as unknown as string;
  }

  public async totalSupply(originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<bigint> {
    const payload = this._program.registry.createType('(String, String)', ['Vft', 'TotalSupply']).toHex();
    if (!this._program.programId) throw new Error('Program ID is not set');
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    const result = this._program.registry.createType('(String, String, U256)', reply.payload);
    return result[2].toBigInt() as unknown as bigint;
  }

  public subscribeToMintedEvent(callback: (data: { to: string; value: number | string }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Vft' && getFnNamePrefix(payload) === 'Minted') {
        callback(this._program.registry.createType('(String, String, {"to":"[u8;32]","value":"U256"})', message.payload)[2].toJSON() as any as { to: string; value: number | string });
      }
    });
  }

  public subscribeToBurnedEvent(callback: (data: { from: string; value: number | string }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Vft' && getFnNamePrefix(payload) === 'Burned') {
        callback(this._program.registry.createType('(String, String, {"from":"[u8;32]","value":"U256"})', message.payload)[2].toJSON() as any as { from: string; value: number | string });
      }
    });
  }

  public subscribeToApprovalEvent(callback: (data: { owner: string; spender: string; value: number | string }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Vft' && getFnNamePrefix(payload) === 'Approval') {
        callback(this._program.registry.createType('(String, String, {"owner":"[u8;32]","spender":"[u8;32]","value":"U256"})', message.payload)[2].toJSON() as any as { owner: string; spender: string; value: number | string });
      }
    });
  }

  public subscribeToTransferEvent(callback: (data: { from: string; to: string; value: number | string }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Vft' && getFnNamePrefix(payload) === 'Transfer') {
        callback(this._program.registry.createType('(String, String, {"from":"[u8;32]","to":"[u8;32]","value":"U256"})', message.payload)[2].toJSON() as any as { from: string; to: string; value: number | string });
      }
    });
  }
}