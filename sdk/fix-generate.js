import * as fs from 'fs'

const filesToModify = ['./src/erc20-token.ts', './src/invariant-contract.ts']

String.prototype.replaceAt = function (index, replacement, length) {
  return this.substring(0, index) + replacement + this.substring(index + length)
}

for (const path of filesToModify) {
  let textBuff = fs
    .readFileSync(path, 'utf8')
    .replaceAll(`at: atBlock || null`, `at: atBlock`)
    .replaceAll(
      ').toHex();\n    const reply = await this._program.api.message.calculateReply',
      ").toHex();\n    if (!this._program.programId) throw new Error('Program ID is not set');\n    const reply = await this._program.api.message.calculateReply"
    )
  // matching no parameter queries
  let regex = /\'\(String, String\)\', '\[(.+?), (.+?)\]'/g
  let matches = textBuff.match(regex)
  matches?.map(val => {
    let match = val.replaceAll("'[", "['").replaceAll("]'", "']")

    let matchCount = 0
    for (let j = 0; j < match.length; j++) {
      if (match[j] === ',') {
        matchCount++
        if (matchCount === 3) {
          match = match.replaceAt(j, "', '", 2)
          break
        }
      }
    }
    textBuff = textBuff.replace(val, match)
  })
  textBuff = textBuff.replaceAll(
    'message.payload)[2].toJSON() as {',
    'message.payload)[2].toJSON() as any as {'
  )
  fs.writeFileSync(path, textBuff, 'utf8')
}
