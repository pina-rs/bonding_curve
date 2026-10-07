# `@pina-rs/bonding-curve`

TypeScript client for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve), a Solana launchpad that sells a new token along a configurable price curve and then graduates it into the [Pina AMM](https://github.com/pina-rs/amm), for [`@solana/kit`](https://github.com/anza-xyz/kit). Generated from the program's IDL.

```sh
pnpm add @pina-rs/bonding-curve @solana/kit
```

```ts
import {
	fetchLaunch,
	findLaunchPda,
	getBuyInstruction,
} from "@pina-rs/bonding-curve";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

const [launchAddress] = await findLaunchPda({ baseMint });
const { data: launch } = await fetchLaunch(rpc, launchAddress);
const instruction = getBuyInstruction({
	trader: wallet,
	config: launch.config,
	launch: launchAddress,
	traderBase,
	traderQuote,
	baseVault: launch.baseVault,
	quoteVault: launch.quoteVault,
	baseTokenProgram: TOKEN_PROGRAM_ADDRESS,
	quoteTokenProgram: TOKEN_PROGRAM_ADDRESS,
	quoteAmountIn: 1_000_000_000n,
	minimumBaseOut: 990_000_000n,
});
```

Full guide: [docs/typescript.md](https://github.com/pina-rs/bonding_curve/blob/main/docs/typescript.md).
