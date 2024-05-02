import { GearApi, UserMessageSent } from '@gear-js/api'
import { U8aFixed } from '@polkadot/types/codec'
import { ISubmittableResult } from '@polkadot/types/types'
import { getMessageId } from './utils.js'
export class EventListener {
  readonly api: GearApi
  readonly messages: Array<UserMessageSent> = []
  constructor(api: GearApi) {
    this.api = api
  }

  async listen() {
    await this.api.gearEvents.subscribeToGearEvent(
      'UserMessageSent',
      (message: UserMessageSent) => {
        console.log(message.toHuman())
        this.messages.push(message)
      }
    )
  }

  lastMessage(): UserMessageSent | undefined {
    return this.messages[this.messages.length - 1]
  }

  getMessageById(id: U8aFixed): UserMessageSent | undefined {
    return this.messages.find(m => m.data.message.details.unwrap().to.eq(id))
  }

  getByFinalizedResult(res: ISubmittableResult): UserMessageSent | undefined {
    if (!res.isFinalized) {
      throw new Error('Transaction not finalized')
    }
    const id = getMessageId(res)
    return this.getMessageById(id)
  }
}
