import {
  HexString,
  Invariant,
  Network,
  TESTNET_INVARIANT_ADDRESS,
  Tickmap,
  initGearApi,
  newFeeTier,
  newPoolKey,
  subscribeToNewHeads
} from '@invariant-labs/vara-sdk'
import dotenv from 'dotenv'

dotenv.config()

// 200 - token0 - token1 - 1ts
// 500 - token0 - token2 - 1ts
// 1000 - token1 - token2 - 1ts
// 10000 - token0 - token1 - 2ts
let tickmap: Tickmap
const TESTNET_TOKEN_0: HexString =
  '0xda68a9f2cbdff47a2f8ea99d0aeb38a16d3ede93478d57285113a724032235b6'
const TESTNET_TOKEN_1: HexString =
  '0x2135fc1e5b77b984540fdf7c7b23152d02b57c5bce6430192c42658c004b5fe1'
const TESTNET_TOKEN_2: HexString =
  '0x5f5526ebe2b58e1f2359dab4c0fd9005dece56443129482ee5e9f62f230fdbdf'
const FEE_TIER_ONE = newFeeTier(10000000000n, 1n)
const FEE_TIER_TWO = newFeeTier(10000000000n, 2n)
const POOL_KEY_ONE = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_1, FEE_TIER_ONE)
const POOL_KEY_TWO = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_2, FEE_TIER_ONE)
const POOL_KEY_THREE = newPoolKey(TESTNET_TOKEN_1, TESTNET_TOKEN_2, FEE_TIER_ONE)
const POOL_KEY_FOUR = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_1, FEE_TIER_TWO)

const main = async () => {
  const network = Network.Testnet

  const api = await initGearApi({ providerAddress: network })
  await subscribeToNewHeads(api)
  const invariant = await Invariant.load(api, TESTNET_INVARIANT_ADDRESS)

  // 200 ticks initialized
  {
    const poolKey = POOL_KEY_ONE
    {
      const timestampBefore = performance.now()
      tickmap = await invariant.getTickmap(poolKey)
      const timestampAfter = performance.now()
      console.log(
        'Time to get full tickmap with 200 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(tickmap.bitmap.size)
    }
    {
      const timestampBefore = performance.now()
      const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)

      const timestampAfter = performance.now()
      console.log(
        'Time to get all liquidity ticks with 200 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(ticks.length)

    }
  }

  // 500 ticks initialized
  {
    const poolKey = POOL_KEY_TWO
    {
      const timestampBefore = performance.now()
      tickmap = await invariant.getTickmap(poolKey)
      const timestampAfter = performance.now()
      console.log(
        'Time to get full tickmap with 500 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(tickmap.bitmap.size)
    }
    {
      const timestampBefore = performance.now()
      const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)

      const timestampAfter = performance.now()
      console.log(
        'Time to get all liquidity ticks with 500 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(ticks.length)
    }
  }

  // 1k ticks initialized
  {
    const poolKey = POOL_KEY_THREE
    {
      const timestampBefore = performance.now()
      tickmap = await invariant.getTickmap(poolKey)
      const timestampAfter = performance.now()
      console.log(
        'Time to get full tickmap with 1k ticks intialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(tickmap.bitmap.size)
    }
    {
      const timestampBefore = performance.now()
      const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = performance.now()
      console.log(
        'Time to get all liquidity ticks with 1k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(ticks.length)
    }
  }

  // 10k ticks initialized
  {
    const poolKey = POOL_KEY_FOUR
    {
      const timestampBefore = performance.now()
      tickmap = await invariant.getTickmap(poolKey)
      const timestampAfter = performance.now()
      console.log(
        'Time to get full tickmap with 10k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(tickmap.bitmap.size)
    }
    {
      const timestampBefore = performance.now()
      const ticks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = performance.now()
      console.log(
        'Time to get all liquidity ticks with 10k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
      console.log(ticks.length)
    }
  }

  process.exit(0)
}

main()
