import { GearApi, HexString, ProgramMetadata } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { Uint8ArrayToHexStr, getDeploymentData, getStateInput, getStateOutput } from './utils.js'
import { INVARIANT_GAS_LIMIT } from './consts.js'
import { ISubmittableResult } from '@polkadot/types/types'
import { EventListener } from './event-listener.js'

export class Invariant {
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
    Invariant.validateMetadata(meta)
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

    if (getStateOutput(meta) === null) {
      throw new Error('Metadata does not contain state output type')
    }
  }

  static async deploy(
    api: GearApi,
    eventListener: EventListener,
    deployer: KeyringPair,
    protocolFee: bigint,
    admin: HexString
  ) {
    const { metadata, wasm } = await getDeploymentData('invariant')
    const inputType = metadata.types.init.input!

    const init = metadata.createType(inputType, {
      config: {
        admin,
        protocolFee
      }
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
    }

    const returnMessage = eventListener.getByFinalizedResult(res)

    if (returnMessage === undefined) {
      throw new Error('No init event found')
    }
    const details = returnMessage.data.message.details

    if (details.isSome && details.unwrap().code.isError) {
      throw new Error(`Failed to upload invariant: ${returnMessage.data.message}`)
    }

    return new Invariant(api, eventListener, metadata, programId, INVARIANT_GAS_LIMIT)
  }
}
