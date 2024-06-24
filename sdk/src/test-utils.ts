import { assert } from 'chai'
import { FungibleToken } from './erc20'
import { isTokenX } from 'invariant-vara-wasm'

export const objectEquals = (
  object: { [key: string]: any },
  expectedObject: { [key: string]: any },
  keys: string[]
) => {
  for (const key in object) {
    if (!keys.includes(key)) {
      assert.deepEqual(object[key], expectedObject[key])
    }
  }
}

export const sortTokens = (tokenX: FungibleToken, tokenY: FungibleToken) => {
  return isTokenX(tokenX.programId(), tokenY.programId())
  ? [tokenX, tokenY]
  : [tokenY, tokenX]
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: string) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}