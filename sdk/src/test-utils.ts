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