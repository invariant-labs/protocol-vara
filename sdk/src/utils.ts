import { GearApi, GearApiOptions, HexString, ProgramMetadata } from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'
import { IKeyringPair } from '@polkadot/types/types'
import {
  CalculateSwapResult,
  FeeTier,
  Percentage,
  Pool,
  PoolKey,
  Position,
  Tick,
  _newFeeTier,
  _newPoolKey
} from 'invariant-vara-wasm'
export type Signer = string | IKeyringPair
export type ActorId = Uint8Array | HexString

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

export const getWasm = async (contractName: string): Promise<Buffer> => {
  const __dirname = new URL('.', import.meta.url).pathname

  return readFile(path.join(__dirname, `../contracts/${contractName}/${contractName}.opt.wasm`))
}

export const createTypeByName = (meta: ProgramMetadata, type: string, payload: any) => {
  return meta.createType(meta.getTypeIndexByName(type)!, payload)
}

export const integerSafeCast = (value: bigint): number => {
  if (value > BigInt(Number.MAX_SAFE_INTEGER) || value < BigInt(Number.MIN_SAFE_INTEGER)) {
    throw new Error('Integer value is outside the safe range for Numbers')
  }
  return Number(value)
}

export type Result<T> = { ok: T } | { err: string }
export const unwrapResult = <T>(result: Result<T>): T => {
  if ('ok' in result) {
    return result.ok
  } else if (result.err) {
    throw new Error(result.err)
  } else {
    return result as any

    throw new Error('Invalid Result type')
  }
}

export const newFeeTier = (fee: Percentage, tickSpacing: bigint): FeeTier => {
  return _newFeeTier(fee, integerSafeCast(tickSpacing))
}

export const newPoolKey = (token0: HexString, token1: HexString, feeTier: FeeTier): PoolKey => {
  // remove 0x prefix
  return _newPoolKey(token0, token1, feeTier)
}

const convertFieldsToBigInt = (returnedObject: any, exclude?: string[]): any => {
  for (const [key, value] of Object.entries(returnedObject)) {
    if (exclude?.includes(key)) {
      continue
    }
    if (typeof value === 'number' || typeof value === 'string') {
      returnedObject[key] = BigInt(value as any)
    }
  }
  return returnedObject
}

export const convertTick = (tick: any): Tick => {
  return convertFieldsToBigInt(tick)
}

export const convertFeeTier = (feeTier: any): FeeTier => {
  return convertFieldsToBigInt(feeTier, ['tickSpacing'])
}

export const convertPoolKey = (poolKey: any): PoolKey => {
  poolKey.feeTier = convertFeeTier(poolKey.feeTier)
  return poolKey
}

export const convertPool = (pool: any): Pool => {
  return convertFieldsToBigInt(pool, ['currentIndex'])
}

export const convertPosition = (position: any): Position => {
  position = convertFieldsToBigInt(position, ['poolKey'])
  position.poolKey = convertPoolKey(position.poolKey)
  return position as Position
}

export const convertCalculateSwapResult = (calculateSwapResult: any): CalculateSwapResult => {
  calculateSwapResult = convertFieldsToBigInt(calculateSwapResult, ['pool', 'ticks'])
  calculateSwapResult.pool = convertPool(calculateSwapResult.pool)
  calculateSwapResult.ticks = calculateSwapResult.ticks.map(convertTick)

  return calculateSwapResult
}

export interface ITransactionBuilder {
  signAndSend(): Promise<{ response: () => Promise<any> }>
  withAccount(signer: Signer): void
}

export class TransactionWrapper<U> {
  private txBuilder: ITransactionBuilder
  private decodeCallback: ((t: any) => U) | null = null
  constructor(txBuilder: ITransactionBuilder) {
    this.txBuilder = txBuilder
  }

  async send(): Promise<U> {
    const { response } = await this.txBuilder.signAndSend()
    if (this.decodeCallback) {
      return this.decodeCallback(await response())
    }

    return await response()
  }

  withAccount(signer: Signer): this {
    this.txBuilder.withAccount(signer)
    return this
  }

  withDecode(decodeFn: (t: any) => U): this {
    this.decodeCallback = decodeFn
    return this
  }
}
