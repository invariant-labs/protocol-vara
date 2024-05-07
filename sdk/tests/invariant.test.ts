import { EventListener } from '../src/event-listener'
import { initGearApi, subscribeToNewHeads, Uint8ArrayToHexStr } from '../src/utils'
import { GearKeyring } from '@gear-js/api'
import { LOCAL } from '../src/consts'
import { Invariant } from '../src/invariant'

const api = await initGearApi({ providerAddress: LOCAL })
const admin = await GearKeyring.fromSuri('//Alice')
const eventListener = new EventListener(api)
eventListener.listen()
let unsub: Promise<VoidFunction> | null = null

describe('Invariant', async function () {
  before(function () {
    unsub = subscribeToNewHeads(api)
  })
  it('deploys', async function () {
    Invariant.deploy(
      api,
      eventListener,
      admin,
      10000000000n,
      `0x${Uint8ArrayToHexStr(admin.publicKey)}`
    )
  })
  after(async function () {
    await unsub!.then(unsub => unsub())
  })
})
