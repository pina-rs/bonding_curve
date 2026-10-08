import { describe, expect, it } from "vitest";
import {
	describeLaunch,
	feeRateAt,
	graduate,
	migrationPoint,
	millionsAtPlotX,
	ONE_SEGMENT,
	PLOT,
	plotX,
	plotY,
	pointAtBase,
	pointAtQuote,
	poolPriceAfter,
	price,
	quoteBuy,
	segmentEnds,
	solPerMillionTokens,
	TERMS,
	THREE_SEGMENTS,
} from "./launch-curve";

const Q64 = 2 ** 64;

function expectClose(actual: number, expected: number, relative = 1e-6): void {
	expect(Math.abs(actual - expected) / expected).toBeLessThan(relative);
}

describe("the reference curve from docs/curves.md", () => {
	it("starts at about 0.000028 lamports per base unit", () => {
		expectClose(price(ONE_SEGMENT.startSqrtPrice), 0.000028, 1e-4);
		expectClose(
			solPerMillionTokens(price(ONE_SEGMENT.startSqrtPrice)),
			0.028,
			1e-4,
		);
	});

	it("derives the documented migration price and sale supply", () => {
		const completed = migrationPoint(ONE_SEGMENT);

		expectClose(completed.sqrtPrice, 411_204_649_253_062_377 / Q64);
		expectClose(completed.baseSold, 720_619_552_472_693);
		expect(completed.quoteRaised).toBe(TERMS.migrationQuote);
		expectClose(
			price(completed.sqrtPrice) / price(ONE_SEGMENT.startSqrtPrice),
			17.7,
			0.01,
		);
	});

	it("graduates as docs/graduation.md describes", () => {
		const graduation = graduate(ONE_SEGMENT);

		expectClose(graduation.poolQuote, 83.3e9);
		expectClose(graduation.poolBase, 167.6e12, 0.001);
		expectClose(graduation.burned, 61.7e12, 0.005);
		expectClose(
			poolPriceAfter(graduation, 0),
			price(migrationPoint(ONE_SEGMENT).sqrtPrice),
		);
	});
});

describe("walking the curve", () => {
	it("agrees whether walked by quote or by base", () => {
		for (const curve of [ONE_SEGMENT, THREE_SEGMENTS]) {
			for (const quote of [1e9, 12e9, 40e9, 84e9]) {
				const byQuote = pointAtQuote(curve, quote);
				const byBase = pointAtBase(curve, byQuote.baseSold);
				expectClose(byBase.quoteRaised, quote, 1e-9);
				expectClose(byBase.sqrtPrice, byQuote.sqrtPrice, 1e-9);
			}
		}
	});

	it("raises 85 SOL inside the three segments, with a steeper start", () => {
		const ends = segmentEnds(THREE_SEGMENTS);

		expect(migrationPoint(THREE_SEGMENTS).quoteRaised).toBe(
			TERMS.migrationQuote,
		);
		expect(migrationPoint(THREE_SEGMENTS).baseSold).toBeLessThan(
			ends.at(-1) ?? 0,
		);
		expect(price(pointAtQuote(THREE_SEGMENTS, 10e9).sqrtPrice)).toBeGreaterThan(
			price(pointAtQuote(ONE_SEGMENT, 10e9).sqrtPrice),
		);
	});

	it("stops at the top of the last segment", () => {
		const exhausted = pointAtQuote(ONE_SEGMENT, 1e15);

		expect(exhausted.sqrtPrice).toBe(ONE_SEGMENT.segments[0]?.upperSqrtPrice);
		expect(exhausted.quoteRaised).toBeLessThan(1e15);
	});
});

describe("fees and buys", () => {
	it("decays the trading fee from 50% to 1% over a minute", () => {
		expect(feeRateAt(0)).toBe(0.5);
		expectClose(feeRateAt(30), 0.255);
		expect(feeRateAt(60)).toBe(0.01);
		expect(feeRateAt(600)).toBe(0.01);
	});

	it("charges a sniper half of their buy", () => {
		const start = pointAtQuote(ONE_SEGMENT, 0);
		const sniped = quoteBuy(ONE_SEGMENT, start, 1e9, 0);
		const patient = quoteBuy(ONE_SEGMENT, start, 1e9, 60);

		expect(sniped.fee).toBe(0.5e9);
		expect(patient.base).toBeGreaterThan(sniped.base * 1.9);
	});

	it("stops a buy at the migration price and charges only what the curve used", () => {
		const nearlyDone = pointAtQuote(ONE_SEGMENT, 84.5e9);
		const buy = quoteBuy(ONE_SEGMENT, nearlyDone, 10e9, 60);

		expect(buy.completes).toBe(true);
		expectClose(buy.paid, 0.5e9 / 0.99);
		expectClose(
			buy.base,
			migrationPoint(ONE_SEGMENT).baseSold - nearlyDone.baseSold,
		);
	});
});

describe("plot and readout", () => {
	it("maps the plot corners and inverts the x axis", () => {
		expect(plotX(PLOT.tokens[0])).toBe(PLOT.left);
		expect(plotY(PLOT.price[1])).toBe(PLOT.top);
		expectClose(millionsAtPlotX(plotX(432.1)), 432.1);
	});

	it("describes a launch halfway to its target", () => {
		const halfway = pointAtQuote(ONE_SEGMENT, 42.5e9);
		const summary = describeLaunch(ONE_SEGMENT, halfway, 0);

		expect(summary.raised).toBe("42.5 of 85 SOL");
		expect(summary.sold).toMatch(/^\d{3}\.\dM tokens$/);
		expect(summary.price).toMatch(/^0\.\d{1,3} SOL per 1M$/);
		expect(summary.buy).toMatch(/M tokens after a 50% fee$/);
		expect(describeLaunch(ONE_SEGMENT, migrationPoint(ONE_SEGMENT), 0).buy)
			.toBe("Trading has stopped");
	});
});
