import { GearApi, ProgramMetadata } from '@gear-js/api'
import { KeyringPair } from '@polkadot/keyring/types'
import { Uint8ArrayToHexStr, getDeploymentData } from './utils.js'
import { GAS_LIMIT_FUNGIBLE_TOKEN as FUNGIBLE_TOKEN_GAS_LIMIT } from './consts.js'
import { ISubmittableResult } from '@polkadot/types/types'
export class FungibleToken {
  meta: ProgramMetadata
  programId: string
  gasLimit: bigint

  constructor(meta: ProgramMetadata, programId: string, gasLimit: bigint) {
    this.programId = programId
    this.meta = meta
    this.gasLimit = gasLimit
  }

  static async deploy(
    api: GearApi,
    deployer: KeyringPair,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n
  ) {
    const { metadata, wasm } = await getDeploymentData('fungible_token')
    const inputType = metadata.types.init.input
    if (inputType === null || inputType === undefined) {
      throw new Error('Metadata does not contain init input type')
    }

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
      return new FungibleToken(metadata, programId, FUNGIBLE_TOKEN_GAS_LIMIT)
    }
  }
}
