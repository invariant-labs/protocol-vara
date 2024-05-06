import { u128, u64, u8 } from '@polkadot/types/primitive'
import { GearApi, HexString, ProgramMetadata } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import {
  FungibleTokenResponse,
  UserMessageStatus,
  Uint8ArrayToHexStr,
  getDeploymentData,
  getStateInput,
  FungibleTokenMetaTypes
} from './utils.js'
import { FUNGIBLE_TOKEN_GAS_LIMIT } from './consts.js'
import { ISubmittableResult } from '@polkadot/types/types'
import { MessageSendOptions } from '@gear-js/api'
import { Option } from '@polkadot/types/codec'
import { Codec } from '@polkadot/types/types'
import { EventListener } from './event-listener.js'
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
  private readonly gasLimit: bigint
  private readonly api: GearApi
  private readonly meta: ProgramMetadata
  readonly programId: HexString
  private readonly eventListener: EventListener
  private constructor(
    api: GearApi,
    eventListener: EventListener,
    meta: ProgramMetadata,
    programId: HexString,
    gasLimit: bigint
  ) {
    FungibleToken.validateMetadata(meta)
    this.api = api
    this.programId = programId
    this.meta = meta
    this.eventListener = eventListener
    this.gasLimit = gasLimit
  }
  static validateMetadata(meta: ProgramMetadata) {
    if (meta.types.init.input === null) {
      throw new Error('Metadata does not contain init type')
    }

    if (meta.types.handle.input === null) {
      throw new Error('Metadata does not contain handle input type')
    }

    if (meta.types.handle.output === null) {
      throw new Error('Metadata does not contain handle output type')
    }

    if (getStateInput(meta) === null) {
      throw new Error('Metadata does not contain state input type')
    }

    Object.values(FungibleTokenMetaTypes).forEach(name => {
      if (meta.getTypeIndexByName(name) === null) {
        throw new Error(`Metadata does not contain ${name} type`)
      }
    })
  }
  static async deploy(
    api: GearApi,
    eventListener: EventListener,
    deployer: KeyringPair,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n
  ) {
    const { metadata, wasm } = await getDeploymentData('fungible_token')
    const inputType = metadata.types.init.input!

    const init = metadata.createType(inputType, {
      name,
      symbol,
      decimals
    })

    const gas = await api.program.calculateGas.initUpload(
      `0x${Uint8ArrayToHexStr(deployer.publicKey)}`,
      wasm,
      init.toHex(), // payload
      0, // value
      false // allow other panics
    )

    const program = {
      code: wasm,
      gasLimit: gas.min_limit.toNumber() * 2,
      value: 0,
      initPayload: init.toHex()
    }

    const { programId, extrinsic } = api.program.upload(program, metadata)

    const event: Promise<ISubmittableResult> = new Promise(resolve => {
      extrinsic.signAndSend(deployer, async result => {
        if (result.isFinalized) {
          resolve(result)
        }
      })
    })

    const res = await event

    if (res.isError) {
      throw new Error(res.dispatchError?.toString())
    } else {
      return new FungibleToken(api, eventListener, metadata, programId, FUNGIBLE_TOKEN_GAS_LIMIT)
    }
  }

  private async sendMessage(
    user: KeyringPair,
    payload: Codec,
    inputType: number
  ): Promise<FungibleTokenResponse> {
    const message: MessageSendOptions = {
      payload: payload.toU8a(),
      gasLimit: this.gasLimit,
      destination: this.programId
    }
    const extrinsic = await this.api.message.send(message, this.meta, inputType)

    const send: Promise<ISubmittableResult> = new Promise(resolve => {
      extrinsic.signAndSend(user, result => {
        if (result.isFinalized) {
          resolve(result)
        }
      })
    })

    const finalized = await send
    if (finalized.isError) {
      throw new Error(`Error when sending a message, ${finalized.dispatchError?.toString()}`)
    }

    const returnMessage = this.eventListener.getByFinalizedResult(finalized as ISubmittableResult)
    if (returnMessage === undefined) {
      throw new Error('Message not found')
    }

    const details = returnMessage.data.message.details
    if (details.isSome && details.unwrap().code.isError) {
      return {
        status: UserMessageStatus.Panicked,
        panic: String.fromCharCode(...returnMessage.data.message.payload)
      }
    }
    const readableResponse = this.meta.createType(
      this.meta.types.handle.output!,
      returnMessage.data.message.payload
    )
    const json: any = readableResponse.toJSON()
    const err = json['err']
    if (err !== undefined && err !== null) {
      return { status: UserMessageStatus.ProcessedWithError, data: json }
    }
    return { status: UserMessageStatus.ProcessedSuccessfully, data: json }
  }

  async mint(signer: KeyringPair, amount: bigint): Promise<FungibleTokenResponse> {
    const inputType = this.meta.types.handle.input!
    const handle = this.meta.createType(inputType, { Mint: amount })

    return this.sendMessage(signer, handle, inputType)
  }

  async burn(signer: KeyringPair, amount: bigint): Promise<FungibleTokenResponse> {
    const inputType = this.meta.types.handle.input!
    const handle = this.meta.createType(inputType, { Burn: amount })

    return this.sendMessage(signer, handle, inputType)
  }

  async transfer(
    signer: KeyringPair,
    from: Uint8Array,
    to: Uint8Array,
    amount: bigint,
    txId?: bigint
  ): Promise<FungibleTokenResponse> {
    const inputType = this.meta.types.handle.input!
    const handle = this.meta.createType(inputType, { Transfer: { txId, amount, from, to } })

    return this.sendMessage(signer, handle, inputType)
  }

  async approve(
    signer: KeyringPair,
    to: Uint8Array,
    amount: bigint,
    txId?: bigint
  ): Promise<FungibleTokenResponse> {
    const inputType = this.meta.types.handle.input!
    const handle = this.meta.createType(inputType, { Approve: { txId, amount, to } })

    return this.sendMessage(signer, handle, inputType)
  }
  private async readState(payload: any, responseTypeIndex: FungibleTokenMetaTypes): Promise<Codec> {
    const encodedPayload = this.meta.createType(getStateInput(this.meta)!, payload).toU8a()
    const readProgramParams = {
      programId: this.programId,
      payload: encodedPayload
    }

    const state = await this.api.programState.read(
      readProgramParams,
      this.meta,
      this.meta.getTypeIndexByName(responseTypeIndex)!
    )
    return state
  }

  async balanceOf(account: Uint8Array): Promise<bigint> {
    const response = (await this.readState(
      { balanceOf: account },
      FungibleTokenMetaTypes.u128
    )) as any as u128
    return response.toBigInt()
  }

  async decimals(): Promise<bigint> {
    const response = (await this.readState('decimals', FungibleTokenMetaTypes.u8)) as any as u8
    return response.toBigInt()
  }

  async allowance(spender: Uint8Array, account: Uint8Array): Promise<bigint> {
    const response = (await this.readState(
      {
        allowance: {
          spender,
          account
        }
      },
      FungibleTokenMetaTypes.u128
    )) as any as u128

    return response.toBigInt()
  }

  async getTxValidityTime(account: Uint8Array, txId: bigint): Promise<bigint | null> {
    const option = (await this.readState(
      {
        getTxValidityTime: {
          account,
          txId
        }
      },
      FungibleTokenMetaTypes.OptionU64
    )) as any as Option<u64>

    return option.isSome ? option.unwrap().toBigInt() : null
  }

  async totalSupply(): Promise<bigint> {
    const response = await this.readState('totalSupply', FungibleTokenMetaTypes.u128)
    return (response as any as u128).toBigInt()
  }
}
