import { ProgramMetadata, UserMessageSent } from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'
import { ISubmittableResult } from '@polkadot/types/types'
import { U8aFixed } from '@polkadot/types/codec'

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
  data: string
}

export const assertProcessed = (res: FungibleTokenResponse) => {
  if (res.status !== UserMessageStatus.ProcessedSuccessfully) {
    throw new Error(`Expected to be processed`)
  }
}
export const assertPanicked = (res: FungibleTokenResponse) => {
  if (res.status !== UserMessageStatus.Panicked) {
    throw new Error(`Expected panic`)
  }
}
export const assertProcessedWithError = (res: FungibleTokenResponse) => {
  if (res.status !== UserMessageStatus.ProcessedWithError) {
    throw new Error(`Expected to be processed with error`)
  }
}
