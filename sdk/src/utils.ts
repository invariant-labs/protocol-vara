import {
  GearApi,
  GearApiOptions,
  HumanTypesRepr,
  ProgramMetadata,
  UserMessageSent
} from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'
import { ISubmittableResult } from '@polkadot/types/types'
import { U8aFixed } from '@polkadot/types/codec'

export const initGearApi = async (gearApiOptions: GearApiOptions | undefined) => {
  const gearApi = await GearApi.create(gearApiOptions)

  const [chain, nodeName, nodeVersion] = await Promise.all([
    gearApi.chain(),
    gearApi.nodeName(),
    gearApi.nodeVersion()
  ])

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`)

  return gearApi
}

// returns usnub function
export const subscribeToNewHeads = async (api: GearApi): Promise<VoidFunction> => {
  return await api.blocks.subscribeNewHeads(header => {
    console.log(
      `New block with number: ${header.number.toNumber()} and hash: ${header.hash.toHex()}`
    )
  })

}


export const getDeploymentData = async (
  contractName: string
): Promise<{ metadata: ProgramMetadata; wasm: Buffer }> => {
  const __dirname = new URL('.', import.meta.url).pathname

  try {
    const metadata = ProgramMetadata.from(
      (
        await readFile(
          path.join(__dirname, `../contracts/${contractName}/${contractName}.meta.txt`)
        )
      ).toString()
    )
    const wasm = await readFile(
      path.join(__dirname, `../contracts/${contractName}/${contractName}.opt.wasm`)
    )

    return { metadata, wasm }
  } catch (error) {
    throw new Error(`${contractName}.meta.txt or ${contractName}.opt.wasm not found`)
  }
}

export const Uint8ArrayToHexStr = (bits: Uint8Array): string => {
  return bits.reduce((acc, val) => acc + val.toString(16).padStart(2, '0'), '')
}

export const getResponseData = (res: UserMessageSent, meta: ProgramMetadata, typeIndex: number) => {
  const message = res.data.message
  const details = res.data.message.details.unwrap()
  if (details.code.isError) {
    throw new Error(`Message panicked: ${message.toHuman()}`)
  }
  const response = meta.createType(typeIndex, message.payload)
  return response
}

export const getMessageId = (res: ISubmittableResult): U8aFixed => {
  for (const ev of res.events) {
    if (ev.event.method === 'MessageQueued') {
      return (ev.event.data as any)['id'] as U8aFixed
    }
  }
  throw new Error('MessageQueued event not found')
}

export enum UserMessageStatus {
  ProcessedSuccessfully,
  Panicked,
  ProcessedWithError
}

export type FungibleTokenResponse = {
  status: UserMessageStatus
  data?: any
  panic?: string
}

// these functions should used to unify HumanProgramMetadataReprRustV1 and V2 interfaces
export const getStateInput = (meta: ProgramMetadata): number | null => {
  const state = meta.types.state
  if (typeof state === 'object') {
    return (state as HumanTypesRepr).input
  } else {
    throw new Error('State input is not available in metadata V1')
  }
}
export const getStateOutput = (meta: ProgramMetadata) => {
  const state = meta.types.state
  if (typeof state === 'object') {
    return (state as HumanTypesRepr).output
  } else {
    return state as number
  }
}

export enum FungibleTokenMetaTypes {
  u128 = 'u128',
  u64 = 'u64',
  u8 = 'u8',
  OptionU64 = 'Option<u64>'
}

export const createTypeByName = (meta: ProgramMetadata, type: string, payload: any) => {
  return meta.createType(meta.getTypeIndexByName(type)!, payload)
}
