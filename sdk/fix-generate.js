import * as fs from 'fs'

const filesToModify = ['./src/erc20-token.ts', './src/invariant-contract.ts']

String.prototype.replaceAt = function (index, replacement, length) {
  return this.substring(0, index) + replacement + this.substring(index + length)
}

const snakeToCamel = str => {
  return str
    .toLowerCase()
    .replace(/([-_][a-z])/g, group => group.toUpperCase().replace('-', '').replace('_', ''))
}

for (const path of filesToModify) {
  let textBuff = fs
    .readFileSync(path, 'utf8')
    // fix missing invalid || null for v | undefined variable
    .replaceAll(`at: atBlock || null`, `at: atBlock`)
    // fix missing null check
    .replaceAll(
      ').toHex();\n    const reply = await this._program.api.message.calculateReply',
      ").toHex();\n    if (!this._program.programId) throw new Error('Program ID is not set');\n    const reply = await this._program.api.message.calculateReply"
    )

  // fix no parameter queries
  {
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
  }
  // fix transactions by replacing wrapper types with actual types
  const typesToReplace = [
    ['Percentage', 'u128'],
    ['TokenAmount', 'U256'],
    ['Liquidity', 'U256'],
    ['SqrtPrice', 'u128'],
    ['FeeGrowth', 'u128']
  ]
  {
    let transactionRegex =
      />\(\s*this._program.api,\s*this._program.registry,\s*'send_message',\s*^[^\}]*/gm
    textBuff.match(transactionRegex)?.map(val => {
      let match = val
      typesToReplace.map(type => {
        match = match.replaceAll(type[0], type[1])
      })
      textBuff = textBuff.replace(val, match)
    })
  }
  // camelize snake case JSON keys for easier use with invariant-vara-wasm types
  {
    let snakeCaseRegex = /"(?!_)[a-z_]*":/g
    textBuff.match(snakeCaseRegex)?.map(val => {
      textBuff = textBuff.replace(val, snakeToCamel(val))
    })
  }
  // fix missing any cast
  textBuff = textBuff.replaceAll(
    'message.payload)[2].toJSON() as {',
    'message.payload)[2].toJSON() as any as {'
  )
  fs.writeFileSync(path, textBuff, 'utf8')
}
