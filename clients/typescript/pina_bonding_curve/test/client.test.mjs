// Smoke tests against the built bundle: run `pnpm build` first.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
	BUY_DISCRIMINATOR,
	getBuyInstructionDataEncoder,
	PINA_BONDING_CURVE_PROGRAM_ADDRESS,
} from "../dist/index.js";

test("exports the deployed program address", () => {
	assert.equal(
		PINA_BONDING_CURVE_PROGRAM_ADDRESS,
		"CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9",
	);
});

test("encodes buy data as discriminator then little-endian fields", () => {
	const bytes = getBuyInstructionDataEncoder().encode({
		quoteAmountIn: 1_000_000_000n,
		minimumBaseOut: 42n,
	});
	assert.equal(bytes.length, 17);
	assert.equal(bytes[0], BUY_DISCRIMINATOR);
	const view = new DataView(bytes.buffer, bytes.byteOffset);
	assert.equal(view.getBigUint64(1, true), 1_000_000_000n);
	assert.equal(view.getBigUint64(9, true), 42n);
});
