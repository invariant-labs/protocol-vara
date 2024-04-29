import { ProgramMetadata } from '@gear-js/api'
import { readFile } from 'fs/promises'
import path from 'path'

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
